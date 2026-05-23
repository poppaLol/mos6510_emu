#[derive(Debug, Copy, Clone)]
pub struct VicII {
    registers: [u8; 0x40],
}

impl VicII {
    pub fn new() -> VicII {
        VicII {
            registers: [0; 0x40],
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        let register = (address as usize) & 0x3F;
        match register {
            0x12 => 0x00,
            _ => self.registers[register],
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        let register = (address as usize) & 0x3F;
        self.registers[register] = value;
    }
}
