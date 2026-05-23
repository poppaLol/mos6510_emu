use std::fs::File;
use std::io::Read;

const BASIC_ROM_OFFSET: u16 = 0xA000;
const CHAR_ROM_OFFSET: u16 = 0xD000;
const KERN_ROM_OFFSET: u16 = 0xE000;

#[derive(Debug, Copy, Clone)]
pub struct C64Memory {
    pub ram: [u8; 0x10000],
    pub basic_rom: [u8; 0x2000],
    pub char_rom: [u8; 0x1000],
    pub kernel_rom: [u8; 0x2000],
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
        trace: true
    }
  }

  #[allow(dead_code)]
  pub fn get_empty_mem() -> C64Memory {
      C64Memory{
          ram: [0; 0x10000],
          basic_rom: [0; 0x2000],
          char_rom: [0; 0x1000],
          kernel_rom: [0; 0x2000],
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
      0xD000..=0xDFFF => if latch_bits > 1 && latch_bits & 0b100 > 0 { 
          self.read_char_rom_word(address - CHAR_ROM_OFFSET) 
        } else { 
          self.read_ram_word(address)
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
      0xD000..=0xDFFF => if latch_bits > 1 && latch_bits & 0b100 > 0 { 
        self.read_char_rom_byte(address - CHAR_ROM_OFFSET) 
        } else { 
          self.read_ram_byte(address)
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
    self.ram[*pointer] = value;
  }

  
  pub fn stack_push_word(&mut self, ptr: usize, val: u16) {
    //stack is 0x100 to 0x1FF - and pointer is u8 so have to add 0x100 to it
    let offset_ptr = ptr + 0x100;
    self.write_byte(&offset_ptr, (val >> 8) as u8);
    self.write_byte(&(offset_ptr - 1), val as u8);
  }


  pub fn stack_push_byte(&mut self, ptr: usize, val: u8) {
    //stack is 0x100 to 0x1FF - and pointer is u8 so have to add 0x100 to it
    let offset_ptr = ptr + 0x100;
    self.write_byte(&offset_ptr, val as u8);
  }


  pub fn stack_pop_word(&mut self, ptr: u16) -> u16 {
    let offset_ptr = ptr + 1 + 0x100;
    let result = self.read_word(offset_ptr);
    self.write_byte(&(offset_ptr as usize), 0u8);
    self.write_byte(&((offset_ptr + 1) as usize), 0u8);
    result
  }

  pub fn stack_pop_byte(&mut self, ptr: u16) -> u8 {
    let offset_ptr = ptr + 1 + 0x100;
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
    mem.ram[1] = 0x37;
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
