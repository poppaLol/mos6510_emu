use super::{get_cpu, C64Memory, sbc, AddressingMode, Flags};

#[test]
fn when_sbc_addr_mode_immediate_subs_next_address_byte_from_accumulator() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.processor_status = Flags::ALWAYS | Flags::C_FLAG;
  cpu.program_counter = 0x3;
  cpu.accumulator = 3;
  cpu.addressing_mode = AddressingMode::Immediate;
  mem.ram[4] = 1;
  let res = sbc(mem,cpu);
  assert_eq!(res.1.accumulator, 2)
}

#[test]
fn when_sbc_addr_mode_if_not_carry_flag_subs_extra_1() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.program_counter = 0x3;
  cpu.accumulator = 3;
  cpu.addressing_mode = AddressingMode::Immediate;
  mem.ram[4] = 1;
  let res = sbc(mem,cpu);
  assert_eq!(res.1.accumulator, 1)
}

#[test]
fn when_sbc_addr_mode_immediate_clear_carry_flag() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.processor_status = Flags::ALWAYS | Flags::C_FLAG;
  cpu.program_counter = 0x3;
  cpu.accumulator = 3;
  cpu.addressing_mode = AddressingMode::Immediate;
  mem.ram[4] = 4;
  let res = sbc(mem,cpu);
  assert!(!res.1.processor_status.contains(Flags::C_FLAG));
  assert_eq!(res.1.accumulator, 0xFF)
}

#[test]
fn when_sbc_addr_mode_immediate_carry_flag_left() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.processor_status = Flags::ALWAYS | Flags::C_FLAG;
  cpu.program_counter = 0x3;
  cpu.accumulator = 4;
  cpu.addressing_mode = AddressingMode::Immediate;
  mem.ram[4] = 4;
  let res = sbc(mem,cpu);
  assert!(res.1.processor_status.contains(Flags::C_FLAG));
}


#[test]
fn when_sbc_addr_mode_absolute_subs_address_byte_to_accumulator() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.processor_status = Flags::ALWAYS | Flags::C_FLAG;
  cpu.program_counter = 0x3;
  cpu.accumulator = 3;
  cpu.addressing_mode = AddressingMode::Absolute;
  mem.ram[4] = 0xFF;
  mem.ram[5] = 0x03;
  mem.ram[0x03FF] = 1;
  let res = sbc(mem,cpu);
  assert_eq!(res.1.accumulator, 2)
}

#[test]
fn when_sbc_addr_mode_xabsolute_subs_address_byte_to_accumulator() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.processor_status = Flags::ALWAYS | Flags::C_FLAG;
  cpu.program_counter = 0x3;
  cpu.accumulator = 3;
  cpu.addressing_mode = AddressingMode::XAbsolute;
  cpu.x_index = 1;
  mem.ram[4] = 0xFE;
  mem.ram[5] = 0x03;
  mem.ram[0x03FF] = 1;
  let res = sbc(mem,cpu);
  assert_eq!(res.1.accumulator, 2)
}

#[test]
fn when_sbc_addr_mode_yabsolute_subs_address_byte_to_accumulator() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.processor_status = Flags::ALWAYS | Flags::C_FLAG;
  cpu.program_counter = 0x3;
  cpu.accumulator = 3;
  cpu.addressing_mode = AddressingMode::YAbsolute;
  cpu.y_index = 1;
  mem.ram[4] = 0xFE;
  mem.ram[5] = 0x03;
  mem.ram[0x03FF] = 1;
  let res = sbc(mem,cpu);
  assert_eq!(res.1.accumulator, 2)
}

#[test]
fn when_sbc_addr_mode_zeropage_subs_next_byte_to_accumulator() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.processor_status = Flags::ALWAYS | Flags::C_FLAG;
  cpu.program_counter = 0x3;
  cpu.accumulator = 3;
  cpu.addressing_mode = AddressingMode::ZeroPage;
  mem.ram[4] = 0x24;
  mem.ram[0x24] = 1;
  let res = sbc(mem,cpu);
  assert_eq!(res.1.accumulator, 2)
}

