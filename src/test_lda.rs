use super::{get_cpu, C64Memory, lda, ldy, AddressingMode, Flags};

#[test]
fn when_lda_addr_mode_immediate_loads_next_byte_to_accumulator() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.program_counter = 0x3;
  cpu.accumulator = 0xDD;
  cpu.addressing_mode = AddressingMode::Immediate;
  mem.ram[4] = 0xEE;
  let res = lda(mem,cpu);
  assert_eq!(res.1.accumulator, 0xEE)
}

#[test]
fn when_lda_addr_mode_yindirect_uses_operand_as_zero_page_pointer() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.program_counter = 0xEA0C;
  cpu.addressing_mode = AddressingMode::YIndirect;
  cpu.y_index = 2;

  mem.ram[0xEA0D] = 0xD1;
  mem.ram[0xEA0E] = 0x88;
  mem.ram[0x00D1] = 0x00;
  mem.ram[0x00D2] = 0x04;
  mem.ram[0x0402] = 0x7B;

  let res = lda(mem,cpu);
  assert_eq!(res.1.accumulator, 0x7B)
}

#[test]
fn ldy_sets_zero_flag_from_y_register() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.program_counter = 0x3;
  cpu.x_index = 0x44;
  cpu.processor_status &= !Flags::Z_FLAG;
  cpu.addressing_mode = AddressingMode::Immediate;
  mem.ram[4] = 0x00;

  let res = ldy(mem,cpu);

  assert_eq!(res.1.y_index, 0x00);
  assert!(res.1.processor_status.contains(Flags::Z_FLAG));
}

#[test]
fn ldy_sets_negative_flag_from_y_register() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.program_counter = 0x3;
  cpu.x_index = 0x00;
  cpu.processor_status &= !Flags::N_FLAG;
  cpu.addressing_mode = AddressingMode::Immediate;
  mem.ram[4] = 0x80;

  let res = ldy(mem,cpu);

  assert_eq!(res.1.y_index, 0x80);
  assert!(res.1.processor_status.contains(Flags::N_FLAG));
  assert!(!res.1.processor_status.contains(Flags::Z_FLAG));
}
