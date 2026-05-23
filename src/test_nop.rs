use super::*;

#[test]
fn when_nop_cycles_count_inc_by_2() {
  let cpu = get_cpu();
  let res = nop(cpu);
  assert_eq!(res.cycles_count, 2)
}

#[test]
fn when_nop_program_counter_inc() {
  let cpu = get_cpu();
  let res = nop(cpu);
  assert_eq!(res.program_counter,1)
}