use super::{ Flags, AddressingMode };

#[derive(Debug, Copy, Clone)]
pub struct Mos6510 {
  pub addressing_mode: AddressingMode,
  pub program_counter: u16,
  pub stack_pointer: u8,
  pub processor_status: Flags,
  pub accumulator: u8,
  pub x_index: u8,
  pub y_index: u8,
  pub cycles_count: usize
}

impl Mos6510 {
    pub fn boot_up() -> Mos6510 {
        Mos6510 {
            addressing_mode: AddressingMode::Implied,
            //The program_counter should be initialized to the word read from memory location 0xFFFC.
            //This will read from the KERNAL ROM due to the latch bits initialization.
            program_counter: 0x0,
            stack_pointer: 0x0,
            processor_status: Flags::ALWAYS,
            accumulator: 0x0,
            x_index: 0x0,
            y_index: 0x0,
            cycles_count: 0
        }
    }
}

pub struct ProcDelta {
    pub am: Option<AddressingMode>,
    pub pc: Option<u16>,
    pub sp: Option<u8>,
    pub status_on: Flags,
    pub status_off: Flags,
    pub acc: Option<u8>,
    pub x: Option<u8>,
    pub y: Option<u8>,
    pub cc: usize
}

impl ProcDelta {
    pub fn apply_proc_delta(&self, cpu: Mos6510) -> Mos6510 {
        Mos6510 {
            addressing_mode:    self.am.unwrap_or(cpu.addressing_mode),
            program_counter:    self.pc.unwrap_or(cpu.program_counter),
            stack_pointer:      self.sp.unwrap_or(cpu.stack_pointer),
            processor_status:   ((cpu.processor_status | self.status_on)
                                    & self.status_off),
            accumulator:        self.acc.unwrap_or(cpu.accumulator),
            x_index:            self.x.unwrap_or(cpu.x_index),
            y_index:            self.y.unwrap_or(cpu.y_index),
            cycles_count:       cpu.cycles_count + self.cc
        }
    }
    pub fn empty() -> ProcDelta {
        ProcDelta {
            am: None, pc: None, sp:None, status_on:Flags::ALWAYS, status_off:Flags::all(), acc:None, x:None, y:None, cc:0
        }
    }
    pub fn with_address_mode(mut self, mode:AddressingMode) -> ProcDelta {
        self.am = Some(mode);
        self
    }
    pub fn with_program_counter(mut self, pc:u16) -> ProcDelta {
        self.pc = Some(pc);
        self
    }
    pub fn with_cycles_count(mut self, cc:usize) -> ProcDelta {
        self.cc = cc;
        self
    }
    pub fn with_stack_pointer(mut self, sp:u8) -> ProcDelta {
        self.sp = Some(sp);
        self
    }
}
