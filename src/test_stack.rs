use super::{get_cpu, jsr, pha, php, pla, rts, stack_push_byte, stack_push_word, tsx, txs, AddressingMode, C64Memory, Flags};

#[test]
fn stack_push_byte_wraps_stack_pointer_below_zero() {
    let mut mem = C64Memory::get_empty_mem();
    let mut cpu = get_cpu();
    cpu.stack_pointer = 0x00;

    let res = stack_push_byte(&mut mem, cpu, 0xAA);

    assert_eq!(res.stack_pointer, 0xFF);
    assert_eq!(mem.ram[0x0100], 0xAA);
}

#[test]
fn stack_push_word_wraps_stack_pointer_below_zero() {
    let mut mem = C64Memory::get_empty_mem();
    let mut cpu = get_cpu();
    cpu.stack_pointer = 0x01;

    let res = stack_push_word(&mut mem, cpu, 0xBEEF);

    assert_eq!(res.stack_pointer, 0xFF);
    assert_eq!(mem.ram[0x0101], 0xBE);
    assert_eq!(mem.ram[0x0100], 0xEF);
}

#[test]
fn pla_wraps_stack_pointer_above_ff() {
    let mut mem = C64Memory::get_empty_mem();
    let mut cpu = get_cpu();
    cpu.addressing_mode = AddressingMode::Implied;
    cpu.stack_pointer = 0xFF;
    mem.ram[0x0100] = 0x42;

    let res = pla(&mut mem, cpu);

    assert_eq!(res.stack_pointer, 0x00);
    assert_eq!(res.accumulator, 0x42);
}

#[test]
fn pha_pushes_accumulator_and_decrements_stack_pointer() {
    let mut mem = C64Memory::get_empty_mem();
    let mut cpu = get_cpu();
    cpu.addressing_mode = AddressingMode::Implied;
    cpu.program_counter = 0xC000;
    cpu.stack_pointer = 0xFF;
    cpu.accumulator = 0x42;

    let res = pha(&mut mem, cpu);

    assert_eq!(res.program_counter, 0xC001);
    assert_eq!(res.stack_pointer, 0xFE);
    assert_eq!(mem.ram[0x01FF], 0x42);
}

#[test]
fn php_pushes_status_and_decrements_stack_pointer() {
    let mut mem = C64Memory::get_empty_mem();
    let mut cpu = get_cpu();
    cpu.addressing_mode = AddressingMode::Implied;
    cpu.program_counter = 0xC000;
    cpu.stack_pointer = 0xFF;

    let res = php(&mut mem, cpu);

    assert_eq!(res.program_counter, 0xC001);
    assert_eq!(res.stack_pointer, 0xFE);
    assert_eq!(mem.ram[0x01FF], cpu.processor_status.bits());
}

#[test]
fn tsx_transfers_stack_pointer_to_x_and_advances_one_byte() {
    let mut cpu = get_cpu();
    cpu.addressing_mode = AddressingMode::Implied;
    cpu.program_counter = 0xC000;
    cpu.stack_pointer = 0x7F;
    cpu.x_index = 0x00;

    let res = tsx(cpu);

    assert_eq!(res.program_counter, 0xC001);
    assert_eq!(res.x_index, 0x7F);
}

#[test]
fn tsx_sets_status_flags_from_x_register() {
    let mut cpu = get_cpu();
    cpu.addressing_mode = AddressingMode::Implied;
    cpu.stack_pointer = 0x80;
    cpu.accumulator = 0x00;
    cpu.processor_status &= !Flags::N_FLAG;

    let res = tsx(cpu);

    assert!(res.processor_status.contains(Flags::N_FLAG));
    assert!(!res.processor_status.contains(Flags::Z_FLAG));
}

#[test]
fn txs_transfers_x_to_stack_pointer_and_advances_one_byte_without_flags() {
    let mut cpu = get_cpu();
    cpu.addressing_mode = AddressingMode::Implied;
    cpu.program_counter = 0xC000;
    cpu.stack_pointer = 0x00;
    cpu.x_index = 0x80;
    cpu.processor_status = Flags::ALWAYS | Flags::Z_FLAG;

    let res = txs(cpu);

    assert_eq!(res.program_counter, 0xC001);
    assert_eq!(res.stack_pointer, 0x80);
    assert_eq!(res.processor_status, Flags::ALWAYS | Flags::Z_FLAG);
}

#[test]
fn rts_wraps_stack_pointer_above_ff() {
    let mut mem = C64Memory::get_empty_mem();
    let mut cpu = get_cpu();
    cpu.stack_pointer = 0xFE;
    mem.ram[0x01FF] = 0x34;
    mem.ram[0x0100] = 0x12;

    let res = rts(&mut mem, cpu);

    assert_eq!(res.stack_pointer, 0x00);
    assert_eq!(res.program_counter, 0x1235);
}

#[test]
fn jsr_pushes_address_of_last_operand_byte_and_jumps_to_target() {
    let mut mem = C64Memory::get_empty_mem();
    let mut cpu = get_cpu();
    cpu.program_counter = 0xC000;
    cpu.stack_pointer = 0xFF;
    mem.ram[0xC001] = 0x34;
    mem.ram[0xC002] = 0x12;

    let res = jsr(&mut mem, cpu);

    assert_eq!(res.program_counter, 0x1234);
    assert_eq!(res.stack_pointer, 0xFD);
    assert_eq!(mem.ram[0x01FF], 0xC0);
    assert_eq!(mem.ram[0x01FE], 0x02);
}

#[test]
fn rts_returns_to_byte_after_pulled_stack_address() {
    let mut mem = C64Memory::get_empty_mem();
    let mut cpu = get_cpu();
    cpu.stack_pointer = 0xFD;
    mem.ram[0x01FE] = 0x02;
    mem.ram[0x01FF] = 0xC0;

    let res = rts(&mut mem, cpu);

    assert_eq!(res.stack_pointer, 0xFF);
    assert_eq!(res.program_counter, 0xC003);
}

#[test]
fn jsr_and_rts_round_trip_to_next_instruction() {
    let mut mem = C64Memory::get_empty_mem();
    let mut cpu = get_cpu();
    cpu.program_counter = 0xC000;
    cpu.stack_pointer = 0xFF;
    mem.ram[0xC001] = 0x34;
    mem.ram[0xC002] = 0x12;

    let jumped = jsr(&mut mem, cpu);
    let returned = rts(&mut mem, jumped);

    assert_eq!(returned.stack_pointer, 0xFF);
    assert_eq!(returned.program_counter, 0xC003);
}