#[test]
fn when_sbc_addr_mode_xzeropage_subs_offset_byte_to_accumulator() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.processor_status = Flags::ALWAYS | Flags::C_FLAG;
  cpu.program_counter = 0x3;
  cpu.accumulator = 3;
  cpu.addressing_mode = AddressingMode::XZeroPage;
  cpu.x_index = 0x49;
  mem.ram[4] = 0xA0;
  mem.ram[0x49 + 0xA0] = 1;
  let res = sbc(mem,cpu);
  assert_eq!(res.1.accumulator, 2)
}

#[test]
fn when_sbc_addr_mode_yindirect_subs_offset_byte_to_accumulator() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.processor_status = Flags::ALWAYS | Flags::C_FLAG;
  cpu.program_counter = 0x3;
  cpu.accumulator = 3;
  cpu.addressing_mode = AddressingMode::YIndirect;
  cpu.y_index = 1;
  mem.ram[4] = 0x86;
  mem.ram[0x86] = 0xFE;
  mem.ram[0x87] = 0x2F;
  mem.ram[0x2FFF] = 1;
  let res = sbc(mem,cpu);
  assert_eq!(res.1.accumulator, 2)
}

#[test]
fn when_sbc_addr_mode_xindirect_subs_offset_byte_to_accumulator() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.processor_status = Flags::ALWAYS | Flags::C_FLAG;
  cpu.program_counter = 0x3;
  cpu.accumulator = 3;
  cpu.addressing_mode = AddressingMode::XIndirect;
  cpu.x_index = 0x4;
  mem.ram[4] = 0x20;
  mem.ram[0x24] = 0xFF;
  mem.ram[0x25] = 0x02;
  mem.ram[0x02FF] = 1;
  let res = sbc(mem,cpu);
  assert_eq!(res.1.accumulator, 2)
}

#[test]
fn when_sbc_addr_mode_immediate_cycles_inc_by_2() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::Immediate;
  let res = sbc(mem,cpu);
  assert_eq!(res.1.cycles_count, 2)
}

#[test]
fn when_sbc_addr_mode_immediate_program_counter_inc_by_2() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::Immediate;
  let res = sbc(mem,cpu);
  assert_eq!(res.1.program_counter, 2)
}

#[test]
fn when_sbc_addr_mode_absolute_cycles_inc_by_4() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::Absolute;
  let res = sbc(mem,cpu);
  assert_eq!(res.1.cycles_count, 4)
}

#[test]
fn when_sbc_addr_mode_absolute_program_counter_inc_by_3() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::Absolute;
  let res = sbc(mem,cpu);
  assert_eq!(res.1.program_counter, 3)
}

#[test]
fn when_sbc_addr_mode_xabsolute_cycles_inc_by_4() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::XAbsolute;
  let res = sbc(mem,cpu);
  assert_eq!(res.1.cycles_count, 4)
}

#[test]
fn when_sbc_addr_mode_xabsolute_and_page_boundary_crossed_cycles_inc_by_5() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.program_counter = 0x3; //pc at 3 - about to read block 4 and 5 of mem
  cpu.addressing_mode = AddressingMode::XAbsolute;
  mem.ram[4] = 0xFF;
  mem.ram[5] = 0x0F; // block 4 and 5 of mem are 0x0FFF (within the page)
  cpu.x_index = 1; //x-register 1 (hence adding 1 to the address to be read, crossing page boundary
  let res = sbc(mem,cpu);
  assert_eq!(res.1.cycles_count, 5)
}

#[test]
fn when_sbc_addr_mode_xabsolute_program_counter_inc_by_3() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::XAbsolute;
  let res = sbc(mem,cpu);
  assert_eq!(res.1.program_counter, 3)
}

#[test]
fn when_sbc_addr_mode_yabsolute_cycles_inc_by_4() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::YAbsolute;
  let res = sbc(mem,cpu);
  assert_eq!(res.1.cycles_count, 4)
}

