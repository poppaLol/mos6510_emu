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