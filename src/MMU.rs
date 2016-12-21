// temporary simple MMU

pub struct MMU {
    pub memory: [u8; 0x10000]
}

impl MMU {
    pub fn new() -> MMU {
        MMU {
            memory: [0; 0x10000]
        }
    }

    pub fn read_byte(&mut self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    pub fn write_byte(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000 ... 0x7FFF => return,
            _ => self.memory[addr as usize] = val
        }
    }

    pub fn read_word(&mut self, addr: u16) -> u16 {
        ((self.read_byte(addr) as u16) << 8) & (self.read_byte(addr+1) as u16)
    }

    pub fn write_word(&mut self, addr: u16, val: u16) {
        self.write_byte(addr, (val & 0x00FF) as u8);
        self.write_byte(addr+1, (val >> 8) as u8);
    }
}