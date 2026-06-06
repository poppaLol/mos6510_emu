#[macro_use]
extern crate bitflags;
extern crate byte;

mod flags;
mod cia;
mod memory;
mod proc;
mod screen;
mod vic;
use flags::{ Flags, AddressingMode, get_mode};
use proc::{Mos6510, ProcDelta};
use memory::C64Memory;
use screen::{render_text_screen, screen_code_to_ascii, SCREEN_HEIGHT, SCREEN_RAM_START, SCREEN_WIDTH};
use std::num::Wrapping;
use std::collections::VecDeque;
use std::env;
use std::panic::{self, AssertUnwindSafe};
use round::round_down;
use tokio::time;

const UPPER_BIT_POS: u8 = 0b10000000;
const LOWER_BIT_POS: u8 = 0b00000001;
const NMI_VECTOR: u16 = 0xFFFA;
const RESET_VECTOR: u16 = 0xFFFC;
const IRQ_BRK_VECTOR: u16 = 0xFFFE;
const KEYBOARD_BUFFER_COUNT: usize = 0x00C6;
const KEYBOARD_BUFFER_START: usize = 0x0277;
const KEYBOARD_WAIT_START: u16 = 0xE5CD;
const KEYBOARD_WAIT_END: u16 = 0xE5D4;

#[allow(dead_code)]
enum Interrupt {
    Irq,
    Nmi,
}

struct BootOptions {
    max_instructions: Option<usize>,
    trace_tail: usize,
    trace_memory: bool,
    verbose: bool,
    screen: bool,
    stop_on_brk: bool,
    stop_outside_rom: bool,
    stop_pc: Option<u16>,
    stop_pc_range: Option<(u16, u16)>,
    watch_stack_word: Option<u16>,
    watch_stack_value: Option<u16>,
    dump_zero_page: bool,
    dump_screen_ram: bool,
    typed_input: VecDeque<u8>,
}

#[derive(Copy, Clone)]
struct TraceEntry {
    index: usize,
    pc: u16,
    op_code: u8,
    c1_pointer: u16,
    c1_effective_address: u16,
    accumulator: u8,
    x_index: u8,
    y_index: u8,
    stack_pointer: u8,
    processor_status: u8,
    cycles_count: usize,
    memory_latch: u8,
    cia2_port_a: u8,
    cia2_data_direction_a: u8,
    stack_next_word: u16,
}

impl TraceEntry {
    fn from_cpu(index: usize, op_code: u8, memory: &C64Memory, proc: &Mos6510) -> TraceEntry {
        let c1_pointer = (memory.ram[0xC2] as u16) << 8 | memory.ram[0xC1] as u16;
        let stack_low = 0x100 + proc.stack_pointer.wrapping_add(1) as usize;
        let stack_high = 0x100 + proc.stack_pointer.wrapping_add(2) as usize;

        TraceEntry {
            index,
            pc: proc.program_counter,
            op_code,
            c1_pointer,
            c1_effective_address: (Wrapping(c1_pointer) + Wrapping(proc.y_index as u16)).0,
            accumulator: proc.accumulator,
            x_index: proc.x_index,
            y_index: proc.y_index,
            stack_pointer: proc.stack_pointer,
            processor_status: proc.processor_status.bits(),
            cycles_count: proc.cycles_count,
            memory_latch: memory.ram[1],
            cia2_port_a: memory.cia2.read_byte(0xDD00),
            cia2_data_direction_a: memory.cia2.read_byte(0xDD02),
            stack_next_word: (memory.ram[stack_high] as u16) << 8 | memory.ram[stack_low] as u16,
        }
    }
}

fn parse_u16_arg(value: &str, name: &str) -> u16 {
    if let Some(hex_value) = value.strip_prefix("0x") {
        u16::from_str_radix(hex_value, 16).unwrap_or_else(|_| {
            panic!("invalid {} value: {}", name, value);
        })
    } else {
        value.parse::<u16>().unwrap_or_else(|_| {
            panic!("invalid {} value: {}", name, value);
        })
    }
}

fn parse_typed_input(value: &str) -> VecDeque<u8> {
    let mut chars = value.chars();
    let mut bytes = VecDeque::new();

    while let Some(ch) = chars.next() {
        let value = if ch == '\\' {
            match chars.next() {
                Some('n') | Some('r') => 0x0D,
                Some('t') => b'\t',
                Some('\\') => b'\\',
                Some('"') => b'"',
                Some(other) => other as u8,
                None => b'\\',
            }
        } else if ch == '\n' || ch == '\r' {
            0x0D
        } else {
            ch.to_ascii_uppercase() as u8
        };

        bytes.push_back(value);
    }

    bytes
}

