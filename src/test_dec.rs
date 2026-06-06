use super::{get_cpu, C64Memory, dec, dex, dey, AddressingMode, Flags};

#[test]
fn when_dec_mode_absolute_decrements_address_value_by_1() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.program_counter = 0x3;
  cpu.addressing_mode = AddressingMode::Absolute;
  mem.ram[4] = 0xFF;
  mem.ram[5] = 0x03;
  mem.ram[0x03FF] = 1;
  let res = dec(&mut mem,cpu);
  assert!(res.processor_status.contains(Flags::Z_FLAG));
  assert_eq!(mem.ram[0x03FF], 0)
}

#[test]
fn when_addr_contains_zero_decrements_address_value_by_1() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.program_counter = 0x3;
  cpu.addressing_mode = AddressingMode::Absolute;
  mem.ram[4] = 0xFF;
  mem.ram[5] = 0x03;
  mem.ram[0x03FF] = 0;
  let res = dec(&mut mem,cpu);
  assert!(res.processor_status.contains(Flags::N_FLAG));
  assert_eq!(mem.ram[0x03FF], 0xFF)
}

#[test]
fn when_dec_mode_xabsolute_decrements_address_value_by_1() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.program_counter = 0x3;
  cpu.addressing_mode = AddressingMode::XAbsolute;
  cpu.x_index = 1;
  mem.ram[4] = 0xFE;
  mem.ram[5] = 0x03;
  mem.ram[0x03FF] = 1;
  dec(&mut mem,cpu);
  assert_eq!(mem.ram[0x03FF], 0)
}


#[test]
fn when_dec_mode_zeropage_decrements_address_value_by_1() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.program_counter = 0x3;
  cpu.addressing_mode = AddressingMode::ZeroPage;
  mem.ram[4] = 0x24;
  mem.ram[0x24] = 1;
  dec(&mut mem,cpu);
  assert_eq!(mem.ram[0x24], 0)
}

#[test]
fn when_dec_mode_xzeropage_decrements_address_value_by_1() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.program_counter = 0x3;
  cpu.addressing_mode = AddressingMode::XZeroPage;
  cpu.x_index = 0x49;
  mem.ram[4] = 0xA0;
  mem.ram[0x49 + 0xA0] = 1;
  dec(&mut mem,cpu);
  assert_eq!(mem.ram[0x49 + 0xA0], 0)
}

#[test]
fn when_dex_decrements_x_reg_value_by_1() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.program_counter = 0x3;
  cpu.x_index = 0x1;

  let res = dex(&mut mem,cpu);
  assert_eq!(res.x_index, 0)
}

#[test]
fn when_dey_decrements_y_reg_value_by_1() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.program_counter = 0x3;
  cpu.y_index = 0x1;

  let res = dey(&mut mem,cpu);
  assert_eq!(res.y_index, 0)
}
