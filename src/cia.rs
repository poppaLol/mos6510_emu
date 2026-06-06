const PORT_A: usize = 0x00;
const PORT_B: usize = 0x01;
const DATA_DIRECTION_A: usize = 0x02;
const DATA_DIRECTION_B: usize = 0x03;
const TIMER_A_LOW: usize = 0x04;
const TIMER_A_HIGH: usize = 0x05;
const INTERRUPT_CONTROL: usize = 0x0D;
const CONTROL_A: usize = 0x0E;

const CIA2_IEC_CLOCK_OUT: u8 = 0x10;
const CIA2_IEC_DATA_OUT: u8 = 0x20;
const CIA2_IEC_CLOCK_IN: u8 = 0x40;
const CIA2_IEC_DATA_IN: u8 = 0x80;

const INTERRUPT_TIMER_A: u8 = 0x01;
const CONTROL_START: u8 = 0x01;
const CONTROL_ONE_SHOT: u8 = 0x08;
const CONTROL_FORCE_LOAD: u8 = 0x10;

#[derive(Debug, Clone)]
pub struct Cia {
    registers: [u8; 0x10],
    port_a_inputs: u8,
    port_b_inputs: u8,
    cia2_iec_lines: bool,
    timer_a_counter: u16,
    timer_a_latch: u16,
    interrupt_mask: u8,
    interrupt_pending: Cell<u8>,
}

impl Cia {
    fn new(port_a_inputs: u8, port_b_inputs: u8, cia2_iec_lines: bool) -> Cia {
        Cia {
            registers: [0; 0x10],
            port_a_inputs,
            port_b_inputs,
            cia2_iec_lines,
            timer_a_counter: 0,
            timer_a_latch: 0,
            interrupt_mask: 0,
            interrupt_pending: Cell::new(0),
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
            TIMER_A_LOW => self.timer_a_counter as u8,
            TIMER_A_HIGH => (self.timer_a_counter >> 8) as u8,
            INTERRUPT_CONTROL => self.read_interrupt_control(),
            _ => self.registers[register],
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        let register = (address as usize) & 0x0F;
        match register {
            TIMER_A_LOW => {
                self.timer_a_latch = (self.timer_a_latch & 0xFF00) | value as u16;
                self.registers[register] = value;
            },
            TIMER_A_HIGH => {
                self.timer_a_latch = (self.timer_a_latch & 0x00FF) | ((value as u16) << 8);
                self.registers[register] = value;
                if self.registers[CONTROL_A] & CONTROL_START == 0 {
                    self.timer_a_counter = self.timer_a_latch;
                }
            },
            INTERRUPT_CONTROL => {
                if value & 0x80 != 0 {
                    self.interrupt_mask |= value & 0x1F;
                } else {
                    self.interrupt_mask &= !(value & 0x1F);
                }
            },
            CONTROL_A => {
                self.registers[register] = value & !CONTROL_FORCE_LOAD;
                if value & CONTROL_FORCE_LOAD != 0 {
                    self.timer_a_counter = self.timer_a_latch;
                }
            },
            _ => self.registers[register] = value,
        }
    }

    pub fn tick(&mut self, cycles: usize) {
        if self.registers[CONTROL_A] & CONTROL_START == 0 {
            return;
        }

        for _ in 0..cycles {
            if self.timer_a_counter == 0 {
                self.timer_a_underflow();
                if self.registers[CONTROL_A] & CONTROL_START == 0 {
                    return;
                }
            }

            if self.timer_a_counter == 0 {
                continue;
            }

            self.timer_a_counter = self.timer_a_counter.wrapping_sub(1);
            if self.timer_a_counter == 0 {
                self.timer_a_underflow();
                if self.registers[CONTROL_A] & CONTROL_START == 0 {
                    return;
                }
            }
        }
    }

    pub fn irq_pending(&self) -> bool {
        self.interrupt_pending.get() & self.interrupt_mask != 0
    }

    fn timer_a_underflow(&mut self) {
        self.interrupt_pending
            .set(self.interrupt_pending.get() | INTERRUPT_TIMER_A);
        self.timer_a_counter = self.timer_a_latch;
        if self.registers[CONTROL_A] & CONTROL_ONE_SHOT != 0 {
            self.registers[CONTROL_A] &= !CONTROL_START;
        }
    }

    fn read_interrupt_control(&self) -> u8 {
        let pending = self.interrupt_pending.replace(0);
        if pending & self.interrupt_mask != 0 {
            pending | 0x80
        } else {
            pending
        }
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

    #[test]
    fn timer_a_underflow_sets_masked_irq() {
        let mut cia = Cia::new_cia1();

        cia.write_byte(0xDC04, 0x02);
        cia.write_byte(0xDC05, 0x00);
        cia.write_byte(0xDC0D, 0x81);
        cia.write_byte(0xDC0E, 0x11);

        cia.tick(2);

        assert!(cia.irq_pending());
        assert_eq!(cia.read_byte(0xDC0D), 0x81);
        assert!(!cia.irq_pending());
    }

    #[test]
    fn timer_a_underflow_without_mask_does_not_raise_irq_line() {
        let mut cia = Cia::new_cia1();

        cia.write_byte(0xDC04, 0x01);
        cia.write_byte(0xDC05, 0x00);
        cia.write_byte(0xDC0E, 0x11);

        cia.tick(1);

        assert!(!cia.irq_pending());
        assert_eq!(cia.read_byte(0xDC0D), 0x01);
    }
}
use std::cell::Cell;
