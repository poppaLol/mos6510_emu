#[macro_use]
extern crate bitflags;
extern crate byte;

mod flags;
mod memory;
mod proc;
use flags::{ Flags, AddressingMode, get_mode};
use proc::{Mos6510, ProcDelta};
use memory::C64Memory;
use std::num::Wrapping;
use round::round_down;
use tokio::time;

const UPPER_BIT_POS: u8 = 0b10000000;
const LOWER_BIT_POS: u8 = 0b00000001;

fn set_proc_status(flag_mask: Flags, value: u8, wrapping: Option<u8>, overflow_tuple: (u8,u8), mut proc: Mos6510) -> Mos6510 {
    if flag_mask.contains(Flags::N_FLAG) {
        match value & 0b10000000 > 0 { //2's compliment negative
            true => proc.processor_status |= Flags::N_FLAG,
            false => proc.processor_status &= !Flags::N_FLAG
        };
    }
    if flag_mask.contains(Flags::Z_FLAG) {
        match value == 0 {
            true => proc.processor_status |= Flags::Z_FLAG,
            false => proc.processor_status &= !Flags::Z_FLAG
        };
    }
    if flag_mask.contains(Flags::V_FLAG) && !flag_mask.contains(Flags::D_FLAG) {
        match (overflow_tuple.0 >> 7) == 0 && (overflow_tuple.1 >> 7) > 0 {
            true => proc.processor_status |= Flags::V_FLAG,
            false => proc.processor_status &= !Flags::V_FLAG
        };
    }
    if flag_mask.contains(Flags::C_FLAG) {
        match wrapping {
            None => proc.processor_status |= Flags::C_FLAG,
            Some(_) => proc.processor_status &= !Flags::C_FLAG,
        } 
    }
    proc
}


fn zero_page_add(base: u8, offset: u8) -> u16 {
    base.wrapping_add(offset) as u16
}

fn read_zero_page_word(memory: &C64Memory, address: u8) -> u16 {
    let low = memory.read_byte(address as u16) as u16;
    let high = memory.read_byte(address.wrapping_add(1) as u16) as u16;
    high << 8 | low
}

fn get_read_address(memory: &C64Memory, proc: &Mos6510) -> u16 {
    match proc.addressing_mode {
        AddressingMode::Implied     => std::panic::panic_any(format!("Implied does not read bytes!! {:#04x}", proc.program_counter)),
        AddressingMode::Relative |
        AddressingMode::Immediate   => proc.program_counter + 1,
        AddressingMode::Absolute    => memory.read_word(proc.program_counter + 1),
        AddressingMode::XAbsolute   => memory.read_word(proc.program_counter + 1) + proc.x_index as u16,
        AddressingMode::YAbsolute   => memory.read_word(proc.program_counter + 1) + proc.y_index as u16,
        AddressingMode::ZeroPage    => memory.read_byte(proc.program_counter + 1) as u16,
        AddressingMode::XZeroPage   => zero_page_add(memory.read_byte(proc.program_counter + 1), proc.x_index),
        AddressingMode::YZeroPage   => zero_page_add(memory.read_byte(proc.program_counter + 1), proc.y_index),
        AddressingMode::XIndirect   => read_zero_page_word(memory, memory.read_byte(proc.program_counter + 1).wrapping_add(proc.x_index)),
        AddressingMode::YIndirect   => read_zero_page_word(memory, memory.read_byte(proc.program_counter + 1)) + proc.y_index as u16,
        AddressingMode::Indirect    => memory.read_word(memory.read_word(proc.program_counter + 1))
    }
}

fn nop(mut proc: Mos6510) -> Mos6510 {
    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(1);
    proc
}

fn brk(memory: C64Memory, mut proc: Mos6510) -> (C64Memory, Mos6510) {
    proc.processor_status = proc.processor_status | Flags::B_FLAG | Flags::I_FLAG;
    
    proc.program_counter += 2;
    proc.cycles_count += 7;

    let with_stored_pc = stack_push_word(memory, proc, proc.program_counter);
    stack_push_byte(with_stored_pc.0, with_stored_pc.1, proc.processor_status.bits())
}