fn parse_boot_options() -> BootOptions {
    let mut max_instructions = None;
    let mut trace_tail = 0;
    let mut trace_memory = false;
    let mut verbose = false;
    let mut screen = false;
    let mut stop_on_brk = false;
    let mut stop_outside_rom = false;
    let mut stop_pc = None;
    let mut stop_pc_range = None;
    let mut watch_stack_word = None;
    let mut watch_stack_value = None;
    let mut dump_zero_page = false;
    let mut dump_screen_ram = false;
    let mut typed_input = VecDeque::new();
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--screen" => {
                screen = true;
            },
            "--stop-on-brk" => {
                stop_on_brk = true;
            },
            "--stop-outside-rom" => {
                stop_outside_rom = true;
            },
            "--max-instructions" => {
                let value = args.next().unwrap_or_else(|| {
                    panic!("--max-instructions requires a value");
                });
                max_instructions = Some(value.parse::<usize>().unwrap_or_else(|_| {
                    panic!("invalid --max-instructions value: {}", value);
                }));
            },
            "--trace-tail" => {
                let value = args.next().unwrap_or_else(|| {
                    panic!("--trace-tail requires a value");
                });
                trace_tail = value.parse::<usize>().unwrap_or_else(|_| {
                    panic!("invalid --trace-tail value: {}", value);
                });
            },
            "--trace-memory" => {
                trace_memory = true;
            },
            "--verbose" => {
                verbose = true;
            },
            "--stop-pc" => {
                let value = args.next().unwrap_or_else(|| {
                    panic!("--stop-pc requires a value");
                });
                stop_pc = Some(parse_u16_arg(&value, "--stop-pc"));
            },
            "--stop-pc-range" => {
                let start = args.next().unwrap_or_else(|| {
                    panic!("--stop-pc-range requires a start value");
                });
                let end = args.next().unwrap_or_else(|| {
                    panic!("--stop-pc-range requires an end value");
                });
                stop_pc_range = Some((
                    parse_u16_arg(&start, "--stop-pc-range start"),
                    parse_u16_arg(&end, "--stop-pc-range end"),
                ));
            },
            "--watch-stack-word" => {
                let value = args.next().unwrap_or_else(|| {
                    panic!("--watch-stack-word requires an address");
                });
                watch_stack_word = Some(parse_u16_arg(&value, "--watch-stack-word"));
            },
            "--watch-stack-value" => {
                let value = args.next().unwrap_or_else(|| {
                    panic!("--watch-stack-value requires a value");
                });
                watch_stack_value = Some(parse_u16_arg(&value, "--watch-stack-value"));
            },
            "--dump-zero-page" => {
                dump_zero_page = true;
            },
            "--dump-screen-ram" => {
                dump_screen_ram = true;
            },
            "--type" => {
                let value = args.next().unwrap_or_else(|| {
                    panic!("--type requires a value");
                });
                typed_input.extend(parse_typed_input(&value));
            },
            _ => panic!("unknown argument: {}", arg),
        }
    }

    BootOptions {
        max_instructions,
        trace_tail,
        trace_memory,
        verbose,
        screen,
        stop_on_brk,
        stop_outside_rom,
        stop_pc,
        stop_pc_range,
        watch_stack_word,
        watch_stack_value,
        dump_zero_page,
        dump_screen_ram,
        typed_input,
    }
}

fn print_screen_snapshot(memory: &C64Memory, options: &BootOptions) {
    if options.screen {
        println!("C64 text screen:");
        println!("{}", render_text_screen(memory));
    }
}

fn print_hex_dump(memory: &C64Memory, start: usize, len: usize) {
    for offset in (0..len).step_by(16) {
        print!("{:#06x}:", start + offset);
        for i in 0..16 {
            if offset + i < len {
                print!(" {:02x}", memory.ram[start + offset + i]);
            } else {
                print!("   ");
            }
        }
        print!("  |");
        for i in 0..16 {
            if offset + i < len {
                let value = memory.ram[start + offset + i];
                let ch = if value.is_ascii_graphic() || value == b' ' {
                    value as char
                } else {
                    '.'
                };
                print!("{}", ch);
            }
        }
        println!("|");
    }
}

fn print_zero_page_dump(memory: &C64Memory, options: &BootOptions) {
    if options.dump_zero_page {
        println!("Zero page RAM dump:");
        print_hex_dump(memory, 0x0000, 0x0100);
    }
}

fn print_screen_ram_dump(memory: &C64Memory, options: &BootOptions) {
    if !options.dump_screen_ram {
        return;
    }

    println!("Screen RAM dump:");
    for row in 0..SCREEN_HEIGHT {
        let row_start = SCREEN_RAM_START + row * SCREEN_WIDTH;
        print!("{:#06x}:", row_start);
        for col in 0..SCREEN_WIDTH {
            print!(" {:02x}", memory.ram[row_start + col]);
        }
        print!("  |");
        for col in 0..SCREEN_WIDTH {
            print!("{}", screen_code_to_ascii(memory.ram[row_start + col]));
        }
        println!("|");
    }
}

