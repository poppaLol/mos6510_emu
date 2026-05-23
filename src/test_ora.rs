use super::{get_cpu, C64Memory, ora, AddressingMode, Flags};

#[test]
fn when_ora_addr_mode_immediate_cycles_inc_by_2() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::Immediate;
  let res = ora(mem,cpu);
  assert_eq!(res.1.cycles_count, 2)
}

#[test]
fn when_ora_addr_mode_immediate_program_counter_inc_by_2() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::Immediate;
  let res = ora(mem,cpu);
  assert_eq!(res.1.program_counter, 2)
}

#[test]
fn when_ora_addr_mode_xindirect_cycles_inc_by_6() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::XIndirect;
  let res = ora(mem,cpu);
  assert_eq!(res.1.cycles_count, 6)
}

#[test]
fn when_ora_addr_mode_xindirect_program_counter_inc_by_2() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::XIndirect;
  let res = ora(mem,cpu);
  assert_eq!(res.1.program_counter, 2)
}

#[test]
fn when_ora_addr_mode_yindirect_cycles_inc_by_5() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::YIndirect;
  let res = ora(mem,cpu);
  assert_eq!(res.1.cycles_count, 5)
}

#[test]
fn when_ora_addr_mode_yindirect_program_counter_inc_by_2() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::YIndirect;
  let res = ora(mem,cpu);
  assert_eq!(res.1.program_counter, 2)
}

#[test]
fn when_ora_addr_mode_absolute_cycles_inc_by_4() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::Absolute;
  let res = ora(mem,cpu);
  assert_eq!(res.1.cycles_count, 4)
}

#[test]
fn when_ora_addr_mode_absolute_program_counter_inc_by_3() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::Absolute;
  let res = ora(mem,cpu);
  assert_eq!(res.1.program_counter, 3)
}

#[test]
fn when_ora_addr_mode_xabsolute_cycles_inc_by_4() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::XAbsolute;
  let res = ora(mem,cpu);
  assert_eq!(res.1.cycles_count, 4)
}

#[test]
fn when_ora_addr_mode_xabsolute_and_page_boundary_crossed_cycles_inc_by_5() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::XAbsolute;
  cpu.program_counter = 0x3; //pc at 3 - about to read block 4 and 5 of mem
  mem.ram[4] = 0xFF;
  mem.ram[5] = 0x0F; // block 4 and 5 of mem are 0x0FFF (within the page)
  cpu.x_index = 1; //x-register 1 (hence adding 1 to the address to be read, crossing page boundary
  let res = ora(mem,cpu);
  assert_eq!(res.1.cycles_count, 5)
}

#[test]
fn when_ora_addr_mode_xabsolute_program_counter_inc_by_3() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::XAbsolute;
  let res = ora(mem,cpu);
  assert_eq!(res.1.program_counter, 3)
}

#[test]
fn when_ora_addr_mode_yabsolute_cycles_inc_by_4() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::YAbsolute;
  let res = ora(mem,cpu);
  assert_eq!(res.1.cycles_count, 4)
}

#[test]
fn when_ora_addr_mode_yabsolute_and_page_boundary_crossed_cycles_inc_by_5() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.program_counter = 0x3; //pc at 3 - about to read word at block 4 and 5 of mem
  cpu.addressing_mode = AddressingMode::YAbsolute;
  mem.ram[4] = 0xFF;
  mem.ram[5] = 0x0F; // block 4 and 5 of mem are 0x0FFF (within the page)
  cpu.y_index = 1; //x-register 1 (hence adding 1 to the address to be read, crossing page boundary
  let res = ora(mem,cpu);
  assert_eq!(res.1.cycles_count, 5)
}

#[test]
fn when_ora_addr_mode_yabsolute_program_counter_inc_by_3() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::YAbsolute;
  let res = ora(mem,cpu);
  assert_eq!(res.1.program_counter, 3)
}

#[test]
fn when_ora_addr_mode_zeropage_cycles_inc_by_3() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::ZeroPage;
  let res = ora(mem,cpu);
  assert_eq!(res.1.cycles_count, 3)
}

#[test]
fn when_ora_addr_mode_zeropage_program_counter_inc_by_2() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::ZeroPage;
  let res = ora(mem,cpu);
  assert_eq!(res.1.program_counter, 2)
}

#[test]
fn when_ora_addr_mode_xzeropage_cycles_inc_by_4() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::XZeroPage;
  let res = ora(mem,cpu);
  assert_eq!(res.1.cycles_count, 4)
}

#[test]
fn when_ora_addr_mode_xzeropage_program_counter_inc_by_2() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::XZeroPage;
  let res = ora(mem,cpu);
  assert_eq!(res.1.program_counter, 2)
}

#[test]
fn when_ora_or_operation_accumulator_updated() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::YIndirect;
  mem.ram[3] = 2;
  cpu.program_counter = 2;
  cpu.accumulator = 3;
  let res = ora(mem,cpu);
  assert_eq!(res.1.accumulator, 3)
}

#[test]
fn when_ora_result_negative_n_flag_set() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::Immediate;
  mem.ram[3] = 0xF0;
  cpu.program_counter = 2;
  cpu.accumulator = 0x0F;
  let res = ora(mem,cpu);
  assert!(res.1.processor_status.contains(Flags::N_FLAG))
}

#[test]
fn when_ora_result_not_negative_n_flag_cleared() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::Immediate;
  mem.ram[3] = 0x00;
  cpu.program_counter = 2;
  cpu.accumulator = 0x0F;
  let res = ora(mem,cpu);
  assert!(!res.1.processor_status.contains(Flags::N_FLAG))
}

#[test]
fn when_ora_result_zero_z_flag_set() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::Immediate;
  mem.ram[3] = 0x00;
  cpu.program_counter = 2;
  cpu.accumulator = 0x00;
  let res = ora(mem,cpu);
  assert!(res.1.processor_status.contains(Flags::Z_FLAG))
}

#[test]
fn when_ora_result_not_zero_z_flag_set() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::Immediate;
  mem.ram[3] = 0x01;
  cpu.program_counter = 2;
  cpu.accumulator = 0x02;
  let res = ora(mem,cpu);
  assert!(!res.1.processor_status.contains(Flags::Z_FLAG))
}
