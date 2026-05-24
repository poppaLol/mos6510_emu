use std::collections::VecDeque;

use super::{
    feed_typed_input, get_cpu, parse_typed_input, C64Memory, KEYBOARD_BUFFER_COUNT,
    KEYBOARD_BUFFER_START,
};

#[test]
fn typed_input_translates_newlines_to_return_and_uppercases_text() {
    let input = parse_typed_input("print \"hello\"\\nrun\\n");
    let bytes: Vec<u8> = input.into_iter().collect();

    assert_eq!(bytes, b"PRINT \"HELLO\"\rRUN\r");
}

#[test]
fn feed_typed_input_puts_one_byte_in_kernal_keyboard_buffer_at_wait_loop() {
    let mut mem = C64Memory::get_empty_mem();
    let mut cpu = get_cpu();
    let mut input = VecDeque::from(vec![b'A', b'B']);
    cpu.program_counter = 0xE5CD;

    assert!(feed_typed_input(&mut mem, &cpu, &mut input));
    assert_eq!(mem.ram[KEYBOARD_BUFFER_COUNT], 1);
    assert_eq!(mem.ram[KEYBOARD_BUFFER_START], b'A');
    assert_eq!(input, VecDeque::from(vec![b'B']));
}

#[test]
fn feed_typed_input_waits_when_keyboard_buffer_is_not_empty() {
    let mut mem = C64Memory::get_empty_mem();
    let mut cpu = get_cpu();
    let mut input = VecDeque::from(vec![b'A']);
    cpu.program_counter = 0xE5CD;
    mem.ram[KEYBOARD_BUFFER_COUNT] = 1;

    assert!(!feed_typed_input(&mut mem, &cpu, &mut input));
    assert_eq!(input, VecDeque::from(vec![b'A']));
}
