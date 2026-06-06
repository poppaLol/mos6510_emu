use std::fs::File;
use std::io::Read;
use crate::cia::Cia;
use crate::vic::VicII;

const BASIC_ROM_OFFSET: u16 = 0xA000;
const CHAR_ROM_OFFSET: u16 = 0xD000;
const KERN_ROM_OFFSET: u16 = 0xE000;

#[derive(Debug)]
pub struct C64Memory {
    pub ram: [u8; 0x10000],
    pub basic_rom: [u8; 0x2000],
    pub char_rom: [u8; 0x1000],
    pub kernel_rom: [u8; 0x2000],
    pub vic: VicII,
    pub cia1: Cia,
    pub cia2: Cia,
    pub trace: bool
}


impl C64Memory {
  pub fn init_memory(rom_file_path: &str, char_file_path: &str) -> C64Memory {
    let mut _romfile=File::open(rom_file_path).unwrap();
    let mut b_rombuffer = [0; 0x2000];
    _romfile.read_exact(&mut b_rombuffer).unwrap();
    
    let mut k_rombuffer = [0; 0x2000];
    _romfile.read_exact(&mut k_rombuffer).unwrap();
  
    assert!(k_rombuffer[(0xEA0E - KERN_ROM_OFFSET) as usize] == 0x88);
  
    let mut _charfile=File::open(char_file_path).unwrap();
    let mut charbuffer = [0; 0x1000];
    _charfile.read_exact(&mut charbuffer).unwrap();
  
    let mut _ram_memory: [u8; 0x10000] = [0; 0x10000];
    _ram_memory[0] = 0x2F; //io bits default 0b00101111 read
    _ram_memory[1] = 0x37; //latch bits default 0b00110111 
  
    C64Memory {
        ram: _ram_memory,
        basic_rom: b_rombuffer,
        char_rom: charbuffer,
        kernel_rom: k_rombuffer,
        vic: VicII::new(),
        cia1: Cia::new_cia1(),
        cia2: Cia::new_cia2(),
        trace: false
    }
  }

  #[allow(dead_code)]
  pub fn get_empty_mem() -> C64Memory {
      C64Memory{
          ram: [0; 0x10000],
          basic_rom: [0; 0x2000],
          char_rom: [0; 0x1000],
          kernel_rom: [0; 0x2000],
          vic: VicII::new(),
          cia1: Cia::new_cia1(),
          cia2: Cia::new_cia2(),
          trace: false
      }
  }


  fn read_ram_byte(&self, address: u16) -> u8 {
    if self.trace { print!("{:#02x} ", self.ram[address as usize]); }
    self.ram[address as usize]
  }
  
  fn read_ram_word(&self, address: u16) -> u16 {
    (self.read_ram_byte(address + 1) as u16) << 8 | 
      self.read_ram_byte(address) as u16
  }
  
  fn read_basic_rom_byte(&self, address: u16) -> u8 {
    if self.trace { print!("{:#02x} ", self.basic_rom[address as usize]); }
    self.basic_rom[address as usize]
  }
  
  fn read_basic_rom_word(&self, address: u16) -> u16 {
    (self.read_basic_rom_byte(address + 1) as u16) << 8 | 
      self.read_basic_rom_byte(address) as u16
  }
  
  fn read_char_rom_byte(&self, address: u16) -> u8 {
    if self.trace { print!("{:#02x} ", self.char_rom[address as usize]); }
    self.char_rom[address as usize]
  }
  
  fn read_char_rom_word(&self, address: u16) -> u16 {
    (self.read_char_rom_byte(address + 1) as u16) << 8 | 
      self.read_char_rom_byte(address) as u16
  }
  
  fn read_kern_rom_byte(&self, address: u16) -> u8 {
    if self.trace { print!("{:#02x} ", self.kernel_rom[address as usize]); }
    self.kernel_rom[address as usize]
  }
  
  fn read_kern_rom_word(&self, address: u16) -> u16 {
    (self.read_kern_rom_byte(address + 1) as u16) << 8 | 
      self.read_kern_rom_byte(address) as u16
  }

  fn read_io_byte(&self, address: u16) -> u8 {
    match address {
      0xD000..=0xD3FF => self.vic.read_byte(address),
      0xDC00..=0xDCFF => self.cia1.read_byte(address),
      0xDD00..=0xDDFF => self.cia2.read_byte(address),
      _ => self.read_ram_byte(address)
    }
  }

  fn read_io_word(&self, address: u16) -> u16 {
    (self.read_io_byte(address + 1) as u16) << 8 |
      self.read_io_byte(address) as u16
  }
  
