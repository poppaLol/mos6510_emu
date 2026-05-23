use super::{get_cpu, pla, stack_push_byte, stack_push_word, rts, AddressingMode, C64Memory};

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
    assert_eq!(res.1.program_counter, 0x1234);
}