fn ora(memory: C64Memory, mut proc: Mos6510) -> (C64Memory, Mos6510) {
    let read_address = get_read_address(&memory, &proc);
    let extra = proc.addressing_mode.crossed_page_boundary(proc.program_counter + 1, read_address);
    proc.accumulator |= memory.read_byte(read_address);

    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(extra);

    (memory, set_proc_status(Flags::N_FLAG | Flags::Z_FLAG, proc.accumulator, None, (0,0), proc))
}

fn and(memory: C64Memory, mut proc: Mos6510) -> (C64Memory, Mos6510) {
    let read_address = get_read_address(&memory, &proc);
    let extra = proc.addressing_mode.crossed_page_boundary(proc.program_counter + 1, read_address);
    proc.accumulator &= memory.read_byte(read_address);

    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(extra);

    (memory, set_proc_status(Flags::N_FLAG | Flags::Z_FLAG, proc.accumulator, None, (0,0), proc))
}

fn anc(memory: C64Memory, proc: Mos6510) -> (C64Memory, Mos6510) {
    let temp = and(memory, proc);
    let carry_option: Option<u8> = match temp.1.processor_status.intersects(Flags::N_FLAG) {
        true => None,
        false => Some(0)
    };
    (temp.0, set_proc_status(Flags::C_FLAG, 0, carry_option, (0,0), temp.1))
}

fn eor(memory: C64Memory, mut proc: Mos6510) -> (C64Memory, Mos6510) {
    let read_address = get_read_address(&memory, &proc);
    let extra = proc.addressing_mode.crossed_page_boundary_indy(proc.program_counter + 1, read_address);
    proc.accumulator ^= memory.read_byte(read_address);

    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(extra);

    (memory, set_proc_status(Flags::N_FLAG | Flags::Z_FLAG, proc.accumulator, None, (0,0), proc))
}

fn adc(memory: C64Memory, mut proc: Mos6510) -> (C64Memory, Mos6510) {
    let read_address = get_read_address(&memory, &proc);
    let extra = proc.addressing_mode.crossed_page_boundary_xzero(proc.program_counter + 1, read_address);

    let orig_val = proc.accumulator;
    let right_operand = memory.read_byte(read_address);

    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(extra);

    let extra_add = if proc.processor_status.contains(Flags::C_FLAG) {1} else {0};

    let sum = Wrapping(proc.accumulator) + Wrapping(right_operand + extra_add);
    let carry = proc.accumulator.checked_add(right_operand + extra_add);
    proc.accumulator = sum.0;
    (memory, set_proc_status(
        Flags::N_FLAG | Flags::Z_FLAG | Flags::C_FLAG | Flags::V_FLAG,
        proc.accumulator, carry, (orig_val,sum.0), proc))
}

fn sbc(memory: C64Memory, mut proc: Mos6510) -> (C64Memory, Mos6510) {
    let read_address = get_read_address(&memory, &proc);
    let extra = proc.addressing_mode.crossed_page_boundary_indy(proc.program_counter + 1, read_address);

    let orig_val = proc.accumulator;
    let right_operand = memory.read_byte(read_address);

    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(extra);

    let extra_sub = if proc.processor_status.contains(Flags::C_FLAG) {0} else {1};

    let sum = Wrapping(proc.accumulator) - Wrapping(right_operand + extra_sub);
    let carry = match proc.accumulator.checked_sub(right_operand + extra_sub) {
        None => Some(sum.0),
        _ => None
    };
    proc.accumulator = sum.0;
    (memory, set_proc_status(
        Flags::N_FLAG | Flags::Z_FLAG | Flags::C_FLAG | Flags::V_FLAG,
        proc.accumulator, carry, (orig_val,sum.0), proc))
}

fn cmp(memory: C64Memory, mut proc: Mos6510) -> (C64Memory, Mos6510) {
    let read_address = get_read_address(&memory, &proc);
    let extra = proc.addressing_mode.crossed_page_boundary_indy(proc.program_counter + 1, read_address);

    let right_operand = memory.read_byte(read_address);

    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(extra);

    let diff = Wrapping(proc.accumulator) - Wrapping(right_operand);
    let carry = proc.accumulator.checked_sub(right_operand);
    //nb - does not set the accumulator
    (memory, set_proc_status(Flags::N_FLAG | Flags::Z_FLAG | Flags::C_FLAG, diff.0, carry, (0,0), proc))
}

