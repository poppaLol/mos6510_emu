use super::{bit, get_cpu, AddressingMode, C64Memory, Flags};

#[test]
fn bit_zero_page_sets_zero_when_accumulator_and_memory_have_no_common_bits() {
    let mut mem = C64Memory::get_empty_mem();
    let mut cpu = get_cpu();
    cpu.addressing_mode = AddressingMode::ZeroPage;
    cpu.accumulator = 0x0F;
    mem.ram[1] = 0x80;
    mem.ram[0x80] = 0xF0;

    let res = bit(&mut mem, cpu);

    assert!(res.processor_status.contains(Flags::Z_FLAG));
}

#[test]
fn bit_zero_page_clears_zero_when_accumulator_and_memory_share_bits() {
    let mut mem = C64Memory::get_empty_mem();
    let mut cpu = get_cpu();
    cpu.addressing_mode = AddressingMode::ZeroPage;
    cpu.processor_status |= Flags::Z_FLAG;
    cpu.accumulator = 0x0F;
    mem.ram[1] = 0x80;
    mem.ram[0x80] = 0x01;

    let res = bit(&mut mem, cpu);

    assert!(!res.processor_status.contains(Flags::Z_FLAG));
}

#[test]
fn bit_copies_negative_and_overflow_from_memory_operand() {
    let mut mem = C64Memory::get_empty_mem();
    let mut cpu = get_cpu();
    cpu.addressing_mode = AddressingMode::ZeroPage;
    mem.ram[1] = 0x80;
    mem.ram[0x80] = 0xC0;

    let res = bit(&mut mem, cpu);

    assert!(res.processor_status.contains(Flags::N_FLAG));
    assert!(res.processor_status.contains(Flags::V_FLAG));
}

#[test]
fn bit_clears_negative_and_overflow_when_memory_operand_bits_are_clear() {
    let mut mem = C64Memory::get_empty_mem();
    let mut cpu = get_cpu();
    cpu.addressing_mode = AddressingMode::ZeroPage;
    cpu.processor_status |= Flags::N_FLAG | Flags::V_FLAG;
    mem.ram[1] = 0x80;
    mem.ram[0x80] = 0x3F;

    let res = bit(&mut mem, cpu);

    assert!(!res.processor_status.contains(Flags::N_FLAG));
    assert!(!res.processor_status.contains(Flags::V_FLAG));
}

#[test]
fn bit_zero_page_advances_program_counter_and_cycles() {
    let mut mem = C64Memory::get_empty_mem();
    let mut cpu = get_cpu();
    cpu.addressing_mode = AddressingMode::ZeroPage;
    mem.ram[1] = 0x80;

    let res = bit(&mut mem, cpu);

    assert_eq!(res.program_counter, 2);
    assert_eq!(res.cycles_count, 3);
}

#[test]
fn bit_absolute_advances_program_counter_and_cycles() {
    let mut mem = C64Memory::get_empty_mem();
    let mut cpu = get_cpu();
    cpu.addressing_mode = AddressingMode::Absolute;
    mem.ram[1] = 0x34;
    mem.ram[2] = 0x12;

    let res = bit(&mut mem, cpu);

    assert_eq!(res.program_counter, 3);
    assert_eq!(res.cycles_count, 4);
}
