bitflags! {
  pub struct Flags: u8 {
    const N_FLAG = 0x1; //Negative
    const V_FLAG = 0x2; //oVerflow
    const ALWAYS = 0x4; //always on apparently
    const B_FLAG = 0x8; //Break (1 when interupt was caused by a BRK)
    const D_FLAG = 0x10; //Decimal (1 when CPU in BCD mode)
    const I_FLAG = 0x20; //IRQ (when 1, no interupts will occur (exceptions are IRQs forced by BRK and NMIs))
    const Z_FLAG = 0x40; //Zero (1 when all bits of a result are 0)
    const C_FLAG = 0x80; //Carry (1 on unsigned overflow)
  }
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(dead_code)]
pub enum AddressingMode {
  //A "no addressing mode at all"-option:
  //Instructions which do not address an arbitrary memory location only supports this mode.
  Implied,
  //supported by bit-shifting instructions, turns the "action" of the operation towards the accumulator.
  Immediate,
  //refers to the byte immediately following the opcode for the instruction.
  Relative,
  //which refers to a given 16-bit address
  ZeroPage,
  //Address at byte after instruction + X register
  XZeroPage,
  //Address at byte after instruction + Y register
  YZeroPage,
  //Address at word after instruction
  Absolute,
  //Address at word after instruction + X register
  XAbsolute,
  //Address at word after instruction + Y register
  YAbsolute,
  //Address at memory which is pointed to by word after instruction
  Indirect,
  //Address at memory which is pointed to by word after instruction + X register
  XIndirect,
  //Address at memory which is pointed to by word after instruction + Y register
  YIndirect,
}

impl AddressingMode {
  pub fn bytes_increment(&self) -> u16 {
    match self {
      AddressingMode::Implied => 1,
      AddressingMode::Absolute | AddressingMode::YAbsolute | AddressingMode::XAbsolute => 3,
      _ => 2,
    }
  }

  pub fn shift_cycles_increment(&self) -> usize {
    match self {
      AddressingMode::Implied => 2,
      AddressingMode::Absolute => 6,
      AddressingMode::XAbsolute => 7,
      AddressingMode::ZeroPage => 5,
      AddressingMode::XZeroPage => 6,
      _ => 0,
    }
  }

  pub fn cycles_increment(&self, extra: usize) -> usize {
    match self {
      AddressingMode::Implied => 1 + extra,
      AddressingMode::Relative => 2 + extra,
      AddressingMode::Immediate => 2,
      AddressingMode::Absolute | AddressingMode::XZeroPage => 4 + extra,
      AddressingMode::XAbsolute => 4 + extra,
      AddressingMode::YAbsolute => 4 + extra,
      AddressingMode::ZeroPage => 3,
      AddressingMode::XIndirect => 6,
      AddressingMode::YIndirect => 5 + extra,
      _ => 2,
    }
  }

  pub fn crossed_page_boundary(&self, base_address: u16, final_address: u16) -> usize {
    //this condition may need expansion later
    match self {
      AddressingMode::Relative | AddressingMode::XAbsolute | AddressingMode::YAbsolute => {
        match (base_address >> 12) == (final_address >> 12) {
          true => 0,
          false => 1,
        }
      }
      _ => 0,
    }
  }

  pub fn crossed_page_boundary_indy(&self, base_address: u16, final_address: u16) -> usize {
    //this condition may need expansion later
    match self {
      AddressingMode::XAbsolute | AddressingMode::YIndirect | AddressingMode::YAbsolute => {
        match (base_address >> 12) == (final_address >> 12) {
          true => 0,
          false => 1,
        }
      }
      _ => 0,
    }
  }

  pub fn crossed_page_boundary_xzero(&self, base_address: u16, final_address: u16) -> usize {
    //this condition may need expansion later
    match self {
      AddressingMode::XAbsolute
      | AddressingMode::YIndirect
      | AddressingMode::XZeroPage
      | AddressingMode::YAbsolute => match (base_address >> 12) == (final_address >> 12) {
        true => 0,
        false => 1,
      },
      _ => 0,
    }
  }
}

pub fn get_mode(op_code: &u8) -> AddressingMode {
  match op_code {
    0x1A | 0x3A | 0x5A | 0x7A | 0xDA | 0xEA | 0xFA | 0x00 | 0xCA | 0x88 | 0xE8 | 0xC8 | 0x60
    | 0x78 | 0x9A | 0xD8 | 0x0A | 0x2A | 0x4A | 0x6A | 0xAA | 0x8A | 0xA8 | 0x98 | 0xBA | 0x08
    | 0x68 | 0x48 | 0x28 | 0x18 | 0x38 | 0xF8 | 0x58 | 0xB8 | 0x40 => AddressingMode::Implied,
    0xC0 | 0xE0 | 0xC9 | 0xE9 | 0x69 | 0x09 | 0x29 | 0x0B | 0x2B | 0x49 | 0xA2 | 0xA9 | 0xA0 => {
      AddressingMode::Immediate
    }
    0xE6 | 0xC6 | 0xC5 | 0xE5 | 0x65 | 0x05 | 0x25 | 0x45 | 0x06 | 0x26 | 0x46 | 0x66 | 0xA5
    | 0xA6 | 0x85 | 0x86 | 0xA4 | 0x84 | 0xE4 | 0xC4 | 0x24 => AddressingMode::ZeroPage,
    0x16 | 0xF6 | 0xD6 | 0xD5 | 0xF5 | 0x75 | 0x15 | 0x35 | 0x55 | 0x36 | 0x56 | 0x76 | 0xB5
    | 0xB4 | 0x94 | 0x95 => AddressingMode::XZeroPage,
    0xB6 | 0x96 => AddressingMode::YZeroPage,
    0xC1 | 0xE1 | 0x61 | 0x01 | 0x21 | 0x41 | 0xA1 | 0x81 => AddressingMode::XIndirect,
    0xD1 | 0xF1 | 0x71 | 0x11 | 0x31 | 0x51 | 0xB1 | 0x91 => AddressingMode::YIndirect,
    0x0E | 0xEE | 0xCE | 0xCD | 0xED | 0x6D | 0x0D | 0x2D | 0x4D | 0x2E | 0x4E | 0x6E | 0x20
    | 0x8E | 0xAD | 0xAE | 0x8D | 0xAC | 0x8C | 0x4C | 0xEC | 0xCC | 0x2C => {
      AddressingMode::Absolute
    }
    0x1E | 0xFE | 0xDE | 0xDD | 0xFD | 0x7D | 0x1D | 0x3D | 0x5D | 0x3E | 0x5E | 0x7E | 0xBD
    | 0xBC | 0x9D => AddressingMode::XAbsolute,
    0xD9 | 0xF9 | 0x79 | 0x19 | 0x39 | 0x59 | 0xBE | 0x99 | 0xB9 => AddressingMode::YAbsolute,
    0x6C => AddressingMode::Indirect,
    0x10 | 0x30 | 0x50 | 0x70 | 0x90 | 0xB0 | 0xD0 | 0xF0 => AddressingMode::Relative,
    _ => std::panic::panic_any(format!("No proc addressing mode!! {:#02x}", op_code)),
  }
}
