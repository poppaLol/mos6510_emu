use super::{bmi, get_cpu, C64Memory, Flags, AddressingMode};

#[test]
fn when_bmi_no_branch_taken_cycles_increment_by_2() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::Relative;
  cpu.processor_status = Flags::ALWAYS; //i.e. neg flag not set
  cpu.program_counter = 0x3;
  mem.ram[0x4] = 0x9;
  let res = bmi(mem, cpu);
  assert_eq!(res.1.program_counter, 0x5); //with no branch goes to next byte instruction
  assert_eq!(res.1.cycles_count, 2)
}


#[test]
fn when_bmi_branch_taken_cycles_increment_by_3() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::Relative;
  cpu.processor_status = Flags::N_FLAG; //i.e. neg flag set
  cpu.program_counter = 0x3;
  mem.ram[0x4] = 0x9;
  let res = bmi(mem, cpu);
  assert_eq!(res.1.program_counter, 0xE); //jump is taken - adds 9 to PC plus 2 from reading the instructions
  assert_eq!(res.1.cycles_count, 3)
}

#[test]
fn when_bmi_branch_taken_is_negative_program_counter_updated_correctly() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::Relative;
  cpu.processor_status = Flags::N_FLAG; //i.e. neg flag set
  cpu.program_counter = 0x301;
  mem.ram[0x302] = 0xFB; //should be -5
  let res = bmi(mem, cpu);
  assert_eq!(res.1.program_counter, 0x2FE) //jump is taken - minus 5 to PC plus 2 from reading the instructions
}

#[test]
fn when_bmi_branch_taken_across_page_boundary_cycles_inc_by_4() {
  let mut mem = C64Memory::get_empty_mem();
  let mut cpu = get_cpu();
  cpu.addressing_mode = AddressingMode::Relative;
  cpu.processor_status = Flags::N_FLAG; //i.e. neg flag set
  cpu.program_counter = 0xF001;
  mem.ram[0xF002] = 0xFB; //should be -5 - taking us to 0xEFFC
  let res = bmi(mem, cpu);
  assert_eq!(res.1.program_counter, 0xEFFE); //jump is taken - minus 5 to PC plus 2 from reading the instructions
  assert_eq!(res.1.cycles_count, 4) //jump is taken - adds extra to cycles
}

