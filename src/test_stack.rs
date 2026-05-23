use super::{get_cpu, jsr, pla, rts, stack_push_byte, stack_push_word, AddressingMode, C64Memory};

#[test]
fn stack_push_byte_wraps_stack_pointer_below_zero() {
    let mem = C64Memory::get_empty_mem();
    let mut cpu = get_cpu();
    cpu.stack_pointer = 0x00;

    let res = stack_push_byte(mem, cpu, 0xAA);

    assert_eq!(res.1.stack_pointer, 0xFF);
    assert_eq!(res.0.ram[0x0100], 0xAA);
}

#[test]
fn stack_push_word_wraps_stack_pointer_below_zero() {
    let mem = C64Memory::get_empty_mem();
    let mut cpu = get_cpu();
    cpu.stack_pointer = 0x01;

    let res = stack_push_word(mem, cpu, 0xBEEF);

    assert_eq!(res.1.stack_pointer, 0xFF);
    assert_eq!(res.0.ram[0x0101], 0xBE);
    assert_eq!(res.0.ram[0x0100], 0xEF);
}

#[test]
fn pla_wraps_stack_pointer_above_ff() {
    let mut mem = C64Memory::get_empty_mem();
    let mut cpu = get_cpu();
    cpu.addressing_mode = AddressingMode::Implied;
    cpu.stack_pointer = 0xFF;
    mem.ram[0x0100] = 0x42;

    let res = pla(mem, cpu);

    assert_eq!(res.1.stack_pointer, 0x00);
    assert_eq!(res.1.accumulator, 0x42);
}

#[test]
fn rts_wraps_stack_pointer_above_ff() {
    let mut mem = C64Memory::get_empty_mem();
    let mut cpu = get_cpu();
    cpu.stack_pointer = 0xFE;
    mem.ram[0x01FF] = 0x34;
    mem.ram[0x0100] = 0x12;

    let res = rts(mem, cpu);

    assert_eq!(res.1.stack_pointer, 0x00);
    assert_eq!(res.1.program_counter, 0x1235);
}

#[test]
fn jsr_pushes_address_of_last_operand_byte_and_jumps_to_target() {
    let mut mem = C64Memory::get_empty_mem();
    let mut cpu = get_cpu();
    cpu.program_counter = 0xC000;
    cpu.stack_pointer = 0xFF;
    mem.ram[0xC001] = 0x34;
    mem.ram[0xC002] = 0x12;

    let res = jsr(mem, cpu);

    assert_eq!(res.1.program_counter, 0x1234);
    assert_eq!(res.1.stack_pointer, 0xFD);
    assert_eq!(res.0.ram[0x01FF], 0xC0);
    assert_eq!(res.0.ram[0x01FE], 0x02);
}

#[test]
fn rts_returns_to_byte_after_pulled_stack_address() {
    let mut mem = C64Memory::get_empty_mem();
    let mut cpu = get_cpu();
    cpu.stack_pointer = 0xFD;
    mem.ram[0x01FE] = 0x02;
    mem.ram[0x01FF] = 0xC0;

    let res = rts(mem, cpu);

    assert_eq!(res.1.stack_pointer, 0xFF);
    assert_eq!(res.1.program_counter, 0xC003);
}

#[test]
fn jsr_and_rts_round_trip_to_next_instruction() {
    let mut mem = C64Memory::get_empty_mem();
    let mut cpu = get_cpu();
    cpu.program_counter = 0xC000;
    cpu.stack_pointer = 0xFF;
    mem.ram[0xC001] = 0x34;
    mem.ram[0xC002] = 0x12;

    let jumped = jsr(mem, cpu);
    let returned = rts(jumped.0, jumped.1);

    assert_eq!(returned.1.stack_pointer, 0xFF);
    assert_eq!(returned.1.program_counter, 0xC003);
}