fn cpx(memory: C64Memory, mut proc: Mos6510) -> (C64Memory, Mos6510) {
    let read_address = get_read_address(&memory, &proc);

    let right_operand = memory.read_byte(read_address);

    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(0);

    let diff = Wrapping(proc.x_index) - Wrapping(right_operand);
    let carry = proc.x_index.checked_sub(right_operand);

    (memory, set_proc_status(Flags::N_FLAG | Flags::Z_FLAG | Flags::C_FLAG, diff.0, carry, (0,0), proc))
}

fn cpy(memory: C64Memory, mut proc: Mos6510) -> (C64Memory, Mos6510) {
    let read_address = get_read_address(&memory, &proc);

    let right_operand = memory.read_byte(read_address);

    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(0);

    let diff = Wrapping(proc.y_index) - Wrapping(right_operand);
    let carry = proc.y_index.checked_sub(right_operand);

    (memory, set_proc_status(Flags::N_FLAG | Flags::Z_FLAG | Flags::C_FLAG, diff.0, carry, (0,0), proc))
}

fn dec(mut memory: C64Memory, mut proc: Mos6510) -> (C64Memory, Mos6510) {
    let read_address = get_read_address(&memory, &proc);

    let orig_value = memory.read_byte(read_address);

    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(0);

    let res = Wrapping(orig_value) - Wrapping(1);
    memory.write_byte(&(read_address as usize), res.0);
    
    (memory, set_proc_status(Flags::N_FLAG | Flags::Z_FLAG, res.0, None, (0,0), proc))
}

fn dex(memory: C64Memory, mut proc: Mos6510) -> (C64Memory, Mos6510) {
    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(0);

    let res = Wrapping(proc.x_index) - Wrapping(1);
    proc.x_index = res.0;
    (memory, set_proc_status(Flags::N_FLAG | Flags::Z_FLAG, res.0, None, (0,0), proc))
}

fn dey(memory: C64Memory, mut proc: Mos6510) -> (C64Memory, Mos6510) {
    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(0);

    let res = Wrapping(proc.y_index) - Wrapping(1);
    proc.y_index = res.0;
    (memory, set_proc_status(Flags::N_FLAG | Flags::Z_FLAG, res.0, None, (0,0), proc))
}

fn inc(mut memory: C64Memory, mut proc: Mos6510) -> (C64Memory, Mos6510) {
    let read_address = get_read_address(&memory, &proc);

    let orig_value = memory.read_byte(read_address);

    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(0);

    let res = Wrapping(orig_value) + Wrapping(1);
    memory.write_byte(&(read_address as usize), res.0);
    
    (memory, set_proc_status(Flags::N_FLAG | Flags::Z_FLAG, res.0, None, (0,0), proc))
}

fn inx(memory: C64Memory, mut proc: Mos6510) -> (C64Memory, Mos6510) {
    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(0);

    let res = Wrapping(proc.x_index) + Wrapping(1);
    proc.x_index = res.0;
    (memory, set_proc_status(Flags::N_FLAG | Flags::Z_FLAG, res.0, None, (0,0), proc))
}

fn iny(memory: C64Memory, mut proc: Mos6510) -> (C64Memory, Mos6510) {
    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(0);

    let res = Wrapping(proc.y_index) + Wrapping(1);
    proc.y_index = res.0;
    (memory, set_proc_status(Flags::N_FLAG | Flags::Z_FLAG, res.0, None, (0,0), proc))
}


fn asl(mut memory: C64Memory, mut proc: Mos6510) -> (C64Memory, Mos6510) {
    let orig_val = if matches!(proc.addressing_mode, AddressingMode::Implied) { 
        (proc.accumulator, None)
    } else {
        let read_address = get_read_address(&memory, &proc);
        (memory.read_byte(read_address), Some(read_address))
    };
    
    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.shift_cycles_increment();

    let res = orig_val.0 << 1;
    let shifted_out = if orig_val.0 & UPPER_BIT_POS > 0 { None } else { Some(res) };
    match proc.addressing_mode {
        AddressingMode::Implied => proc.accumulator = res,
        _ => memory.write_byte(&(orig_val.1.unwrap() as usize), res)
    }

    (memory, set_proc_status(Flags::N_FLAG | Flags::Z_FLAG | Flags::C_FLAG, res, shifted_out, (0,0), proc))
}