fn print_checkpoint_diagnostics(memory: &C64Memory, options: &BootOptions) {
    print_screen_snapshot(memory, options);
    print_zero_page_dump(memory, options);
    print_screen_ram_dump(memory, options);
}

fn remember_trace(trace: &mut VecDeque<TraceEntry>, entry: TraceEntry, max_len: usize) {
    if max_len == 0 {
        return;
    }
    if trace.len() == max_len {
        trace.pop_front();
    }
    trace.push_back(entry);
}

fn trace_enabled(options: &BootOptions) -> bool {
    options.trace_tail > 0
        || options.watch_stack_word.is_some()
}

fn print_trace_tail(trace: &VecDeque<TraceEntry>) {
    if trace.is_empty() {
        return;
    }

    println!("Recent instruction trace:");
    for entry in trace {
        println!(
            "#{:<6} PC={:#06x} OP={:#04x} PTR_C1={:#06x} EFF_C1Y={:#06x} A={:#04x} X={:#04x} Y={:#04x} SP={:#04x} RET={:#06x} SR={:#04x} LATCH={:#04x} DD00={:#04x} DD02={:#04x} CC={}",
            entry.index,
            entry.pc,
            entry.op_code,
            entry.c1_pointer,
            entry.c1_effective_address,
            entry.accumulator,
            entry.x_index,
            entry.y_index,
            entry.stack_pointer,
            entry.stack_next_word,
            entry.processor_status,
            entry.memory_latch,
            entry.cia2_port_a,
            entry.cia2_data_direction_a,
            entry.cycles_count
        );
    }
}

fn is_rom_address(address: u16) -> bool {
    matches!(address, 0xA000..=0xBFFF | 0xE000..=0xFFFF)
}

fn is_ram_code_address(address: u16) -> bool {
    matches!(address, 0x0200..=0x9FFF | 0xC000..=0xCFFF)
}

fn read_raw_word(memory: &C64Memory, address: u16) -> u16 {
    let low_address = address as usize;
    let high_address = address.wrapping_add(1) as usize;

    (memory.ram[high_address] as u16) << 8 | memory.ram[low_address] as u16
}

fn feed_typed_input(memory: &mut C64Memory, proc: &Mos6510, typed_input: &mut VecDeque<u8>) -> bool {
    if typed_input.is_empty()
        || memory.ram[KEYBOARD_BUFFER_COUNT] != 0
        || proc.program_counter < KEYBOARD_WAIT_START
        || proc.program_counter > KEYBOARD_WAIT_END
    {
        return false;
    }

    if let Some(ch) = typed_input.pop_front() {
        memory.ram[KEYBOARD_BUFFER_START] = ch;
        memory.ram[KEYBOARD_BUFFER_COUNT] = 1;
        true
    } else {
        false
    }
}

fn panic_summary(error: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = error.downcast_ref::<String>() {
        message.clone()
    } else if let Some(message) = error.downcast_ref::<&str>() {
        message.to_string()
    } else {
        "unknown panic".to_string()
    }
}

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

