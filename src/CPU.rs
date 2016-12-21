use registers::Registers;
use registers::RegisterFlags::{C,H,N,Z};
use mmu::MMU;

pub struct CPU {
    pub pc: u16,
    pub sp: u16,
    pub registers: Registers,
    pub mmu: MMU
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            pc: 0,
            sp: 0,
            registers: Registers::new(),
            mmu: MMU::new()
        }
    }

    pub fn exec_opcode(&mut self, opcode: u8) -> u8 {
        let first = opcode >> 6;
        let second = (opcode >> 3) & 0b111;
        let third = opcode & 0b111;
        match first {
            0 => {
                match second {
                    0b110 => {
                        match third {
                            0b110 => { // 00 110 110 - LD (HL), n
                                let n = self.next_byte();
                                self.mmu.write_byte(self.registers.hl(), n);
                                3
                            }
                            _ => { // 00 110  r  - LD (HL), r
                                self.mmu.write_byte(self.registers.hl(), self.registers.get_reg(opcode & 0b111));
                                2
                            }
                        }
                    },
                    _ if third == 0b110 => { // 00  r  110 - LD r, n
                        let n = self.next_byte();
                        self.registers.set_reg(second, n);
                        2
                    },
                    _ if ((second & 1 == 0) && third == 0b001) => { // 00 dd0 001 - LD dd, nn
                        let reg = second >> 1;
                        let n = self.next_word();
                        if reg == 3 {
                            self.sp = self.next_word();
                        } else {
                            self.registers.set_reg16(reg, n);
                        }
                        3
                    },
                    _ => 0
                }
            },
            _ => 0
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

    pub fn alu_add16(&mut self, a: u16, b: u16) -> u16 {
        let result = a + b;
        self.registers.set_flag(N, false);
        self.registers.set_flag(H, (a & 0x0FFF) + (b & 0x0FFF) > 0x0FFF);
        self.registers.set_flag(C, (a as i32) + (b as i32) > 0xFFFF);
        result
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

    pub fn alu_and(&mut self, a: u8, b: u8) -> u8 {
        let result = a & b;
        self.registers.set_flag(Z, result == 0);
        self.registers.set_flag(C, false);
        self.registers.set_flag(H, true);
        self.registers.set_flag(N, false);
        result
    }

    pub fn alu_or(&mut self, a: u8, b: u8) -> u8 {
        let result = a | b;
        self.registers.set_flag(Z, result == 0);
        self.registers.set_flag(C, false);
        self.registers.set_flag(H, false);
        self.registers.set_flag(N, false);
        result
    }

    pub fn alu_xor(&mut self, a: u8, b: u8) -> u8 {
        let result = a ^ b;
        self.registers.set_flag(Z, result == 0);
        self.registers.set_flag(C, false);
        self.registers.set_flag(H, false);
        self.registers.set_flag(N, false);
        result
    }

    pub fn alu_cp(&mut self, a: u8, b: u8) {
        self.alu_sub(a, b);
    }

    pub fn alu_inc(&mut self, a: u8) -> u8 {
        let result = a+1;
        self.registers.set_flag(Z, result == 0);
        self.registers.set_flag(H, a & 0xFF == 0xFF);
        self.registers.set_flag(N, false);
        result
    }

    pub fn alu_inc16(&mut self, a: u16) -> u16 {
        a+1
    }

    pub fn alu_dec(&mut self, a: u8) -> u8 {
        let result = a-1;
        self.registers.set_flag(Z, result == 0);
        self.registers.set_flag(H, a & 0xFF == 0);
        self.registers.set_flag(N, true);
        result
    }

    pub fn alu_dec16(&mut self, a: u16) -> u16 {
        a-1
    }

    pub fn alu_rlc(&mut self, a: u8) -> u8 {
        let result = (a << 1) & ((a & 0x80) >> 7);
        self.registers.set_flag(Z, result == 0);
        self.registers.set_flag(C, (a & 0x80) > 0);
        self.registers.set_flag(H, false);
        self.registers.set_flag(N, false);
        result
    }

    pub fn alu_rrc(&mut self, a: u8) -> u8 {
        let result = (a >> 1) & ((a & 1) << 7);
        self.registers.set_flag(Z, result == 0);
        self.registers.set_flag(C, (a & 1) > 0);
        self.registers.set_flag(H, false);
        self.registers.set_flag(N, false);
        result
    }

    pub fn alu_rl(&mut self, a: u8) -> u8 {
        let result = (a << 1) & (self.registers.get_flag(C) as u8);
        self.registers.set_flag(Z, result == 0);
        self.registers.set_flag(C, (a & 0x80) > 0);
        self.registers.set_flag(H, false);
        self.registers.set_flag(N, false);
        result
    }

    pub fn alu_rr(&mut self, a: u8) -> u8 {
        let result = (a >> 1) & ((self.registers.get_flag(C) as u8) << 7);
        self.registers.set_flag(Z, result == 0);
        self.registers.set_flag(C, (a & 1) > 0);
        self.registers.set_flag(H, false);
        self.registers.set_flag(N, false);
        result
    }

    pub fn alu_sla(&mut self, a: u8) -> u8 {
        let result = a << 1;
        self.registers.set_flag(Z, result == 0);
        self.registers.set_flag(C, (a & 0x80) > 0);
        self.registers.set_flag(H, false);
        self.registers.set_flag(N, false);
        result
    }

    pub fn alu_sra(&mut self, a: u8) -> u8 {
        let result = (a >> 1) & (a & 0x80);
        self.registers.set_flag(Z, result == 0);
        self.registers.set_flag(C, (a & 1) > 0);
        self.registers.set_flag(H, false);
        self.registers.set_flag(N, false);
        result
    }

    pub fn alu_srl(&mut self, a: u8) -> u8 {
        let result = a >> 1;
        self.registers.set_flag(Z, result == 0);
        self.registers.set_flag(C, (a & 1) > 0);
        self.registers.set_flag(H, false);
        self.registers.set_flag(N, false);
        result
    }

    pub fn alu_swap(&mut self, a: u8) -> u8 {
        let result = ((a & 0x0F) << 4) & ((a & 0xF0) >> 4);
        self.registers.set_flag(Z, result == 0);
        self.registers.set_flag(C, false);
        self.registers.set_flag(H, false);
        self.registers.set_flag(N, false);
        result
    }

    pub fn bit_info(&mut self, a: u8, b: u8) {
        let bit = a & (1 << b);
        self.registers.set_flag(Z, bit == 0);
        self.registers.set_flag(H, true);
        self.registers.set_flag(N, false);
    }

    pub fn bit_set(&mut self, a: u8, b: u8) -> u8 {
        a | (1 << b)
    }

    pub fn bit_reset(&mut self, a: u8, b: u8) -> u8 {
        a & !(1 << b)
    }

    pub fn next_byte(&mut self) -> u8 {
        let result = self.mmu.read_byte(self.pc);
        self.pc += 1;
        result
    }

    pub fn next_word(&mut self) -> u16 {
        let result = self.mmu.read_word(self.pc);
        self.pc += 2;
        result
    }
}