fn rol(mut memory: C64Memory, mut proc: Mos6510) -> (C64Memory, Mos6510) {
    let orig_val = if matches!(proc.addressing_mode, AddressingMode::Implied) { 
        (proc.accumulator, None)
    } else {
        let read_address = get_read_address(&memory, &proc);
        (memory.read_byte(read_address), Some(read_address))
    };

    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.shift_cycles_increment(); //todo - ensure correct vals
    let extra = if proc.processor_status.contains(Flags::C_FLAG) { 1 } else { 0 }; 
    let res = (orig_val.0 << 1) + extra;
    let shifted_out = if orig_val.0 & UPPER_BIT_POS > 0 { None } else { Some(res) };
    match proc.addressing_mode {
        AddressingMode::Implied => proc.accumulator = res,
        _ => memory.write_byte(&(orig_val.1.unwrap() as usize), res)
    }

    (memory, set_proc_status(Flags::N_FLAG | Flags::Z_FLAG | Flags::C_FLAG, res, shifted_out, (0,0), proc))
}

fn lsr(mut memory: C64Memory, mut proc: Mos6510) -> (C64Memory, Mos6510) {
    let orig_val = if matches!(proc.addressing_mode, AddressingMode::Implied) { 
        (proc.accumulator, None)
    } else {
        let read_address = get_read_address(&memory, &proc);
        (memory.read_byte(read_address), Some(read_address))
    };

    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(0); //todo - ensure correct vals
    let res = orig_val.0 >> 1;
    let shifted_out = if orig_val.0 & LOWER_BIT_POS > 0 { None } else { Some(res) };

    match proc.addressing_mode {
        AddressingMode::Implied => proc.accumulator = res,
        _ => memory.write_byte(&(orig_val.1.unwrap() as usize), res)
    }

    (memory, set_proc_status(Flags::N_FLAG | Flags::Z_FLAG | Flags::C_FLAG, res, shifted_out, (0,0), proc))
}

fn ror(mut memory: C64Memory, mut proc: Mos6510) -> (C64Memory, Mos6510) {
    let orig_val = if matches!(proc.addressing_mode, AddressingMode::Implied) { 
        (proc.accumulator, None)
    } else {
        let read_address = get_read_address(&memory, &proc);
        (memory.read_byte(read_address), Some(read_address))
    };

    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(0); //todo - ensure correct vals
    let extra = if proc.processor_status.contains(Flags::C_FLAG) { UPPER_BIT_POS } else { 0 }; 
    let res = (orig_val.0 >> 1) | extra;
    let shifted_out = if orig_val.0 & LOWER_BIT_POS > 0 { None } else { Some(res) };
    match proc.addressing_mode {
        AddressingMode::Implied => proc.accumulator = res,
        _ => memory.write_byte(&(orig_val.1.unwrap() as usize), res)
    }

    (memory, set_proc_status(Flags::N_FLAG | Flags::Z_FLAG | Flags::C_FLAG, res, shifted_out, (0,0), proc))
}

fn lda(memory: C64Memory, mut proc: Mos6510) -> (C64Memory, Mos6510) {
    let read_address = get_read_address(&memory, &proc);
    let extra = proc.addressing_mode.crossed_page_boundary(proc.program_counter + 1, read_address);
    proc.accumulator = memory.read_byte(read_address);

    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(extra);

    (memory, set_proc_status(Flags::N_FLAG | Flags::Z_FLAG, proc.accumulator, None, (0,0), proc))
}

fn sta(mut memory: C64Memory, mut proc: Mos6510) -> (C64Memory, Mos6510) {
    let write_address = get_read_address(&memory, &proc);
    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(0); //todo - ensure correct vals
    memory.write_byte(&(write_address as usize), proc.accumulator);
    (memory, proc)
}

fn ldx(memory: C64Memory, mut proc: Mos6510) -> (C64Memory, Mos6510) {
    let read_address = get_read_address(&memory, &proc);
    //load to x register - todo - cycles/etc - and test
    proc.x_index = memory.read_byte(read_address);
    proc.program_counter += proc.addressing_mode.bytes_increment();
    (memory, set_proc_status(Flags::N_FLAG | Flags::Z_FLAG, proc.x_index, None, (0,0), proc))
}

