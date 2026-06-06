use std::collections::VecDeque;

use super::{
    feed_typed_input, get_cpu, map_live_key, parse_typed_input, C64Memory, LiveKey,
    KEYBOARD_BUFFER_COUNT, KEYBOARD_BUFFER_START, PETSCII_DELETE, PETSCII_RETURN,
    terminal_line_endings,
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

#[test]
fn live_keyboard_accepts_uppercase_alphanumeric_and_space() {
    assert_eq!(map_live_key(b'a'), Some(LiveKey::Input(b'A')));
    assert_eq!(map_live_key(b'Z'), Some(LiveKey::Input(b'Z')));
    assert_eq!(map_live_key(b'7'), Some(LiveKey::Input(b'7')));
    assert_eq!(map_live_key(b' '), Some(LiveKey::Input(b' ')));
    assert_eq!(map_live_key(b'"'), Some(LiveKey::Input(b'"')));
    assert_eq!(map_live_key(b'='), Some(LiveKey::Input(b'=')));
    assert_eq!(map_live_key(b'+'), Some(LiveKey::Input(b'+')));
    assert_eq!(map_live_key(b'-'), Some(LiveKey::Input(b'-')));
    assert_eq!(map_live_key(b'*'), Some(LiveKey::Input(b'*')));
    assert_eq!(map_live_key(b'/'), Some(LiveKey::Input(b'/')));
    assert_eq!(map_live_key(b';'), Some(LiveKey::Input(b';')));
    assert_eq!(map_live_key(b':'), Some(LiveKey::Input(b':')));
    assert_eq!(map_live_key(b'>'), Some(LiveKey::Input(b'>')));
}

#[test]
fn live_keyboard_maps_terminal_control_keys_to_petscii() {
    assert_eq!(map_live_key(b'\n'), Some(LiveKey::Input(PETSCII_RETURN)));
    assert_eq!(map_live_key(0x7F), Some(LiveKey::Input(PETSCII_DELETE)));
    assert_eq!(map_live_key(0x1B), Some(LiveKey::Stop));
    assert_eq!(map_live_key(0x03), Some(LiveKey::Quit));
}

#[test]
fn live_keyboard_ignores_unsupported_punctuation() {
    assert_eq!(map_live_key(b'!'), None);
}

#[test]
fn live_terminal_uses_carriage_return_line_feed_in_raw_mode() {
    assert_eq!(terminal_line_endings("ONE\nTWO"), "ONE\r\nTWO");
}