  pub fn read_word(&self, address: u16) -> u16 {
    let latch_bits = self.ram[1];
    //no cart memory coded yet
    match address {
      0..=0x7FFF |
        0x8000..=0x9FFF |  // 0x8000..=0x9FFF or cart_lo todo (peek(address + 1) << 8) | peek(address)
        0xC000..=0xCFFF => self.read_ram_word(address),
      0xA000..=0xBFFF => if latch_bits & 0b1 > 0 { 
          self.read_basic_rom_word(address - BASIC_ROM_OFFSET) 
        } else { 
          self.read_ram_word(address) 
        },
      0xD000..=0xDFFF => if latch_bits & 0b100 > 0 {
          self.read_io_word(address)
        } else {
          self.read_char_rom_word(address - CHAR_ROM_OFFSET)
        }, //or cart_hi todo
      0xE000..=0xFFFF => if latch_bits & 0b10 > 0 { 
        self.read_kern_rom_word(address - KERN_ROM_OFFSET)
        } else {
          self.read_ram_word(address)
        } //or cart_hi todo
    }
  }
  
  pub fn read_byte(&self, address: u16) -> u8 {
    let latch_bits = self.ram[1];
    //no cart memory coded yet
    match address {
      0..=0x7FFF |
        0x8000..=0x9FFF |  // 0x8000..=0x9FFF or cart_lo todo (peek(address + 1) << 8) | peek(address)
        0xC000..=0xCFFF => self.read_ram_byte(address),
      0xA000..=0xBFFF => if latch_bits & 0b1 > 0 { 
          self.read_basic_rom_byte(address - BASIC_ROM_OFFSET) 
        } else { 
          self.read_ram_byte(address) 
        },
      0xD000..=0xDFFF => if latch_bits & 0b100 > 0 {
        self.read_io_byte(address)
        } else { 
          self.read_char_rom_byte(address - CHAR_ROM_OFFSET)
        }, //or cart_hi todo
      0xE000..=0xFFFF => if latch_bits & 0b10 > 0 { 
        self.read_kern_rom_byte(address - KERN_ROM_OFFSET)
        } else {
          self.read_ram_byte(address)
        } //or cart_hi todo
    }
  }

  pub fn write_byte(&mut self, pointer: &usize, value: u8) {
    if self.trace { print!(" ------  write ({:#02x}) at: {:#04x}", value, pointer); }
    match *pointer {
      0xD000..=0xD3FF => self.vic.write_byte(*pointer as u16, value),
      0xDC00..=0xDCFF => self.cia1.write_byte(*pointer as u16, value),
      0xDD00..=0xDDFF => self.cia2.write_byte(*pointer as u16, value),
      _ => self.ram[*pointer] = value
    }
  }

  pub fn tick(&mut self, cycles: usize) {
    self.cia1.tick(cycles);
    self.cia2.tick(cycles);
  }

  pub fn irq_pending(&self) -> bool {
    self.cia1.irq_pending()
  }

  pub fn acknowledge_irq(&mut self) {
    self.cia1.acknowledge_irq();
  }

  
  pub fn stack_push_word(&mut self, ptr: usize, val: u16) {
    //stack is 0x100 to 0x1FF - and pointer is u8 so have to add 0x100 to it
    let offset_ptr = ptr + 0x100;
    self.write_byte(&offset_ptr, (val >> 8) as u8);
    let low_ptr = 0x100 + (ptr as u8).wrapping_sub(1) as usize;
    self.write_byte(&low_ptr, val as u8);
  }


  pub fn stack_push_byte(&mut self, ptr: usize, val: u8) {
    //stack is 0x100 to 0x1FF - and pointer is u8 so have to add 0x100 to it
    let offset_ptr = ptr + 0x100;
    self.write_byte(&offset_ptr, val as u8);
  }


  pub fn stack_pop_word(&mut self, ptr: u16) -> u16 {
    let low_ptr = 0x100 + (ptr as u8).wrapping_add(1) as u16;
    let high_ptr = 0x100 + (ptr as u8).wrapping_add(2) as u16;
    let result = (self.read_byte(high_ptr) as u16) << 8 | self.read_byte(low_ptr) as u16;
    self.write_byte(&(low_ptr as usize), 0u8);
    self.write_byte(&(high_ptr as usize), 0u8);
    result
  }

  pub fn stack_pop_byte(&mut self, ptr: u16) -> u8 {
    let offset_ptr = 0x100 + (ptr as u8).wrapping_add(1) as u16;
    let result = self.read_byte(offset_ptr);
    self.write_byte(&(offset_ptr as usize), 0u8);
    result
  }
}

#[cfg(test)]
mod tests {
  use super::C64Memory;
  // Note this useful idiom: importing names from outer (for mod tests) scope.

  #[test]
  fn memory_regions_have_expected_sizes() {
    let mem = C64Memory::get_empty_mem();

    assert_eq!(mem.ram.len(), 0x10000);
    assert_eq!(mem.basic_rom.len(), 0x2000);
    assert_eq!(mem.char_rom.len(), 0x1000);
    assert_eq!(mem.kernel_rom.len(), 0x2000);
  }

  #[test]
  fn can_write_to_last_ram_address() {
    let mut mem = C64Memory::get_empty_mem();
    mem.write_byte(&0xFFFF, 0xAA);

    assert_eq!(mem.ram[0xFFFF], 0xAA)
  }

