use registers::Registers;
use registers::RegisterFlags::{C,H,N,Z};

pub struct CPU {
    pub pc: u16,
    pub sp: u16,
    pub registers: Registers
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            pc: 0,
            sp: 0,
            registers: Registers::new()
        }
    }

    pub fn alu_add(&mut self, a: u8, b: u8) -> u8 {
        let result = (a as u16) + (b as u16);
        self.registers.set_flag(Z, result as u8 == 0);
        self.registers.set_flag(N, false);
        self.registers.set_flag(H, (a & 0x0F) + (b & 0x0F) > 0x0F);
        self.registers.set_flag(C, result > 0xFF);
        result as u8
    }

    pub fn alu_adc(&mut self, a: u8, b: u8) -> u8 {
        let carry = self.registers.get_flag(C) as u8;
        let result = self.alu_add(a, b+carry);
        if b == 255 {
            self.registers.set_flag(C, true);
        }
        if (b & 0x0F) == 0x0F {
            self.registers.set_flag(H, true);
        }
        result
    }

    pub fn alu_sub(&mut self, a: u8, b: u8) -> u8 {
        let result = a - b;
        self.registers.set_flag(Z, result == 0);
        self.registers.set_flag(N, true);
        self.registers.set_flag(H, ((a & 0x0F) as i16 - (b & 0x0F) as i16) >= 0);
        self.registers.set_flag(C, (a as i16) - (b as i16) >= 0);
        result
    }

    pub fn alu_sbc(&mut self, a: u8, b: u8) -> u8 {
        let carry = self.registers.get_flag(C) as u8;
        let result = self.alu_sub(a, b+carry);
        if b == 0 {
            self.registers.set_flag(C, true);
            self.registers.set_flag(H, true);
        }
        result
    }
}