fn set_compare_status(left: u8, right: u8, proc: Mos6510) -> Mos6510 {
    let diff = left.wrapping_sub(right);
    let mut proc = set_proc_status(Flags::N_FLAG | Flags::Z_FLAG, diff, None, (0,0), proc);
    if left >= right {
        proc.processor_status |= Flags::C_FLAG;
    } else {
        proc.processor_status &= !Flags::C_FLAG;
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

fn read_jmp_indirect_word(memory: &C64Memory, address: u16) -> u16 {
    let low = memory.read_byte(address) as u16;
    let high_address = (address & 0xFF00) | (address.wrapping_add(1) & 0x00FF);
    let high = memory.read_byte(high_address) as u16;
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
        AddressingMode::Indirect    => read_jmp_indirect_word(memory, memory.read_word(proc.program_counter + 1))
    }
}

fn nop(mut proc: Mos6510) -> Mos6510 {
    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(1);
    proc
}

fn brk(memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
    proc.program_counter += 2;
    interrupt(memory, proc, Interrupt::Irq, true)
}

fn ora(memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
    let read_address = get_read_address(&memory, &proc);
    let extra = proc.addressing_mode.crossed_page_boundary(proc.program_counter + 1, read_address);
    proc.accumulator |= memory.read_byte(read_address);

    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(extra);

    set_proc_status(Flags::N_FLAG | Flags::Z_FLAG, proc.accumulator, None, (0,0), proc)
}

fn and(memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
    let read_address = get_read_address(&memory, &proc);
    let extra = proc.addressing_mode.crossed_page_boundary(proc.program_counter + 1, read_address);
    proc.accumulator &= memory.read_byte(read_address);

    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(extra);

    set_proc_status(Flags::N_FLAG | Flags::Z_FLAG, proc.accumulator, None, (0,0), proc)
}

fn anc(memory: &mut C64Memory, proc: Mos6510) -> Mos6510 {
    let temp = and(memory, proc);
    let carry_option: Option<u8> = match temp.processor_status.intersects(Flags::N_FLAG) {
        true => None,
        false => Some(0)
    };
    set_proc_status(Flags::C_FLAG, 0, carry_option, (0,0), temp)
}

fn eor(memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
    let read_address = get_read_address(&memory, &proc);
    let extra = proc.addressing_mode.crossed_page_boundary_indy(proc.program_counter + 1, read_address);
    proc.accumulator ^= memory.read_byte(read_address);

    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(extra);

    set_proc_status(Flags::N_FLAG | Flags::Z_FLAG, proc.accumulator, None, (0,0), proc)
}

fn bit(memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
    let read_address = get_read_address(&memory, &proc);
    let value = memory.read_byte(read_address);

    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(0);

    match proc.accumulator & value == 0 {
        true => proc.processor_status |= Flags::Z_FLAG,
        false => proc.processor_status &= !Flags::Z_FLAG
    };
    match value & Flags::N_FLAG.bits() > 0 {
        true => proc.processor_status |= Flags::N_FLAG,
        false => proc.processor_status &= !Flags::N_FLAG
    };
    match value & Flags::V_FLAG.bits() > 0 {
        true => proc.processor_status |= Flags::V_FLAG,
        false => proc.processor_status &= !Flags::V_FLAG
    };

    proc
}

fn adc(memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
    let read_address = get_read_address(&memory, &proc);
    let extra = proc.addressing_mode.crossed_page_boundary_xzero(proc.program_counter + 1, read_address);

    let orig_val = proc.accumulator;
    let right_operand = memory.read_byte(read_address);

    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(extra);

    let extra_add = if proc.processor_status.contains(Flags::C_FLAG) {1u16} else {0u16};

    let sum = proc.accumulator as u16 + right_operand as u16 + extra_add;
    let carry = if sum > 0xFF { None } else { Some(sum as u8) };
    proc.accumulator = sum as u8;
    set_proc_status(
        Flags::N_FLAG | Flags::Z_FLAG | Flags::C_FLAG | Flags::V_FLAG,
        proc.accumulator, carry, (orig_val, proc.accumulator), proc)
}

fn sbc(memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
    let read_address = get_read_address(&memory, &proc);
    let extra = proc.addressing_mode.crossed_page_boundary_indy(proc.program_counter + 1, read_address);

    let orig_val = proc.accumulator;
    let right_operand = memory.read_byte(read_address);

    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(extra);

    let extra_sub = if proc.processor_status.contains(Flags::C_FLAG) {0u16} else {1u16};

    let subtrahend = right_operand as u16 + extra_sub;
    let sum = (proc.accumulator as u16).wrapping_sub(subtrahend);
    let carry = if proc.accumulator as u16 >= subtrahend {
        None
    } else {
        Some(sum as u8)
    };
    proc.accumulator = sum as u8;
    set_proc_status(
        Flags::N_FLAG | Flags::Z_FLAG | Flags::C_FLAG | Flags::V_FLAG,
        proc.accumulator, carry, (orig_val, proc.accumulator), proc)
}

fn cmp(memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
    let read_address = get_read_address(&memory, &proc);
    let extra = proc.addressing_mode.crossed_page_boundary_indy(proc.program_counter + 1, read_address);

    let right_operand = memory.read_byte(read_address);

    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(extra);

    //nb - does not set the accumulator
    set_compare_status(proc.accumulator, right_operand, proc)
}

fn cpx(memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
    let read_address = get_read_address(&memory, &proc);

    let right_operand = memory.read_byte(read_address);

    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(0);

    set_compare_status(proc.x_index, right_operand, proc)
}

fn cpy(memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
    let read_address = get_read_address(&memory, &proc);

    let right_operand = memory.read_byte(read_address);

    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(0);

    set_compare_status(proc.y_index, right_operand, proc)
}

fn dec(memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
    let read_address = get_read_address(&memory, &proc);

    let orig_value = memory.read_byte(read_address);

    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(0);

    let res = Wrapping(orig_value) - Wrapping(1);
    memory.write_byte(&(read_address as usize), res.0);
    
    set_proc_status(Flags::N_FLAG | Flags::Z_FLAG, res.0, None, (0,0), proc)
}

fn dex(_memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(0);

    let res = Wrapping(proc.x_index) - Wrapping(1);
    proc.x_index = res.0;
    set_proc_status(Flags::N_FLAG | Flags::Z_FLAG, res.0, None, (0,0), proc)
}

fn dey(_memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(0);

    let res = Wrapping(proc.y_index) - Wrapping(1);
    proc.y_index = res.0;
    set_proc_status(Flags::N_FLAG | Flags::Z_FLAG, res.0, None, (0,0), proc)
}

fn inc(memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
    let read_address = get_read_address(&memory, &proc);

    let orig_value = memory.read_byte(read_address);

    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(0);

    let res = Wrapping(orig_value) + Wrapping(1);
    memory.write_byte(&(read_address as usize), res.0);
    
    set_proc_status(Flags::N_FLAG | Flags::Z_FLAG, res.0, None, (0,0), proc)
}

fn inx(_memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(0);

    let res = Wrapping(proc.x_index) + Wrapping(1);
    proc.x_index = res.0;
    set_proc_status(Flags::N_FLAG | Flags::Z_FLAG, res.0, None, (0,0), proc)
}

fn iny(_memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(0);

    let res = Wrapping(proc.y_index) + Wrapping(1);
    proc.y_index = res.0;
    set_proc_status(Flags::N_FLAG | Flags::Z_FLAG, res.0, None, (0,0), proc)
}


fn asl(memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
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

    set_proc_status(Flags::N_FLAG | Flags::Z_FLAG | Flags::C_FLAG, res, shifted_out, (0,0), proc)
}

fn rol(memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
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

    set_proc_status(Flags::N_FLAG | Flags::Z_FLAG | Flags::C_FLAG, res, shifted_out, (0,0), proc)
}

fn lsr(memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
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

    set_proc_status(Flags::N_FLAG | Flags::Z_FLAG | Flags::C_FLAG, res, shifted_out, (0,0), proc)
}

fn ror(memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
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

    set_proc_status(Flags::N_FLAG | Flags::Z_FLAG | Flags::C_FLAG, res, shifted_out, (0,0), proc)
}

fn lda(memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
    let read_address = get_read_address(&memory, &proc);
    let extra = proc.addressing_mode.crossed_page_boundary(proc.program_counter + 1, read_address);
    proc.accumulator = memory.read_byte(read_address);

    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(extra);

    set_proc_status(Flags::N_FLAG | Flags::Z_FLAG, proc.accumulator, None, (0,0), proc)
}

fn sta(memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
    let write_address = get_read_address(&memory, &proc);
    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(0); //todo - ensure correct vals
    memory.write_byte(&(write_address as usize), proc.accumulator);
    proc
}

fn ldx(memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
    let read_address = get_read_address(&memory, &proc);
    //load to x register - todo - cycles/etc - and test
    proc.x_index = memory.read_byte(read_address);
    proc.program_counter += proc.addressing_mode.bytes_increment();
    set_proc_status(Flags::N_FLAG | Flags::Z_FLAG, proc.x_index, None, (0,0), proc)
}

fn stx(memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
    let write_address = get_read_address(&memory, &proc);
    memory.write_byte(&(write_address as usize), proc.x_index);
    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc
}

fn ldy(memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
    let read_address = get_read_address(&memory, &proc);
    //load to y register - todo - cycles/etc - and test
    proc.y_index = memory.read_byte(read_address);
    proc.program_counter += proc.addressing_mode.bytes_increment();
    set_proc_status(Flags::N_FLAG | Flags::Z_FLAG, proc.y_index, None, (0,0), proc)
}

fn sty(memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
    let write_address = get_read_address(&memory, &proc);
    memory.write_byte(&(write_address as usize), proc.y_index);
    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc
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
    set_proc_status(Flags::N_FLAG | Flags::Z_FLAG, proc.x_index, None, (0,0), proc)
}

fn txs(mut proc: Mos6510) -> Mos6510 {
    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(0);
    proc.stack_pointer = proc.x_index;
    proc
}

fn pla(memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(0);
    proc.accumulator = stack_read_byte(memory, proc);
    let delta = ProcDelta::empty().with_stack_pointer(proc.stack_pointer.wrapping_add(1));
    proc = delta.apply_proc_delta(proc);
    set_proc_status(Flags::N_FLAG | Flags::Z_FLAG, proc.accumulator, None, (0,0), proc)
}

fn pha(memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(0);
    stack_push_byte(memory, proc, proc.accumulator)
}


fn plp(memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(0);
    proc.processor_status = Flags::from_bits(stack_read_byte(memory, proc)).unwrap_or(Flags::ALWAYS);
    proc = ProcDelta::empty().with_stack_pointer(proc.stack_pointer.wrapping_add(1)).apply_proc_delta(proc);
    set_proc_status(Flags::N_FLAG | Flags::Z_FLAG, proc.accumulator, None, (0,0), proc)
}


fn php(memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc.cycles_count += proc.addressing_mode.cycles_increment(0);
    stack_push_byte(memory, proc, proc.processor_status.bits())
}

fn branch_on(memory: &mut C64Memory, mut proc: Mos6510, branch_cond: bool) -> Mos6510 {
    let mut extra = 0;
    let next_instruction = proc.program_counter.wrapping_add(proc.addressing_mode.bytes_increment());
    if branch_cond {
        let read_address = get_read_address(&memory, &proc);
        let jump_offset = memory.read_byte(read_address) as i8;
        proc.program_counter = (next_instruction as i32 + jump_offset as i32) as u16;
        extra = 1 + proc.addressing_mode.crossed_page_boundary(next_instruction, proc.program_counter);
    } else {
        proc.program_counter = next_instruction;
    }
    proc.cycles_count += proc.addressing_mode.cycles_increment(extra);
    
    proc
}

fn bpl(memory: &mut C64Memory, proc: Mos6510) -> Mos6510 {
    let cond = !proc.processor_status.contains(Flags::N_FLAG);
    branch_on(memory, proc, cond)
}

fn bmi(memory: &mut C64Memory, proc: Mos6510) -> Mos6510 {
    let cond = proc.processor_status.contains(Flags::N_FLAG);
    branch_on(memory, proc, cond)
}

fn bvc(memory: &mut C64Memory, proc: Mos6510) -> Mos6510 {
    let cond = !proc.processor_status.contains(Flags::V_FLAG);
    branch_on(memory, proc, cond)
}

fn bvs(memory: &mut C64Memory, proc: Mos6510) -> Mos6510 {
    let cond = proc.processor_status.contains(Flags::V_FLAG);
    branch_on(memory, proc, cond)
}

fn bcc(memory: &mut C64Memory, proc: Mos6510) -> Mos6510 {
    let cond = !proc.processor_status.contains(Flags::C_FLAG);
    branch_on(memory, proc, cond)
}

fn bcs(memory: &mut C64Memory, proc: Mos6510) -> Mos6510 {
    let cond = proc.processor_status.contains(Flags::C_FLAG);
    branch_on(memory, proc, cond)
}

fn bne(memory: &mut C64Memory, proc: Mos6510) -> Mos6510 {
    let cond = !proc.processor_status.contains(Flags::Z_FLAG);
    branch_on(memory, proc, cond)
}

fn beq(memory: &mut C64Memory, proc: Mos6510) -> Mos6510 {
    let cond = proc.processor_status.contains(Flags::Z_FLAG);
    branch_on(memory, proc, cond)
}

fn jmp(memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
    proc.program_counter = get_read_address(&memory, &proc);
    proc
}


fn clc(_memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
    proc.processor_status = proc.processor_status & !Flags::C_FLAG;
    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc
}


fn sec(_memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
    proc.processor_status = proc.processor_status | Flags::C_FLAG;
    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc
}

fn cld(_memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
    proc.processor_status = proc.processor_status & !Flags::D_FLAG;
    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc
}

fn sed(_memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
    proc.processor_status = proc.processor_status | Flags::D_FLAG;
    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc
}

fn cli(_memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
    proc.processor_status = proc.processor_status & !Flags::I_FLAG;
    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc
}

fn sei(_memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
    proc.processor_status = proc.processor_status | Flags::I_FLAG;
    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc
}

fn clv(_memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
    proc.processor_status = proc.processor_status & !Flags::V_FLAG;
    proc.program_counter += proc.addressing_mode.bytes_increment();
    proc
}

fn jsr(memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
    let rts = proc.program_counter + 2;
    proc.program_counter = memory.read_word(proc.program_counter + 1);
    let mut res = stack_push_word(memory, proc, rts);
    res.cycles_count += 6;
    res
}


fn stack_push_word(memory: &mut C64Memory, mut proc: Mos6510, val: u16) -> Mos6510 {
    memory.stack_push_word(proc.stack_pointer as usize, val);
    proc.stack_pointer = proc.stack_pointer.wrapping_sub(2);
    proc
}


fn stack_push_byte(memory: &mut C64Memory, mut proc: Mos6510, val: u8) -> Mos6510 {
    memory.stack_push_byte(proc.stack_pointer as usize, val);
    proc.stack_pointer = proc.stack_pointer.wrapping_sub(1);
    proc
}


fn stack_read_word(memory: &mut C64Memory, proc: Mos6510) -> u16 {
    memory.stack_pop_word(proc.stack_pointer as u16)
}

fn stack_read_byte(memory: &mut C64Memory, proc: Mos6510) -> u8 {
    memory.stack_pop_byte(proc.stack_pointer as u16)
}

fn interrupt(memory: &mut C64Memory, mut proc: Mos6510, interrupt: Interrupt, break_flag: bool) -> Mos6510 {
    let vector = match interrupt {
        Interrupt::Nmi => NMI_VECTOR,
        Interrupt::Irq => IRQ_BRK_VECTOR,
    };
    let return_address = proc.program_counter;
    let mut stored_status = proc.processor_status | Flags::ALWAYS;
    if break_flag {
        stored_status |= Flags::B_FLAG;
    } else {
        stored_status &= !Flags::B_FLAG;
    }

    proc.processor_status = proc.processor_status | Flags::ALWAYS | Flags::I_FLAG;
    if break_flag {
        proc.processor_status |= Flags::B_FLAG;
    } else {
        proc.processor_status &= !Flags::B_FLAG;
    }
    proc.program_counter = memory.read_word(vector);
    proc.cycles_count += 7;

    let with_stored_pc = stack_push_word(memory, proc, return_address);
    stack_push_byte(memory, with_stored_pc, stored_status.bits())
}

fn service_pending_interrupt(memory: &mut C64Memory, proc: Mos6510) -> (Mos6510, bool) {
    if memory.irq_pending() && !proc.processor_status.contains(Flags::I_FLAG) {
        memory.acknowledge_irq();
        (interrupt(memory, proc, Interrupt::Irq, false), true)
    } else {
        (proc, false)
    }
}

fn rts(memory: &mut C64Memory, proc: Mos6510) -> Mos6510 {
    let target_address = stack_read_word(memory, proc);
    let delta = ProcDelta::empty()
        .with_stack_pointer(proc.stack_pointer.wrapping_add(2))
        .with_program_counter(target_address.wrapping_add(1))
        .with_cycles_count(6);
    delta.apply_proc_delta(proc)
}

fn rti(memory: &mut C64Memory, mut proc: Mos6510) -> Mos6510 {
    proc.processor_status = Flags::from_bits_truncate(stack_read_byte(memory, proc)) | Flags::ALWAYS;
    proc.stack_pointer = proc.stack_pointer.wrapping_add(1);

    let target_address = stack_read_word(memory, proc);
    let delta = ProcDelta::empty()
        .with_stack_pointer(proc.stack_pointer.wrapping_add(2))
        .with_program_counter(target_address)
        .with_cycles_count(6);
    delta.apply_proc_delta(proc)
}

fn execute_opcode(memory: &mut C64Memory, mut proc: Mos6510, op_code: u8) -> Mos6510 {
    proc = ProcDelta::empty()
            .with_address_mode(get_mode(&op_code))
            .apply_proc_delta(proc);
    match op_code {
        //Arith
        0x1A | 0x3A | 0x5A | 0x7A | 0xDA | 0xEA | 0xFA => nop(proc),
        0x01 | 0x05 | 0x09 | 0x0D | 0x11 | 0x15 | 0x19 | 0x1D => ora(memory, proc),
        0x29 | 0x2D | 0x3D | 0x39 | 0x25 | 0x35 | 0x21 | 0x31 => and(memory, proc),
        0x0B | 0x2B => anc(memory, proc),
        0x49 | 0x45 | 0x55 | 0x41 | 0x51 | 0x4D | 0x5D | 0x59 => eor(memory, proc),
        0x24 | 0x2C => bit(memory, proc),
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
        0xAA => tax(proc),
        0x8A => txa(proc),
        0xA8 => tay(proc),
        0x98 => tya(proc),
        0xBA => tsx(proc),
        0x9A => txs(proc),
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
        0x40 => rti(memory, proc),
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


#[allow(dead_code)]
fn process_control_loop(memory: &mut C64Memory, proc: Mos6510) -> Mos6510 {
    let op_code = memory.read_byte(proc.program_counter);
    execute_opcode(memory, proc, op_code)
}


#[tokio::main]
async fn main() {
    let mut options = parse_boot_options();
    let mut memory = C64Memory::init_memory(
        "rom/64c.251913-01.bin",
        "rom/characters.901225-01.bin"
    );
    memory.trace = options.trace_memory;
    let pc = memory.read_word(RESET_VECTOR);
    let mut proc = ProcDelta::empty()
        .with_program_counter(pc)
        .with_cycles_count(6)
        .apply_proc_delta(Mos6510::boot_up());
    
    let pal_duration = round_down(1.0/985_000.0*1_000_000_000.0,0) as u64;
    //let ntsc_duration = round_down(1.0/1_023_000.0*1_000_000_000.0,0) as u64;

    let mut interval = time::interval(time::Duration::from_nanos(pal_duration));
    let mut i = 0;
    let collect_trace = trace_enabled(&options);
    let mut trace = VecDeque::with_capacity(options.trace_tail);
    let mut has_entered_rom = is_rom_address(proc.program_counter);
    loop {
        if let Some(max_instructions) = options.max_instructions {
            if i >= max_instructions {
                println!(
                    "Checkpoint: reached --max-instructions={} at PC={:#06x}, cycles={}",
                    max_instructions,
                    proc.program_counter,
                    proc.cycles_count
                );
                print_trace_tail(&trace);
                print_checkpoint_diagnostics(&memory, &options);
                break;
            }
        }

        if options.verbose {
            println!("({})", i);
            print!("{:#06x} ", proc.program_counter);
        }
        if options.max_instructions.is_none() {
            interval.tick().await;
        }

        feed_typed_input(&mut memory, &proc, &mut options.typed_input);

        let interrupt_start_cycles = proc.cycles_count;
        let pending_interrupt = service_pending_interrupt(&mut memory, proc);
        proc = pending_interrupt.0;
        if pending_interrupt.1 {
            memory.tick(proc.cycles_count - interrupt_start_cycles);
            i += 1;
            continue;
        }

        let op_code = memory.read_byte(proc.program_counter);
        if collect_trace {
            remember_trace(&mut trace, TraceEntry::from_cpu(i, op_code, &memory, &proc), options.trace_tail);
        }

        if options.stop_on_brk && op_code == 0x00 {
            println!(
                "Checkpoint: stopped on BRK opcode at instruction {} PC={:#06x}, cycles={}",
                i,
                proc.program_counter,
                proc.cycles_count
            );
            print_trace_tail(&trace);
            print_checkpoint_diagnostics(&memory, &options);
            break;
        }

        if let Some(stop_pc) = options.stop_pc {
            if proc.program_counter == stop_pc {
                println!(
                    "Checkpoint: reached --stop-pc {:#06x} at instruction {}, cycles={}",
                    stop_pc,
                    i,
                    proc.cycles_count
                );
                print_trace_tail(&trace);
                print_checkpoint_diagnostics(&memory, &options);
                break;
            }
        }

        if let Some((start, end)) = options.stop_pc_range {
            if proc.program_counter >= start && proc.program_counter <= end {
                println!(
                    "Checkpoint: reached --stop-pc-range {:#06x}..={:#06x} at PC={:#06x}, instruction {}, cycles={}",
                    start,
                    end,
                    proc.program_counter,
                    i,
                    proc.cycles_count
                );
                print_trace_tail(&trace);
                print_checkpoint_diagnostics(&memory, &options);
                break;
            }
        }

        if options.stop_outside_rom && has_entered_rom && is_ram_code_address(proc.program_counter) {
            println!(
                "Checkpoint: stopped outside ROM at instruction {} PC={:#06x}, cycles={}",
                i,
                proc.program_counter,
                proc.cycles_count
            );
            print_trace_tail(&trace);
            print_checkpoint_diagnostics(&memory, &options);
            break;
        }

        let watched_stack_word_before = options
            .watch_stack_word
            .map(|address| read_raw_word(&memory, address));
        let instruction_start_cycles = proc.cycles_count;
        let execution = panic::catch_unwind(AssertUnwindSafe(|| execute_opcode(&mut memory, proc, op_code)));
        match execution {
            Ok(res) => {
                proc = res;
                memory.tick(proc.cycles_count - instruction_start_cycles);
                has_entered_rom = has_entered_rom || is_rom_address(proc.program_counter);

                if let Some(address) = options.watch_stack_word {
                    let before = watched_stack_word_before.unwrap();
                    let after = read_raw_word(&memory, address);
                    let matches_value = options
                        .watch_stack_value
                        .map_or(true, |expected| after == expected);

                    if before != after && matches_value {
                        println!(
                            "Checkpoint: stack word {:#06x} changed from {:#06x} to {:#06x} after instruction {} at PC={:#06x}, cycles={}",
                            address,
                            before,
                            after,
                            i,
                            trace.back().map_or(0, |entry| entry.pc),
                            proc.cycles_count
                        );
                        print_trace_tail(&trace);
                        print_checkpoint_diagnostics(&memory, &options);
                        break;
                    }
                }
            },
            Err(error) => {
                println!(
                    "Stopped after {} instructions at PC={:#06x}: {}",
                    i,
                    proc.program_counter,
                    panic_summary(error)
                );
                print_trace_tail(&trace);
                print_checkpoint_diagnostics(&memory, &options);
                break;
            }
        }
        
        i+=1;
        if options.verbose {
            print!(" ----- AddressMode {:?}", proc.addressing_mode);
            if i > 5 && proc.stack_pointer < 1 {
                println!("Warning - stack pointer {}", proc.stack_pointer);
            }
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
#[cfg(test)] mod test_input;
#[cfg(test)] mod test_interrupt;
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
#[cfg(test)] mod test_stack;
#[cfg(test)] mod test_bit;