fn stx(mut memory: C64Memory, mut proc: Mos6510) -> (C64Memory, Mos6510) {
    let write_address = get_read_address(&memory, &proc);
    memory.write_byte(&(write_address as usize), proc.x_index);
    proc.program_counter += proc.addressing_mode.bytes_increment();
    (memory, proc)
}

fn ldy(memory: C64Memory, mut proc: Mos6510) -> (C64Memory, Mos6510) {
    let read_address = get_read_address(&memory, &proc);
    //load to y register - todo - cycles/etc - and test
    proc.y_index = memory.read_byte(read_address);
    proc.program_counter += proc.addressing_mode.bytes_increment();
    (memory, set_proc_status(Flags::N_FLAG | Flags::Z_FLAG, proc.x_index, None, (0,0), proc))
}

fn sty(mut memory: C64Memory, mut proc: Mos6510) -> (C64Memory, Mos6510) {
    let write_address = get_read_address(&memory, &proc);
    memory.write_byte(&(write_address as usize), proc.y_index);
    proc.program_counter += proc.addressing_mode.bytes_increment();
    (memory, proc)
}

fn tax(mut proc: Mos6510) -> Mos6510 {
    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(0);
    proc.x_index = proc.accumulator;
    set_proc_status(Flags::N_FLAG | Flags::Z_FLAG, proc.x_index, None, (0,0), proc)
}

fn txa(mut proc: Mos6510) -> Mos6510 {
    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(0);
    proc.accumulator = proc.x_index;
    set_proc_status(Flags::N_FLAG | Flags::Z_FLAG, proc.accumulator, None, (0,0), proc)
}

fn tay(mut proc: Mos6510) -> Mos6510 {
    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(0);
    proc.y_index = proc.accumulator;
    set_proc_status(Flags::N_FLAG | Flags::Z_FLAG, proc.y_index, None, (0,0), proc)
}

fn tya(mut proc: Mos6510) -> Mos6510 {
    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(0);
    proc.accumulator = proc.y_index;
    set_proc_status(Flags::N_FLAG | Flags::Z_FLAG, proc.accumulator, None, (0,0), proc)
}

fn tsx(mut proc: Mos6510) -> Mos6510 {
    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(0);
    proc.x_index = proc.stack_pointer;
    proc.program_counter += 1;
    set_proc_status(Flags::N_FLAG | Flags::Z_FLAG, proc.accumulator, None, (0,0), proc)
}

fn txs(mut proc: Mos6510) -> Mos6510 {
    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(0);
    proc.stack_pointer = proc.x_index;
    proc.program_counter += 1;
    proc
}

fn pla(memory: C64Memory, mut proc: Mos6510) -> (C64Memory, Mos6510) {
    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(0);
    proc.accumulator = stack_read_byte(memory, proc);
    let delta = ProcDelta::empty().with_stack_pointer(proc.stack_pointer + 1);
    proc = delta.apply_proc_delta(proc);
    (memory, set_proc_status(Flags::N_FLAG | Flags::Z_FLAG, proc.accumulator, None, (0,0), proc))
}

fn pha(memory: C64Memory, mut proc: Mos6510) -> (C64Memory, Mos6510) {
    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(0);
    stack_push_byte(memory, proc, proc.accumulator);
    (memory, proc)
}


fn plp(memory: C64Memory, mut proc: Mos6510) -> (C64Memory, Mos6510) {
    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(0);
    proc.processor_status = Flags::from_bits(stack_read_byte(memory, proc)).unwrap_or(Flags::ALWAYS);
    proc = ProcDelta::empty().with_stack_pointer(proc.stack_pointer + 1).apply_proc_delta(proc);
    (memory, set_proc_status(Flags::N_FLAG | Flags::Z_FLAG, proc.accumulator, None, (0,0), proc))
}


fn php(memory: C64Memory, mut proc: Mos6510) -> (C64Memory, Mos6510) {
    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(0);
    stack_push_byte(memory, proc, proc.processor_status.bits());
    (memory, proc)
}