  #[test]
  fn rom_reads_map_first_and_last_bytes() {
    let mut mem = C64Memory::get_empty_mem();
    mem.ram[1] = 0x33;
    mem.basic_rom[0] = 0xA0;
    mem.basic_rom[0x1FFF] = 0xBF;
    mem.char_rom[0] = 0xD0;
    mem.char_rom[0x0FFF] = 0xDF;
    mem.kernel_rom[0] = 0xE0;
    mem.kernel_rom[0x1FFF] = 0xFF;

    assert_eq!(mem.read_byte(0xA000), 0xA0);
    assert_eq!(mem.read_byte(0xBFFF), 0xBF);
    assert_eq!(mem.read_byte(0xD000), 0xD0);
    assert_eq!(mem.read_byte(0xDFFF), 0xDF);
    assert_eq!(mem.read_byte(0xE000), 0xE0);
    assert_eq!(mem.read_byte(0xFFFF), 0xFF);
  }

  #[test]
  fn io_reads_expose_fake_vic_raster_when_charen_set() {
    let mut mem = C64Memory::get_empty_mem();
    mem.ram[1] = 0x37;
    mem.vic.write_byte(0xD012, 0x66);

    assert_eq!(mem.read_byte(0xD012), 0x00);
  }

  #[test]
  fn io_reads_expose_cia_registers_when_charen_set() {
    let mut mem = C64Memory::get_empty_mem();
    mem.ram[1] = 0x37;

    assert_eq!(mem.read_byte(0xDC00), 0xFF);
    assert_eq!(mem.read_byte(0xDD00), 0xFF);
  }

  #[test]
  fn io_writes_update_cia_registers_when_charen_set() {
    let mut mem = C64Memory::get_empty_mem();
    mem.ram[1] = 0x37;

    mem.write_byte(&0xDD02, 0xFF);
    mem.write_byte(&0xDD00, 0x7F);

    assert_eq!(mem.read_byte(0xDD00), 0x7F);
    assert_eq!(mem.ram[0xDD00], 0x00);
  }

  #[test]
  fn cia1_port_input_bits_read_high_when_data_direction_bits_are_clear() {
    let mut mem = C64Memory::get_empty_mem();
    mem.ram[1] = 0x37;

    mem.write_byte(&0xDC02, 0x3F);
    mem.write_byte(&0xDC00, 0x0F);

    assert_eq!(mem.read_byte(0xDC00), 0xCF);
  }

  #[test]
  fn cia2_iec_input_bits_follow_released_output_lines() {
    let mut mem = C64Memory::get_empty_mem();
    mem.ram[1] = 0x37;

    mem.write_byte(&0xDD02, 0x3F);
    mem.write_byte(&0xDD00, 0x00);

    assert_eq!(mem.read_byte(0xDD00), 0xC0);
  }

  #[test]
  fn cia2_iec_input_bits_follow_pulled_low_output_lines() {
    let mut mem = C64Memory::get_empty_mem();
    mem.ram[1] = 0x37;

    mem.write_byte(&0xDD02, 0x3F);
    mem.write_byte(&0xDD00, 0x30);

    assert_eq!(mem.read_byte(0xDD00), 0x30);
  }

  #[test]
  fn can_push_byte_to_stack() {
    let mut mem = C64Memory::get_empty_mem();
    mem.stack_push_byte(255, 1);

    assert_eq!(mem.ram[0x1FF], 1)
  }

  #[test]
  fn can_pop_byte_from_stack() {
    let mut mem = C64Memory::get_empty_mem();
    mem.stack_push_byte(255, 1);

    let val = mem.stack_pop_byte(254);
    assert_eq!(val, 1)
  }

  #[test]
  fn when_pop_byte_from_stack_mem_cleared() {
    let mut mem = C64Memory::get_empty_mem();
    mem.stack_push_byte(255, 1);

    mem.stack_pop_byte(254);
    assert_eq!(mem.ram[0x1FF], 0)
  }

  #[test]
  fn can_push_word_to_stack() {
    let mut mem = C64Memory::get_empty_mem();
    mem.stack_push_word(255, 258);

    assert_eq!(mem.ram[0x1FF], 1);
    assert_eq!(mem.ram[0x1FE], 2)
  }

  #[test]
  fn can_pop_word_from_stack() {
    let mut mem = C64Memory::get_empty_mem();
    mem.stack_push_word(255, 258);

    let val = mem.stack_pop_word(253);
    assert_eq!(val, 258)
  }
  
  #[test]
  fn when_pop_word_from_stack_mem_cleared() {
    let mut mem = C64Memory::get_empty_mem();
    mem.stack_push_word(255, 258);

    mem.stack_pop_word(253);
    assert_eq!(mem.ram[0x1FF], 0);
    assert_eq!(mem.ram[0x1FE], 0)
  }
}
