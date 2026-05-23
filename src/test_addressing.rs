use super::{get_cpu, get_read_address, AddressingMode, C64Memory};

#[test]
fn xzero_page_wraps_inside_zero_page() {
    let mut mem = C64Memory::get_empty_mem();
    let mut cpu = get_cpu();
    cpu.program_counter = 0x3;
    cpu.addressing_mode = AddressingMode::XZeroPage;
    cpu.x_index = 0x20;
    mem.ram[4] = 0xF0;

    assert_eq!(get_read_address(&mem, &cpu), 0x10)
}

#[test]
fn yzero_page_wraps_inside_zero_page() {
    let mut mem = C64Memory::get_empty_mem();
    let mut cpu = get_cpu();
    cpu.program_counter = 0x3;
    cpu.addressing_mode = AddressingMode::YZeroPage;
    cpu.y_index = 0x20;
    mem.ram[4] = 0xF0;

    assert_eq!(get_read_address(&mem, &cpu), 0x10)
}

#[test]
fn xindirect_wraps_operand_plus_x_inside_zero_page() {
    let mut mem = C64Memory::get_empty_mem();
    let mut cpu = get_cpu();
    cpu.program_counter = 0x3;
    cpu.addressing_mode = AddressingMode::XIndirect;
    cpu.x_index = 0x20;
    mem.ram[4] = 0xF0;
    mem.ram[0x10] = 0x34;
    mem.ram[0x11] = 0x12;

    assert_eq!(get_read_address(&mem, &cpu), 0x1234)
}

#[test]
fn xindirect_zero_page_pointer_high_byte_wraps_to_zero() {
    let mut mem = C64Memory::get_empty_mem();
    let mut cpu = get_cpu();
    cpu.program_counter = 0x3;
    cpu.addressing_mode = AddressingMode::XIndirect;
    cpu.x_index = 0x01;
    mem.ram[4] = 0xFE;
    mem.ram[0xFF] = 0x78;
    mem.ram[0x00] = 0x56;

    assert_eq!(get_read_address(&mem, &cpu), 0x5678)
}

#[test]
fn yindirect_zero_page_pointer_high_byte_wraps_to_zero() {
    let mut mem = C64Memory::get_empty_mem();
    let mut cpu = get_cpu();
    cpu.program_counter = 0x3;
    cpu.addressing_mode = AddressingMode::YIndirect;
    cpu.y_index = 0x02;
    mem.ram[4] = 0xFF;
    mem.ram[0xFF] = 0x78;
    mem.ram[0x00] = 0x56;

    assert_eq!(get_read_address(&mem, &cpu), 0x567A)
}

#[test]
fn indirect_jmp_reads_pointer_word_normally() {
    let mut mem = C64Memory::get_empty_mem();
    let mut cpu = get_cpu();
    cpu.program_counter = 0x3;
    cpu.addressing_mode = AddressingMode::Indirect;
    mem.ram[4] = 0x34;
    mem.ram[5] = 0x12;
    mem.ram[0x1234] = 0x78;
    mem.ram[0x1235] = 0x56;

    assert_eq!(get_read_address(&mem, &cpu), 0x5678)
}

#[test]
fn indirect_jmp_reproduces_6502_page_boundary_bug() {
    let mut mem = C64Memory::get_empty_mem();
    let mut cpu = get_cpu();
    cpu.program_counter = 0x3;
    cpu.addressing_mode = AddressingMode::Indirect;
    mem.ram[4] = 0xFF;
    mem.ram[5] = 0x12;
    mem.ram[0x12FF] = 0x78;
    mem.ram[0x1200] = 0x56;
    mem.ram[0x1300] = 0x9A;

    assert_eq!(get_read_address(&mem, &cpu), 0x5678)
}
