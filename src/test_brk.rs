use super::{brk, get_cpu, C64Memory};

#[test]
fn when_brk_cycles_count_inc_by_seven() {
  let mem = C64Memory::get_empty_mem();
  let cpu = get_cpu();
  let res = brk(mem, cpu);
  assert_eq!(res.1.cycles_count, 7)
}

#[test]
fn when_brk_program_counter_inc_by_two() {
  let mem = C64Memory::get_empty_mem();
  let cpu = get_cpu();
  let res = brk(mem, cpu);
  assert_eq!(res.1.program_counter, 2)
}

#[test]
fn when_brk_stack_pushed_with_flags() {
  let mem = C64Memory::get_empty_mem();
  let cpu = get_cpu();
  let res = brk(mem, cpu);
  let stored_flags = res.0.read_byte(0x1FD);
  assert_eq!(stored_flags, res.1.processor_status.bits())
}

#[test]
fn when_brk_stack_pushed_with_incremented_program_counter() {
  let mem = C64Memory::get_empty_mem();
  let cpu = get_cpu();
  let res = brk(mem, cpu);
  let stored_program_counter = res.0.read_word(0x1FE);
  assert_eq!(stored_program_counter, res.1.program_counter)
}