fn branch_on(memory: C64Memory, mut proc: Mos6510, branch_cond: bool) -> (C64Memory, Mos6510) {
    let mut extra = 0;
    if branch_cond {
        let original_address = proc.program_counter;
        let read_address = get_read_address(&memory, &proc);
        let jump_offset = memory.read_byte(read_address) as i8;
        proc.program_counter = (proc.program_counter as i32 + jump_offset as i32) as u16;
        extra = 1 + proc.addressing_mode.crossed_page_boundary(original_address, proc.program_counter);
    }
    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(extra);
    
    (memory, proc)
}

fn bpl(memory: C64Memory, proc: Mos6510) -> (C64Memory, Mos6510) {
    let cond = !proc.processor_status.contains(Flags::N_FLAG);
    branch_on(memory, proc, cond)
}

fn bmi(memory: C64Memory, proc: Mos6510) -> (C64Memory, Mos6510) {
    let cond = proc.processor_status.contains(Flags::N_FLAG);
    branch_on(memory, proc, cond)
}

fn bvc(memory: C64Memory, proc: Mos6510) -> (C64Memory, Mos6510) {
    let cond = !proc.processor_status.contains(Flags::V_FLAG);
    branch_on(memory, proc, cond)
}

fn bvs(memory: C64Memory, proc: Mos6510) -> (C64Memory, Mos6510) {
    let cond = proc.processor_status.contains(Flags::V_FLAG);
    branch_on(memory, proc, cond)
}

fn bcc(memory: C64Memory, proc: Mos6510) -> (C64Memory, Mos6510) {
    let cond = !proc.processor_status.contains(Flags::C_FLAG);
    branch_on(memory, proc, cond)
}

fn bcs(memory: C64Memory, proc: Mos6510) -> (C64Memory, Mos6510) {
    let cond = proc.processor_status.contains(Flags::C_FLAG);
    branch_on(memory, proc, cond)
}

fn bne(memory: C64Memory, proc: Mos6510) -> (C64Memory, Mos6510) {
    let cond = !proc.processor_status.contains(Flags::Z_FLAG);
    branch_on(memory, proc, cond)
}

fn beq(memory: C64Memory, proc: Mos6510) -> (C64Memory, Mos6510) {
    let cond = proc.processor_status.contains(Flags::Z_FLAG);
    branch_on(memory, proc, cond)
}

fn jmp(memory: C64Memory, mut proc: Mos6510) -> (C64Memory, Mos6510) {
    proc.program_counter = get_read_address(&memory, &proc);
    (memory, proc)
}


fn clc(memory: C64Memory, mut proc: Mos6510) -> (C64Memory, Mos6510) {
    proc.processor_status = proc.processor_status & !Flags::C_FLAG;
    proc.program_counter += proc.addressing_mode.bytes_increment();
    (memory, proc)
}


fn sec(memory: C64Memory, mut proc: Mos6510) -> (C64Memory, Mos6510) {
    proc.processor_status = proc.processor_status | Flags::C_FLAG;
    proc.program_counter += proc.addressing_mode.bytes_increment();
    (memory, proc)
}

fn cld(memory: C64Memory, mut proc: Mos6510) -> (C64Memory, Mos6510) {
    proc.processor_status = proc.processor_status & !Flags::D_FLAG;
    proc.program_counter += proc.addressing_mode.bytes_increment();
    (memory, proc)
}

fn sed(memory: C64Memory, mut proc: Mos6510) -> (C64Memory, Mos6510) {
    proc.processor_status = proc.processor_status | Flags::D_FLAG;
    proc.program_counter += proc.addressing_mode.bytes_increment();
    (memory, proc)
}

fn cli(memory: C64Memory, mut proc: Mos6510) -> (C64Memory, Mos6510) {
    proc.processor_status = proc.processor_status & !Flags::I_FLAG;
    proc.program_counter += proc.addressing_mode.bytes_increment();
    (memory, proc)
}

fn sei(memory: C64Memory, mut proc: Mos6510) -> (C64Memory, Mos6510) {
    proc.processor_status = proc.processor_status | Flags::I_FLAG;
    proc.program_counter += proc.addressing_mode.bytes_increment();
    (memory, proc)
}

fn clv(memory: C64Memory, mut proc: Mos6510) -> (C64Memory, Mos6510) {
    proc.processor_status = proc.processor_status & !Flags::V_FLAG;
    proc.program_counter += proc.addressing_mode.bytes_increment();
    (memory, proc)
}

