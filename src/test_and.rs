use super::{get_cpu, C64Memory, and, anc, AddressingMode, Flags};


#[test]
fn when_and_addr_mode_immediate_cycles_inc_by_2() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::Immediate;
  let res = and(mem,cpu);
  assert_eq!(res.1.cycles_count, 2)
}

#[test]
fn when_and_addr_mode_immediate_program_counter_inc_by_2() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::Immediate;
  let res = and(mem,cpu);
  assert_eq!(res.1.program_counter, 2)
}

#[test]
fn when_and_addr_mode_xindirect_cycles_inc_by_6() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::XIndirect;
  let res = and(mem,cpu);
  assert_eq!(res.1.cycles_count, 6)
}

#[test]
fn when_and_addr_mode_xindirect_program_counter_inc_by_2() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::XIndirect;
  let res = and(mem,cpu);
  assert_eq!(res.1.program_counter, 2)
}

#[test]
fn when_and_addr_mode_yindirect_cycles_inc_by_5() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::YIndirect;
  let res = and(mem,cpu);
  assert_eq!(res.1.cycles_count, 5)
}

#[test]
fn when_and_addr_mode_yindirect_program_counter_inc_by_2() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::YIndirect;
  let res = and(mem,cpu);
  assert_eq!(res.1.program_counter, 2)
}

#[test]
fn when_and_addr_mode_absolute_cycles_inc_by_4() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::Absolute;
  let res = and(mem,cpu);
  assert_eq!(res.1.cycles_count, 4)
}

#[test]
fn when_and_addr_mode_absolute_program_counter_inc_by_3() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::Absolute;
  let res = and(mem,cpu);
  assert_eq!(res.1.program_counter, 3)
}

#[test]
fn when_and_addr_mode_xabsolute_cycles_inc_by_4() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::XAbsolute;
  let res = and(mem,cpu);
  assert_eq!(res.1.cycles_count, 4)
}

#[test]
fn when_and_addr_mode_xabsolute_and_page_boundary_crossed_cycles_inc_by_5() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.program_counter = 0x3; //pc at 3 - about to read block 4 and 5 of mem
  cpu.addressing_mode = AddressingMode::XAbsolute;
  mem.ram[4] = 0xFF;
  mem.ram[5] = 0x0F; // block 4 and 5 of mem are 0x0FFF (within the page)
  cpu.x_index = 1; //x-register 1 (hence adding 1 to the address to be read, crossing page boundary
  let res = and(mem,cpu);
  assert_eq!(res.1.cycles_count, 5)
}

#[test]
fn when_and_addr_mode_xabsolute_program_counter_inc_by_3() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::XAbsolute;
  let res = and(mem,cpu);
  assert_eq!(res.1.program_counter, 3)
}

#[test]
fn when_and_addr_mode_yabsolute_cycles_inc_by_4() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::YAbsolute;
  let res = and(mem,cpu);
  assert_eq!(res.1.cycles_count, 4)
}

#[test]
fn when_and_addr_mode_yabsolute_and_page_boundary_crossed_cycles_inc_by_5() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.program_counter = 0x3; //pc at 3 - about to read word at block 4 and 5 of mem
  cpu.addressing_mode = AddressingMode::YAbsolute;
  mem.ram[4] = 0xFF;
  mem.ram[5] = 0x0F; // block 4 and 5 of mem are 0x0FFF (within the page)
  cpu.y_index = 1; //x-register 1 (hence adding 1 to the address to be read, crossing page boundary
  let res = and(mem,cpu);
  assert_eq!(res.1.cycles_count, 5)
}

#[test]
fn when_and_addr_mode_yabsolute_program_counter_inc_by_3() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::YAbsolute;
  let res = and(mem,cpu);
  assert_eq!(res.1.program_counter, 3)
}

#[test]
fn when_and_addr_mode_zeropage_cycles_inc_by_3() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::ZeroPage;
  let res = and(mem,cpu);
  assert_eq!(res.1.cycles_count, 3)
}

#[test]
fn when_and_addr_mode_zeropage_program_counter_inc_by_2() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::ZeroPage;
  let res = and(mem,cpu);
  assert_eq!(res.1.program_counter, 2)
}

#[test]
fn when_and_addr_mode_xzeropage_cycles_inc_by_4() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::XZeroPage;
  let res = and(mem,cpu);
  assert_eq!(res.1.cycles_count, 4)
}

#[test]
fn when_and_addr_mode_xzeropage_program_counter_inc_by_2() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::XZeroPage;
  let res = and(mem,cpu);
  assert_eq!(res.1.program_counter, 2)
}

#[test]
fn when_and_operation_accumulator_updated() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::Immediate;
  mem.ram[3] = 2;
  cpu.program_counter = 2;
  cpu.accumulator = 3;
  let res = and(mem,cpu);
  assert_eq!(res.1.accumulator, 2)
}

#[test]
fn when_anc_operation_neg_c_flag_set_per_n_flag() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  mem.ram[3] = 0xFA;
  cpu.program_counter = 2;
  cpu.accumulator = 0xFF;
  cpu.addressing_mode = AddressingMode::Immediate;
  let res = anc(mem,cpu);
  assert_eq!(res.1.accumulator, mem.ram[3]);
  assert!(res.1.processor_status.contains(Flags::C_FLAG));
  assert!(res.1.processor_status.contains(Flags::N_FLAG))
}

#[test]
fn when_anc_operation_pos_c_flag_set_per_n_flag() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  mem.ram[3] = 0x0A;
  cpu.program_counter = 2;
  cpu.accumulator = 0x0F;
  cpu.addressing_mode = AddressingMode::Immediate;
  let res = anc(mem,cpu);
  assert_eq!(res.1.accumulator, mem.ram[3]);
  assert!(!res.1.processor_status.contains(Flags::C_FLAG));
  assert!(!res.1.processor_status.contains(Flags::N_FLAG))
}