#[test]
fn when_sbc_addr_mode_yabsolute_and_page_boundary_crossed_cycles_inc_by_5() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.program_counter = 0x3; //pc at 3 - about to read block 4 and 5 of mem
  cpu.addressing_mode = AddressingMode::YAbsolute;
  mem.ram[4] = 0xFF;
  mem.ram[5] = 0x0F; // block 4 and 5 of mem are 0x0FFF (within the page)
  cpu.y_index = 1; //x-register 1 (hence adding 1 to the address to be read, crossing page boundary
  let res = sbc(mem,cpu);
  assert_eq!(res.1.cycles_count, 5)
}

#[test]
fn when_sbc_addr_mode_yabsolute_program_counter_inc_by_3() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::YAbsolute;
  let res = sbc(mem,cpu);
  assert_eq!(res.1.program_counter, 3)
}

#[test]
fn when_sbc_addr_mode_zeropage_cycles_inc_by_3() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::ZeroPage;
  let res = sbc(mem,cpu);
  assert_eq!(res.1.cycles_count, 3)
}

#[test]
fn when_sbc_addr_mode_zeropage_program_counter_inc_by_2() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::ZeroPage;
  let res = sbc(mem,cpu);
  assert_eq!(res.1.program_counter, 2)
}

#[test]
fn when_sbc_addr_mode_xzeropage_cycles_inc_by_4() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::XZeroPage;
  let res = sbc(mem,cpu);
  assert_eq!(res.1.cycles_count, 4)
}

#[test]
fn when_sbc_addr_mode_xzeropage_program_counter_inc_by_2() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::XZeroPage;
  let res = sbc(mem,cpu);
  assert_eq!(res.1.program_counter, 2)
}

#[test]
fn when_adc_addr_mode_xzeropage_and_page_boundary_crossed_cycles_inc_by_4() {
  //this is apparently different to adc - in that it doesnt add any extra
  //cycles for XZeroPage by crossing a page boundary?
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.program_counter = 0x2FFD;
  cpu.accumulator = 1;
  cpu.addressing_mode = AddressingMode::XZeroPage;
  cpu.x_index = 0xFF;
  mem.ram[0x2FFE] = 0xFF;
  mem.ram[0xFFE + 0xFF] = 1;
  let res = sbc(mem,cpu);
  assert_eq!(res.1.cycles_count, 4)
}

#[test]
fn when_sbc_addr_mode_xindirect_cycles_inc_by_6() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::XIndirect;
  let res = sbc(mem,cpu);
  assert_eq!(res.1.cycles_count, 6)
}

#[test]
fn when_sbc_addr_mode_xindirect_program_counter_inc_by_2() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::XIndirect;
  let res = sbc(mem,cpu);
  assert_eq!(res.1.program_counter, 2)
}

#[test]
fn when_sbc_addr_mode_yindirect_and_page_boundary_crossed_cycles_inc_by_6() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.program_counter = 0x3; //pc at 3 - about to read block 4 and 5 of mem
  cpu.addressing_mode = AddressingMode::YIndirect;
  cpu.y_index = 1; //y-register 1 offsetting to read at 5 and 6
  mem.ram[4] = 0x86;
  mem.ram[0x86] = 0xFE;
  mem.ram[0x87] = 0x2F;
  mem.ram[0x2FFF] = 1; // final address 0x2FFF (beyond base page address)
  let res = sbc(mem,cpu);
  assert_eq!(res.1.cycles_count, 6)
}

#[test]
fn when_sbc_addr_mode_yindirect_cycles_inc_by_5() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::YIndirect;
  let res = sbc(mem,cpu);
  assert_eq!(res.1.cycles_count, 5)
}

#[test]
fn when_sbc_addr_mode_yindirect_program_counter_inc_by_2() {
  let mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::YIndirect;
  let res = sbc(mem,cpu);
  assert_eq!(res.1.program_counter, 2)
}
