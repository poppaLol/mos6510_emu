use super::{AddressingMode, Flags, get_mode};

#[test]
fn status_flags_use_6502_bit_layout() {
  assert_eq!(Flags::C_FLAG.bits(), 0x01);
  assert_eq!(Flags::Z_FLAG.bits(), 0x02);
  assert_eq!(Flags::I_FLAG.bits(), 0x04);
  assert_eq!(Flags::D_FLAG.bits(), 0x08);
  assert_eq!(Flags::B_FLAG.bits(), 0x10);
  assert_eq!(Flags::ALWAYS.bits(), 0x20);
  assert_eq!(Flags::V_FLAG.bits(), 0x40);
  assert_eq!(Flags::N_FLAG.bits(), 0x80);
}

#[test]
fn all_are_implied() {
  let op_codes = [ 0xCA,  0x88,  0xE8,  0xC8,  0x0A,  0x2A,  0x4A,  0x6A,  0xAA,  
      0x8A,  0xA8,  0x98,  0xBA,  0x9A,  0x68,  0x48,  0x28,  0x08,  0x00,  0x40,  
      0x60,  0x18,  0x38,  0xD8,  0xF8,  0x58,  0x78,  0xB8,  0xEA ];
  for op_code in op_codes.iter() {
    //println!("{:?}, {:#02x}", get_mode(op_code), op_code);
    assert_eq!(get_mode(op_code), AddressingMode::Implied)
  }
}

#[test]
fn all_are_immediate() {
  let op_codes = [
      0x09, 0x29, 0x49, 0x69, 0xE9, 0xC9, 0xE0, 0xC0, 0xA9, 0xA2, 0xA0,
  ];
  for op_code in op_codes.iter() {
    //println!("{:?}, {:#02x}", get_mode(op_code), op_code);
    assert_eq!(get_mode(op_code), AddressingMode::Immediate)
  }
}

#[test]
fn all_are_zero_page() {
  let op_codes = [
      0x05, 0x25, 0x45, 0x65, 0xE5, 0xC5, 0xE4, 0xC4, 0xC6, 0xE6, 0x06, 
      0x26, 0x46, 0x66, 0xA5, 0x85, 0xA6, 0x86, 0xA4, 0x84, 0x24,
  ];
  for op_code in op_codes.iter() {
    //println!("{:?}, {:#02x}", get_mode(op_code), op_code);
    assert_eq!(get_mode(op_code), AddressingMode::ZeroPage)
  }
}

#[test]
fn all_are_xzero_page() {
  let op_codes = [
      0x15, 0x35, 0x55, 0x75, 0xF5, 0xD5, 0xD6, 0xF6, 0x16, 0x36, 0x56,
      0x76, 0xB5, 0x95, 0xB4, 0x94 ];
  for op_code in op_codes.iter() {
    //println!("{:?}, {:#02x}", get_mode(op_code), op_code);
    assert_eq!(get_mode(op_code), AddressingMode::XZeroPage)
  }
}


#[test]
fn all_are_yzero_page() {
  let op_codes = [ 0xB6, 0x96 ];
  for op_code in op_codes.iter() {
    //println!("{:?}, {:#02x}", get_mode(op_code), op_code);
    assert_eq!(get_mode(op_code), AddressingMode::YZeroPage)
  }
}

#[test]
fn all_are_xindirect() {
  let op_codes = [ 0x01, 0x21, 0x41, 0x61, 0xE1, 0xC1, 0xA1, 0x81 ];
  for op_code in op_codes.iter() {
    //println!("{:?}, {:#02x}", get_mode(op_code), op_code);
    assert_eq!(get_mode(op_code), AddressingMode::XIndirect)
  }
}

#[test]
fn all_are_yindirect() {
  let op_codes = [ 0x11, 0x31, 0x51, 0x71, 0xF1, 0xD1, 0xB1, 0x91 ];
  for op_code in op_codes.iter() {
    //println!("{:?}, {:#02x}", get_mode(op_code), op_code);
    assert_eq!(get_mode(op_code), AddressingMode::YIndirect)
  }
}

#[test]
fn all_are_absolute() {
  let op_codes = [0x0D, 0x2D, 0x4D, 0x6D, 0xED, 0xCD, 0xEC, 0xCC, 0xCE, 0xEE, 0x0E, 0x2E,
      0x4E, 0x6E, 0xAD, 0x8D, 0xAE, 0x8E, 0xAC, 0x8C, 0x20, 0x4C, 0x2C];
  for op_code in op_codes.iter() {
    //println!("{:?}, {:#02x}", get_mode(op_code), op_code);
    assert_eq!(get_mode(op_code), AddressingMode::Absolute)
  }
}

#[test]
fn all_are_xabsolute() {
  let op_codes = [0x1D, 0x3D, 0x5D, 0x7D, 0xFD, 0xDD, 0xDE, 0xFE, 0x1E,
      0x3E, 0x5E, 0x7E, 0xBD, 0x9D, 0xBC];
  for op_code in op_codes.iter() {
    //println!("{:?}, {:#02x}", get_mode(op_code), op_code);
    assert_eq!(get_mode(op_code), AddressingMode::XAbsolute)
  }
}

#[test]
fn all_are_yabsolute() {
  let op_codes = [0x19, 0x39, 0x59, 0x79, 0xF9, 0xD9, 0xB9, 0x99, 0xBE];
  for op_code in op_codes.iter() {
    //println!("{:?}, {:#02x}", get_mode(op_code), op_code);
    assert_eq!(get_mode(op_code), AddressingMode::YAbsolute)
  }
}

#[test]
fn all_are_indirect() {
  let op_codes = [ 0x6C ];
  for op_code in op_codes.iter() {
    //println!("{:?}, {:#02x}", get_mode(op_code), op_code);
    assert_eq!(get_mode(op_code), AddressingMode::Indirect)
  }
}

#[test]
fn all_are_relative() {
  let op_codes = [0x10, 0x30, 0x50, 0x70, 0x90, 0xB0, 0xD0, 0xF0];
  for op_code in op_codes.iter() {
    //println!("{:?}, {:#02x}", get_mode(op_code), op_code);
    assert_eq!(get_mode(op_code), AddressingMode::Relative)
  }
}