fn jsr(memory: C64Memory, mut proc: Mos6510) -> (C64Memory, Mos6510) {
    let rts = proc.program_counter + 3;
    proc.program_counter = memory.read_word(proc.program_counter + 1);
    stack_push_word(memory, proc, rts)
}


fn stack_push_word(mut memory: C64Memory, mut proc: Mos6510, val: u16) -> (C64Memory, Mos6510) {
    memory.stack_push_word(proc.stack_pointer as usize, val);
    proc.stack_pointer -= 2;
    (memory, proc)
}


fn stack_push_byte(mut memory: C64Memory, mut proc: Mos6510, val: u8) -> (C64Memory, Mos6510) {
    memory.stack_push_byte(proc.stack_pointer as usize, val);
    proc.stack_pointer -= 1;
    (memory, proc)
}


fn stack_read_word(mut memory: C64Memory, proc: Mos6510) -> u16 {
    memory.stack_pop_word(proc.stack_pointer as u16)
}

fn stack_read_byte(mut memory: C64Memory, proc: Mos6510) -> u8 {
    memory.stack_pop_byte(proc.stack_pointer as u16)
}

fn rts(memory: C64Memory, proc: Mos6510) -> (C64Memory, Mos6510) {
    let target_address = stack_read_word(memory, proc);
    let delta = ProcDelta::empty()
        .with_stack_pointer(proc.stack_pointer + 2)
        .with_program_counter(target_address);
    (memory, delta.apply_proc_delta(proc))
}


fn process_control_loop(memory: C64Memory, mut proc: Mos6510) -> (C64Memory, Mos6510) {
    print!("{:#06x} ", proc.program_counter);
    let op_code = memory.read_byte(proc.program_counter);
    proc = ProcDelta::empty()
            .with_address_mode(get_mode(&op_code))
            .apply_proc_delta(proc);
    match op_code {
        //Arith
        0x1A | 0x3A | 0x5A | 0x7A | 0xDA | 0xEA | 0xFA => (memory, nop(proc)),
        0x01 | 0x05 | 0x09 | 0x0D | 0x11 | 0x15 | 0x19 | 0x1D => ora(memory, proc),
        0x29 | 0x2D | 0x3D | 0x39 | 0x25 | 0x35 | 0x21 | 0x31 => and(memory, proc),
        0x0B | 0x2B => anc(memory, proc),
        0x49 | 0x45 | 0x55 | 0x41 | 0x51 | 0x4D | 0x5D | 0x59 => eor(memory, proc),
        0x69 | 0x65 | 0x75 | 0x61 | 0x71 | 0x6D | 0x7D | 0x79 => adc(memory, proc),
        0xE9 | 0xE5 | 0xF5 | 0xE1 | 0xF1 | 0xED | 0xFD | 0xF9 => sbc(memory, proc),
        0xC9 | 0xC5 | 0xD5 | 0xC1 | 0xD1 | 0xCD | 0xDD | 0xD9 => cmp(memory, proc),
        0xE0 | 0xE4 | 0xEC => cpx(memory, proc),
        0xC0 | 0xC4 | 0xCC => cpy(memory, proc),
        0xC6 | 0xD6 | 0xCE | 0xDE => dec(memory, proc),
        0xCA => dex(memory, proc),
        0x88 => dey(memory, proc),
        0xE6 | 0xF6 | 0xEE | 0xFE => inc(memory, proc),
        0xE8 => inx(memory, proc),
        0xC8 => iny(memory, proc),
        0x0A | 0x06 | 0x16 | 0x0E | 0x1E => asl(memory, proc),
        0x2A | 0x26 | 0x36 | 0x2E | 0x3E => rol(memory, proc),
        0x4A | 0x46 | 0x56 | 0x4E | 0x5E => lsr(memory, proc),
        0x6A | 0x66 | 0x76 | 0x6E | 0x7E => ror(memory, proc), 
        //Move
        0xA9 | 0xA5 | 0xB5 | 0xA1 | 0xB1 | 0xAD | 0xBD | 0xB9 => lda(memory, proc),
        0x85 | 0x95 | 0x81 | 0x91 | 0x8D | 0x9D | 0x99 => sta(memory, proc),
        0xA2 | 0xA6 | 0xB6 | 0xAE | 0xBE => ldx(memory, proc),
        0x86 | 0x96 | 0x8E => stx(memory, proc),
        0xA0 | 0xA4 | 0xB4 | 0xAC | 0xBC => ldy(memory, proc),
        0x84 | 0x94 | 0x8C => sty(memory, proc),
        0xAA => (memory, tax(proc)),
        0x8A => (memory, txa(proc)),
        0xA8 => (memory, tay(proc)),
        0x98 => (memory, tya(proc)),
        0xBA => (memory, tsx(proc)),
        0x9A => (memory, txs(proc)),
        0x68 => pla(memory, proc), 
        0x48 => pha(memory, proc), 
        0x28 => plp(memory, proc), 
        0x08 => php(memory, proc),
        //jump
        0x10 => bpl(memory, proc),
        0x30 => bmi(memory, proc),
        0x50 => bvc(memory, proc),
        0x70 => bvs(memory, proc),
        0x90 => bcc(memory, proc),
        0xB0 => bcs(memory, proc),
        0xD0 => bne(memory, proc),
        0xF0 => beq(memory, proc),
        
        0x4C | 0x6C => jmp(memory, proc),

        //assorted - need testing
        0x20 => jsr(memory, proc),
        0x60 => rts(memory, proc),
        0x18 => clc(memory, proc),
        0x38 => sec(memory, proc),
        0xD8 => cld(memory, proc),
        0xF8 => sed(memory, proc),
        0x58 => cli(memory, proc),
        0x78 => sei(memory, proc),
        0xB8 => clv(memory, proc),
        
        
        0x00 => brk(memory, proc),
        _ => std::panic::panic_any(proc.program_counter)
    }
}


