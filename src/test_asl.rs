use super::{get_cpu, C64Memory, asl, AddressingMode, Flags};

#[test]
fn when_asl_addr_mode_implied_cycles_inc_by_2() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::Implied;
  let res = asl(&mut mem,cpu);
  assert_eq!(res.cycles_count, 2)
}

#[test]
fn when_asl_addr_mode_implied_pc_inc_by_1() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::Implied;
  let res = asl(&mut mem,cpu);
  assert_eq!(res.program_counter, 1)
}

#[test]
fn when_asl_addr_mode_implied_accumulator_shifted_left() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.accumulator = 0b010;
  cpu.addressing_mode = AddressingMode::Implied;
  let res = asl(&mut mem,cpu);
  assert!(!res.processor_status.contains(Flags::C_FLAG));
  assert_eq!(res.accumulator, 0b100)
}

#[test]
fn when_asl_addr_mode_implied_carry_set() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.accumulator = 0b10000001;
  cpu.addressing_mode = AddressingMode::Implied;
  let res = asl(&mut mem,cpu);
  assert!(res.processor_status.contains(Flags::C_FLAG));
  assert_eq!(res.accumulator, 0b10)
}

#[test]
fn when_asl_addr_mode_res_zero_sets_zero_flag() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.accumulator = 0;
  cpu.addressing_mode = AddressingMode::Implied;
  let res = asl(&mut mem,cpu);
  assert!(res.processor_status.contains(Flags::Z_FLAG))
}

#[test]
fn when_asl_addr_mode_res_non_zero_clears_zero_flag() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.accumulator = 0b10000001;
  cpu.addressing_mode = AddressingMode::Implied;
  let res = asl(&mut mem,cpu);
  assert!(!res.processor_status.contains(Flags::Z_FLAG))
}

#[test]
fn when_asl_addr_mode_res_neg_sets_neg_flag() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.accumulator = 0b1000000;
  cpu.addressing_mode = AddressingMode::Implied;
  let res = asl(&mut mem,cpu);
  assert!(res.processor_status.contains(Flags::N_FLAG))
}

#[test]
fn when_asl_addr_mode_res_non_neg_clears_neg_flag() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.accumulator = 0b100000;
  cpu.addressing_mode = AddressingMode::Implied;
  let res = asl(&mut mem,cpu);
  assert!(!res.processor_status.contains(Flags::N_FLAG))
}

#[test]
fn when_asl_addr_mode_absolute_program_counter_inc_by_3() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::Absolute;
  let res = asl(&mut mem,cpu);
  assert_eq!(res.program_counter, 3)
}

#[test]
fn when_asl_addr_mode_absolute_cycles_inc_by_6() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::Absolute;
  let res = asl(&mut mem,cpu);
  assert_eq!(res.cycles_count, 6)
}

#[test]
fn when_asl_addr_mode_absolute_res_assigned_to_indicated_memory() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.program_counter = 0x3;
  mem.ram[0x4] = 0xFF;
  mem.ram[0x5] = 0x1F;
  mem.ram[0x1FFF] = 0b010;
  cpu.addressing_mode = AddressingMode::Absolute;
  asl(&mut mem,cpu);
  assert_eq!(mem.ram[0x1FFF], 0b100)
}

#[test]
fn when_asl_addr_mode_xabsolute_res_assigned_to_indicated_memory() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.program_counter = 0x3;
  cpu.x_index = 1;
  mem.ram[0x4] = 0xFE;
  mem.ram[0x5] = 0x1F;
  mem.ram[0x1FFF] = 0b010;
  cpu.addressing_mode = AddressingMode::XAbsolute;
  asl(&mut mem,cpu);
  assert_eq!(mem.ram[0x1FFF], 0b100)
}

#[test]
fn when_asl_addr_mode_zeropage_res_assigned_to_indicated_memory() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.program_counter = 0x3;
  mem.ram[0x4] = 0xFF;
  mem.ram[0xFF] = 0b010;
  cpu.addressing_mode = AddressingMode::ZeroPage;
  asl(&mut mem,cpu);
  assert_eq!(mem.ram[0xFF], 0b100)
}

#[test]
fn when_asl_addr_mode_xzeropage_res_assigned_to_indicated_memory() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.program_counter = 0x3;
  cpu.x_index = 1;
  mem.ram[0x4] = 0xFE;
  mem.ram[0xFF] = 0b010;
  cpu.addressing_mode = AddressingMode::XZeroPage;
  asl(&mut mem,cpu);
  assert_eq!(mem.ram[0xFF], 0b100)
}
