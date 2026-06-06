use crate::memory::C64Memory;

pub const SCREEN_WIDTH: usize = 40;
pub const SCREEN_HEIGHT: usize = 25;
pub const SCREEN_RAM_START: usize = 0x0400;

pub fn render_text_screen(memory: &C64Memory) -> String {
    let mut output = String::new();

    output.push_str("+----------------------------------------+\n");
    for row in 0..SCREEN_HEIGHT {
        output.push('|');
        for col in 0..SCREEN_WIDTH {
            let index = SCREEN_RAM_START + row * SCREEN_WIDTH + col;
            output.push(screen_code_to_ascii(memory.ram[index]));
        }
        output.push_str("|\n");
    }
    output.push_str("+----------------------------------------+");

    output
}

pub fn screen_code_to_ascii(code: u8) -> char {
    match code {
        0x00 => '@',
        0x01..=0x1A => (b'A' + code - 1) as char,
        0x20 => ' ',
        0x21..=0x3F => code as char,
        0x40 => '@',
        0x41..=0x5A => code as char,
        0x5B => '[',
        0x5D => ']',
        0x5F => '_',
        0x60 => '-',
        0x61..=0x7A => (b'A' + code - 0x61) as char,
        0xA0 => '█',
        _ => '.',
    }
}

#[cfg(test)]
mod tests {
    use super::{render_text_screen, screen_code_to_ascii, SCREEN_HEIGHT, SCREEN_WIDTH};
    use crate::memory::C64Memory;

    #[test]
    fn renders_standard_text_screen_dimensions() {
        let mem = C64Memory::get_empty_mem();
        let rendered = render_text_screen(&mem);
        let lines: Vec<&str> = rendered.lines().collect();

        assert_eq!(lines.len(), SCREEN_HEIGHT + 2);
        assert_eq!(lines[0].len(), SCREEN_WIDTH + 2);
        assert_eq!(lines[1].len(), SCREEN_WIDTH + 2);
    }

    #[test]
    fn renders_screen_ram_as_ascii() {
        let mut mem = C64Memory::get_empty_mem();
        mem.ram[0x0400] = 0x08;
        mem.ram[0x0401] = 0x09;
        mem.ram[0x0402] = 0x21;

        let rendered = render_text_screen(&mem);

        assert!(rendered.contains("|HI!"));
    }

    #[test]
    fn renders_inverse_space_as_cursor_block() {
        assert_eq!(screen_code_to_ascii(0xA0), '█');
    }
}