#[tokio::main]
async fn main() {
    let mut memory = C64Memory::init_memory(
        "rom/64c.251913-01.bin",
        "rom/characters.901225-01.bin"
    );
    let pc = memory.read_word(0xFFFC);
    let mut proc = ProcDelta::empty()
        .with_program_counter(pc)
        .with_cycles_count(6)
        .apply_proc_delta(Mos6510::boot_up());
    
    let pal_duration = round_down(1.0/985_000.0*1_000_000_000.0,0) as u64;
    //let ntsc_duration = round_down(1.0/1_023_000.0*1_000_000_000.0,0) as u64;

    let mut interval = time::interval(time::Duration::from_nanos(pal_duration));
    let mut i = 0;
    loop {
        println!("({})", i);
        interval.tick().await;
        let res = process_control_loop(memory, proc);
        memory = res.0;
        proc = res.1;
        
        //dbg!(proc);
        i+=1;
        print!(" ----- AddressMode {:?}", proc.addressing_mode);
        if i > 5 && proc.stack_pointer < 1 {
            println!("Warning - stack pointer {}", proc.stack_pointer);
        }
    }
}


#[allow(dead_code)]
fn get_cpu() -> Mos6510 {
    Mos6510 {
        addressing_mode: AddressingMode::Implied,
        //The program_counter should be initialized to the word read from memory location 0xFFFC.
        //This will read from the KERNAL ROM due to the latch bits initialization.
        program_counter: 0x0,   // normally not zero to start
        stack_pointer: 0xFF,    // stack pointer set directly here - would be set by Kern Rom init
        processor_status: Flags::ALWAYS,
        accumulator: 0x0,
        x_index: 0x0,
        y_index: 0x0,
        cycles_count: 0
    }
}

#[cfg(test)] mod test_flags;
#[cfg(test)] mod test_brk;
#[cfg(test)] mod test_ora;
#[cfg(test)] mod test_and;
#[cfg(test)] mod test_nop;
#[cfg(test)] mod test_eor;
#[cfg(test)] mod test_adc;
#[cfg(test)] mod test_sbc;
#[cfg(test)] mod test_cmp;
#[cfg(test)] mod test_dec;
#[cfg(test)] mod test_cpx_cpy;
#[cfg(test)] mod test_bpl;
#[cfg(test)] mod test_bmi;
#[cfg(test)] mod test_asl;
#[cfg(test)] mod test_rol;
#[cfg(test)] mod test_lda;
#[cfg(test)] mod test_addressing;
