const PORT_A: usize = 0x00;
const PORT_B: usize = 0x01;
const DATA_DIRECTION_A: usize = 0x02;
const DATA_DIRECTION_B: usize = 0x03;

const CIA2_IEC_CLOCK_OUT: u8 = 0x10;
const CIA2_IEC_DATA_OUT: u8 = 0x20;
const CIA2_IEC_CLOCK_IN: u8 = 0x40;
const CIA2_IEC_DATA_IN: u8 = 0x80;

#[derive(Debug, Copy, Clone)]
pub struct Cia {
    registers: [u8; 0x10],
    port_a_inputs: u8,
    port_b_inputs: u8,
    cia2_iec_lines: bool,
}

impl Cia {
    fn new(port_a_inputs: u8, port_b_inputs: u8, cia2_iec_lines: bool) -> Cia {
        Cia {
            registers: [0; 0x10],
            port_a_inputs,
            port_b_inputs,
            cia2_iec_lines,
        }
    }

    pub fn new_cia1() -> Cia {
        Cia::new(0xFF, 0xFF, false)
    }

    pub fn new_cia2() -> Cia {
        Cia::new(0xFF, 0xFF, true)
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        let register = (address as usize) & 0x0F;
        match register {
            PORT_A => self.read_port(PORT_A, DATA_DIRECTION_A, self.port_a_input_pins()),
            PORT_B => self.read_port(PORT_B, DATA_DIRECTION_B, self.port_b_inputs),
            _ => self.registers[register],
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        let register = (address as usize) & 0x0F;
        self.registers[register] = value;
    }

    fn read_port(
        &self,
        port_register: usize,
        data_direction_register: usize,
        input_pins: u8,
    ) -> u8 {
        let output_mask = self.registers[data_direction_register];
        let input_mask = !output_mask;

        (self.registers[port_register] & output_mask) | (input_pins & input_mask)
    }

    fn port_a_input_pins(&self) -> u8 {
        if !self.cia2_iec_lines {
            return self.port_a_inputs;
        }

        let mut pins = self.port_a_inputs & !(CIA2_IEC_CLOCK_IN | CIA2_IEC_DATA_IN);

        if self.cia2_output_line_released(CIA2_IEC_CLOCK_OUT) {
            pins |= CIA2_IEC_CLOCK_IN;
        }

        if self.cia2_output_line_released(CIA2_IEC_DATA_OUT) {
            pins |= CIA2_IEC_DATA_IN;
        }

        pins
    }

    fn cia2_output_line_released(&self, output_mask: u8) -> bool {
        let data_direction = self.registers[DATA_DIRECTION_A];
        let port_a = self.registers[PORT_A];

        // CIA2 PA4/PA5 are inverted before the IEC bus: 0 releases the line,
        // 1 actively pulls it low.
        data_direction & output_mask == 0 || port_a & output_mask == 0
    }
}

#[cfg(test)]
mod tests {
    use super::Cia;

    #[test]
    fn port_reads_default_to_external_high_inputs() {
        let cia = Cia::new_cia1();

        assert_eq!(cia.read_byte(0xDC00), 0xFF);
        assert_eq!(cia.read_byte(0xDC01), 0xFF);
    }

    #[test]
    fn non_port_registers_default_to_zero() {
        let cia = Cia::new_cia1();

        assert_eq!(cia.read_byte(0xDC02), 0x00);
        assert_eq!(cia.read_byte(0xDC04), 0x00);
    }

    #[test]
    fn port_outputs_are_mixed_with_external_inputs_by_data_direction() {
        let mut cia = Cia::new_cia1();

        cia.write_byte(0xDC02, 0x0F);
        cia.write_byte(0xDC00, 0x05);

        assert_eq!(cia.read_byte(0xDC00), 0xF5);
    }

    #[test]
    fn cia2_released_iec_output_lines_read_high_on_input_pins() {
        let mut cia = Cia::new_cia2();

        cia.write_byte(0xDD02, 0x3F);
        cia.write_byte(0xDD00, 0x00);

        assert_eq!(cia.read_byte(0xDD00), 0xC0);
    }

    #[test]
    fn cia2_low_iec_output_lines_read_low_on_input_pins() {
        let mut cia = Cia::new_cia2();

        cia.write_byte(0xDD02, 0x3F);
        cia.write_byte(0xDD00, 0x30);

        assert_eq!(cia.read_byte(0xDD00), 0x30);
    }

    #[test]
    fn cia2_input_direction_releases_iec_lines() {
        let mut cia = Cia::new_cia2();

        cia.write_byte(0xDD02, 0x0F);
        cia.write_byte(0xDD00, 0x00);

        assert_eq!(cia.read_byte(0xDD00), 0xF0);
    }
}
