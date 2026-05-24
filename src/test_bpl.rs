use super::{bpl, get_cpu, C64Memory, Flags, AddressingMode};

#[test]
fn when_bpl_no_branch_taken_cycles_increment_by_2() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::Relative;
  cpu.processor_status = Flags::N_FLAG; //i.e. neg flag set
  cpu.program_counter = 0x3;
  mem.ram[0x4] = 0x9;
  let res = bpl(mem, cpu);
  assert_eq!(res.1.program_counter, 0x5); //with no branch goes to next byte instruction
  assert_eq!(res.1.cycles_count, 2)
}


#[test]
fn when_bpl_branch_taken_cycles_increment_by_3() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::Relative;
  cpu.processor_status = Flags::ALWAYS; //i.e. neg flag not set
  cpu.program_counter = 0x3;
  mem.ram[0x4] = 0x9;
  let res = bpl(mem, cpu);
  assert_eq!(res.1.program_counter, 0xE); //jump is taken - adds 9 to PC plus 2 from reading instructions
  assert_eq!(res.1.cycles_count, 3)
}

#[test]
fn when_bpl_branch_taken_is_negative_program_counter_updated_correctly() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::Relative;
  cpu.processor_status = Flags::ALWAYS; //i.e. neg flag not set
  cpu.program_counter = 0x301;
  mem.ram[0x302] = 0xFB; //should be -5
  let res = bpl(mem, cpu);
  assert_eq!(res.1.program_counter, 0x2FE) //jump is taken - minus 5 to PC plus 2 from reading instructions
}

#[test]
fn when_bpl_branch_taken_across_page_boundary_cycles_inc_by_4() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::Relative;
  cpu.processor_status = Flags::ALWAYS; //i.e. neg flag not set
  cpu.program_counter = 0xF001;
  mem.ram[0xF002] = 0xFB; //should be -5 - taking us to 0xEFFC
  let res = bpl(mem, cpu);
  assert_eq!(res.1.program_counter, 0xEFFE); //jump is taken - minus 5 to PC plus 2 from reading instructions
  assert_eq!(res.1.cycles_count, 4) //jump is taken - adds extra to cycles
}

#[test]
fn when_bpl_branch_target_crosses_from_next_instruction_cycles_inc_by_4() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::Relative;
  cpu.processor_status = Flags::ALWAYS; //i.e. neg flag not set
  cpu.program_counter = 0x10FD;
  mem.ram[0x10FE] = 0x01; // next instruction is 0x10FF, target is 0x1100
  let res = bpl(mem, cpu);
  assert_eq!(res.1.program_counter, 0x1100);
  assert_eq!(res.1.cycles_count, 4)
}

#[test]
fn when_bpl_branch_stays_on_page_from_next_instruction_cycles_inc_by_3() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::Relative;
  cpu.processor_status = Flags::ALWAYS; //i.e. neg flag not set
  cpu.program_counter = 0x10FE;
  mem.ram[0x10FF] = 0x00; // next instruction and target are both 0x1100
  let res = bpl(mem, cpu);
  assert_eq!(res.1.program_counter, 0x1100);
  assert_eq!(res.1.cycles_count, 3)
}
