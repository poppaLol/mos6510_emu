use super::{get_cpu, C64Memory, lda, AddressingMode};

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
