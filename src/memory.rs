use std::fs::File;
use std::io::Read;

const BASIC_ROM_OFFSET: u16 = 0x9FFF;
const CHAR_ROM_OFFSET: u16 = 0xCFFF;
const KERN_ROM_OFFSET: u16 = 0xDFFF;

#[derive(Debug, Copy, Clone)]
pub struct C64Memory {
    pub ram: [u8; 0xFFFF],
    pub basic_rom: [u8; 0x1FFF],
    pub char_rom: [u8; 0x0FFF],
    pub kernel_rom: [u8; 0x1FFF]
}


impl C64Memory {
  pub fn init_memory(rom_file_path: &str, char_file_path: &str) -> C64Memory {
    let mut _romfile=File::open(rom_file_path).unwrap();
    let mut b_rombuffer = [0; 0x1FFF];
    _romfile.read(&mut b_rombuffer).unwrap();
    
    let mut k_rombuffer = [0; 0x1FFF];
    _romfile.read(&mut k_rombuffer).unwrap();
  
    assert!(k_rombuffer[(0xEA0E - KERN_ROM_OFFSET) as usize] == 0x88);
  
    let mut _charfile=File::open(char_file_path).unwrap();
    let mut charbuffer = [0; 0x0FFF];
    _charfile.read(&mut charbuffer).unwrap();
  
    let mut _ram_memory: [u8; 0xFFFF] = [0; 0xFFFF];
    _ram_memory[0] = 0x2F; //io bits default 0b00101111 read
    _ram_memory[1] = 0x37; //latch bits default 0b00110111 
  
    C64Memory {
        ram: _ram_memory,
        basic_rom: b_rombuffer,
        char_rom: charbuffer,
        kernel_rom: k_rombuffer
    }
  }

  #[allow(dead_code)]
  pub fn get_empty_mem() -> C64Memory {
      C64Memory{
          ram: [0; 0xFFFF],
          basic_rom: [0; 0x1FFF],
          char_rom: [0; 0x0FFF],
          kernel_rom: [0; 0x1FFF]
      }
  }


  fn read_ram_byte(&self, address: u16) -> u8 {
    print!("{:#02x} ", self.ram[address as usize]);
    self.ram[address as usize]
  }
  
  fn read_ram_word(&self, address: u16) -> u16 {
    (self.read_ram_byte(address + 1) as u16) << 8 | 
      self.read_ram_byte(address) as u16
  }
  
  fn read_basic_rom_byte(&self, address: u16) -> u8 {
    print!("{:#02x} ", self.basic_rom[address as usize]);
    self.basic_rom[address as usize]
  }
  
  fn read_basic_rom_word(&self, address: u16) -> u16 {
    (self.read_basic_rom_byte(address + 1) as u16) << 8 | 
      self.read_basic_rom_byte(address) as u16
  }
  
  fn read_char_rom_byte(&self, address: u16) -> u8 {
    print!("{:#02x} ", self.char_rom[address as usize]);
    self.char_rom[address as usize]
  }
  
  fn read_char_rom_word(&self, address: u16) -> u16 {
    (self.read_char_rom_byte(address + 1) as u16) << 8 | 
      self.read_char_rom_byte(address) as u16
  }
  
  fn read_kern_rom_byte(&self, address: u16) -> u8 {
    print!("{:#02x} ", self.kernel_rom[address as usize]);
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
    print!(" ------  write ({:#02x}) at: {:#04x}", value, pointer);
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