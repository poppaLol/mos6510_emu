use super::{get_cpu, C64Memory, cpx, cpy, AddressingMode, Flags};


//immediate
#[test]
fn when_cpx_given_x_reg_and_target_addr_same_zero_flag_set() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.processor_status = Flags::ALWAYS;
  cpu.program_counter = 0x3;
  cpu.addressing_mode = AddressingMode::Immediate;
  
  cpu.x_index = 3;
  mem.ram[0x4] = 3;

  let res = cpx(mem,cpu);
  assert!(!res.1.processor_status.contains(Flags::N_FLAG));
  assert!(res.1.processor_status.contains(Flags::ALWAYS | Flags::C_FLAG | Flags::Z_FLAG))
}

#[test]
fn when_cpx_given_x_reg_gt_target_addr_flags_not_set() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.processor_status = Flags::ALWAYS;
  cpu.program_counter = 0x3;
  cpu.addressing_mode = AddressingMode::Immediate;
  
  cpu.x_index = 4;
  mem.ram[0x4] = 3;

  let res = cpx(mem,cpu);
  assert!(!res.1.processor_status.contains(Flags::N_FLAG | Flags::Z_FLAG));
  assert!(res.1.processor_status.contains(Flags::ALWAYS | Flags::C_FLAG))
}

#[test]
fn when_cpx_given_x_reg_lt_target_addr_flags_not_set() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.processor_status = Flags::ALWAYS;
  cpu.program_counter = 0x3;
  cpu.addressing_mode = AddressingMode::Immediate;
  
  cpu.x_index = 2;
  mem.ram[0x4] = 3;

  let res = cpx(mem,cpu);
  assert!(!res.1.processor_status.contains(Flags::C_FLAG | Flags::Z_FLAG));
  assert!(res.1.processor_status.contains(Flags::ALWAYS | Flags::N_FLAG))
}

#[test]
fn when_cpy_given_y_reg_and_target_addr_same_zero_flag_set() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.processor_status = Flags::ALWAYS;
  cpu.program_counter = 0x3;
  cpu.addressing_mode = AddressingMode::Immediate;
  
  cpu.y_index = 3;
  mem.ram[0x4] = 3;

  let res = cpy(mem,cpu);
  assert!(res.1.processor_status.contains(Flags::ALWAYS | Flags::C_FLAG | Flags::Z_FLAG))
}


#[test]
fn when_cpy_given_y_reg_gt_target_addr_flags_not_set() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.processor_status = Flags::ALWAYS;
  cpu.program_counter = 0x3;
  cpu.addressing_mode = AddressingMode::Immediate;
  
  cpu.y_index = 4;
  mem.ram[0x4] = 3;

  let res = cpy(mem,cpu);
  assert!(!res.1.processor_status.contains(Flags::N_FLAG | Flags::Z_FLAG));
  assert!(res.1.processor_status.contains(Flags::ALWAYS | Flags::C_FLAG))
}

#[test]
fn when_cpy_given_y_reg_lt_target_addr_flags_not_set() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.processor_status = Flags::ALWAYS;
  cpu.program_counter = 0x3;
  cpu.addressing_mode = AddressingMode::Immediate;
  
  cpu.y_index = 2;
  mem.ram[0x4] = 3;

  let res = cpy(mem,cpu);
  assert!(!res.1.processor_status.contains(Flags::C_FLAG | Flags::Z_FLAG));
  assert!(res.1.processor_status.contains(Flags::ALWAYS | Flags::N_FLAG))
}

#[test]
fn when_cpx_addr_mode_absolute_compares_address_byte_to_x_reg() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.program_counter = 0x3;
  cpu.addressing_mode = AddressingMode::Absolute;
  
  cpu.x_index = 3;
  mem.ram[4] = 0xFF;
  mem.ram[5] = 0x03;
  mem.ram[0x03FF] = 3;

  let res = cpx(mem,cpu);
  assert!(res.1.processor_status.contains(Flags::Z_FLAG))
}

#[test]
fn when_cpy_addr_mode_absolute_compares_address_byte_to_y_reg() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.program_counter = 0x3;
  cpu.addressing_mode = AddressingMode::Absolute;
  
  cpu.y_index = 3;
  mem.ram[4] = 0xFF;
  mem.ram[5] = 0x03;
  mem.ram[0x03FF] = 3;

  let res = cpy(mem,cpu);
  assert!(res.1.processor_status.contains(Flags::Z_FLAG))
}


#[test]
fn when_cpx_addr_mode_zeropage_compares_address_byte_to_x_reg() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.program_counter = 0x3;
  cpu.addressing_mode = AddressingMode::ZeroPage;
  
  cpu.x_index = 3;
  mem.ram[4] = 0x24;
  mem.ram[0x24] = 3;
  
  let res = cpx(mem,cpu);
  assert!(res.1.processor_status.contains(Flags::Z_FLAG))
}

#[test]
fn when_cpx_addr_mode_zeropage_compares_address_byte_to_y_reg() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.program_counter = 0x3;
  cpu.addressing_mode = AddressingMode::ZeroPage;
  
  cpu.y_index = 3;
  mem.ram[4] = 0x24;
  mem.ram[0x24] = 3;
  
  let res = cpy(mem,cpu);
  assert!(res.1.processor_status.contains(Flags::Z_FLAG))
}
