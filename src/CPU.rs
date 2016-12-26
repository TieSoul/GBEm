use registers::Registers;
use registers::RegisterFlags::{C,H,N,Z};
use mmu::MMU;

pub struct CPU {
    pub pc: u16,
    pub sp: u16,
    pub registers: Registers,
    pub mmu: MMU,
    pub ei: bool
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            pc: 0,
            sp: 0,
            registers: Registers::new(),
            mmu: MMU::new(),
            ei: true
        }
    }

    pub fn exec_opcode(&mut self, opcode: u8) -> u8 {
        let first = opcode >> 6;
        let second = (opcode >> 3) & 0b111;
        let third = opcode & 0b111;
        match first {
            0b00 => {
                match third {
                    0b000 => {
                        match second {
                            0b000 => 1, // 00 000 000 - NOP
                            0b001 => { // 00 001 000 - LD (nn), SP
                                let addr = self.next_word();
                                self.mmu.write_word(addr, self.sp);
                                5
                            },
                            0b010 => { // 00 010 000 - STOP
                                // stop mode to be implemented.
                                1
                            },
                            0b011 => { // 00 011 000 - JR e
                                let e = self.next_byte() as i8;
                                self.pc += e as u16;
                                3
                            }
                            _ => { // 00 1cc 000 - conditional JR e
                                let e = self.next_byte() as i8;
                                if (second == 0b100 && !self.registers.get_flag(Z)) ||
                                    (second == 0b101 && self.registers.get_flag(Z)) ||
                                    (second == 0b110 && !self.registers.get_flag(C)) ||
                                    (second == 0b111 && self.registers.get_flag(C)) {
                                    self.pc += e as u16;
                                    3
                                } else {
                                    2
                                }
                            }
                        }
                    },
                    0b001 => {
                        if (second & 1) == 0 { // 00 rr0 001 - LD rr, nn
                            let reg = second >> 1;
                            if reg == 0b11 {
                                self.sp = self.next_word();
                            } else {
                                let w = self.next_word();
                                self.registers.set_reg16(reg, w);
                            }
                            3
                        } else { // 00 rr1 001 - ADD HL, rr
                            let reg = second >> 1;
                            let val : u16;
                            if reg == 0b11 {
                                val = self.sp
                            } else {
                                val = self.registers.get_reg16(reg);
                            }
                            let hl = self.registers.hl();
                            let result = self.alu_add16(hl, val);
                            self.registers.set_hl(result);
                            2
                        }
                    },
                    0b010 => {
                        if (second & 1) == 1 { // 00 rr1 010 - LD A, (rr)
                                               // including 00 101 010 - LDI A, (HL)
                                               // and 00 111 010 - LDD A, (HL).
                            let reg = second >> 1;
                            let val : u16;
                            if reg > 0b01 {
                                val = self.registers.hl();
                                if (reg & 1) == 0 {
                                    self.registers.set_hl(val + 1)
                                } else {
                                    self.registers.set_hl(val - 1)
                                }
                            } else {
                                val = self.registers.get_reg16(reg);
                            }
                            let result = self.mmu.read_byte(val);
                            self.registers.a = result;
                            2
                        } else { // 00 rr0 010 - LD (rr), A
                                 // including 00 100 010 - LDI (HL), A
                                 // and 00 110 010 - LDD (HL), A.
                            let reg = second >> 1;
                            let val : u16;
                            if reg > 0b01 {
                                val = self.registers.hl();
                                if (reg & 1) == 0 {
                                    self.registers.set_hl(val + 1);
                                } else {
                                    self.registers.set_hl(val - 1);
                                }
                            } else {
                                val = self.registers.get_reg16(reg);
                            }
                            let n = self.mmu.read_byte(val);
                            self.mmu.write_byte(val, self.registers.a);
                            2
                        }
                    },
                    0b011 => { // 00 rr1 011 - DEC rr
                               // 00 rr0 011 - INC rr
                        let a = (1 - 2*(second & 1)) as u16;
                        let reg = second >> 1;
                        if reg == 0b11 {
                            self.sp += a;
                        } else {
                            let v = self.registers.get_reg16(reg);
                            self.registers.set_reg16(reg, v+a);
                        }
                        2
                    },
                    0b100 => { // 00 r 100 - INC r
                        if second == 0b110 {
                            let addr = self.registers.hl();
                            let mut val = self.mmu.read_byte(addr);
                            val = self.alu_inc(val);
                            self.mmu.write_byte(addr, val);
                            3
                        } else {
                            let mut val = self.registers.get_reg(second);
                            val = self.alu_inc(val);
                            self.registers.set_reg(second, val);
                            1
                        }
                    },
                    0b101 => { // 00 r 101 - DEC r
                        if second == 0b110 {
                            let addr = self.registers.hl();
                            let mut val = self.mmu.read_byte(addr);
                            val = self.alu_dec(val);
                            self.mmu.write_byte(addr, val);
                            3
                        } else {
                            let mut val = self.registers.get_reg(second);
                            val = self.alu_dec(val);
                            self.registers.set_reg(second, val);
                            1
                        }
                    },
                    0b110 => { // 00 r 110 - LD r, n
                        if second == 0b110 {
                            let addr = self.registers.hl();
                            let val = self.next_byte();
                            self.mmu.write_byte(addr, val);
                            3
                        } else {
                            let val = self.next_byte();
                            self.registers.set_reg(second, val);
                            2
                        }
                    },
                    _ => {
                        match second {
                            0b000 => { // 00 000 111 - RLCA
                                let a = self.registers.a;
                                let val = self.alu_rlc(a);
                                self.registers.set_flag(Z, false);
                                self.registers.a = val;
                                1
                            },
                            0b010 => { // 00 010 111 - RLA
                                let a = self.registers.a;
                                let val = self.alu_rl(a);
                                self.registers.set_flag(Z, false);
                                self.registers.a = val;
                                1
                            },
                            0b001 => { // 00 001 111 - RRCA
                                let a = self.registers.a;
                                let val = self.alu_rrc(a);
                                self.registers.set_flag(Z, false);
                                self.registers.a = val;
                                1
                            },
                            0b011 => { // 00 011 111 - RRA
                                let a = self.registers.a;
                                let val = self.alu_rr(a);
                                self.registers.set_flag(Z, false);
                                self.registers.a = val;
                                1
                            },
                            0b100 => { // 00 100 111 - DAA
                                let a = self.registers.a;
                                if (a & 0x0F) > 9 || self.registers.get_flag(H) {
                                    if self.registers.get_flag(N) {
                                        self.registers.a -= 0x06;
                                    } else {
                                        self.registers.a += 0x06;
                                    }
                                }
                                if (a & 0xF0) > 0x90 || self.registers.get_flag(C) {
                                    if self.registers.get_flag(N) {
                                        self.registers.a -= 0x60;
                                    } else {
                                        self.registers.a += 0x60;
                                    }
                                    self.registers.set_flag(C, true);
                                }
                                self.registers.set_flag(H, false);
                                let final_a = self.registers.a;
                                self.registers.set_flag(Z, final_a == 0);
                                1
                            },
                            0b101 => { // 00 101 111 - CPL
                                let not_a = !self.registers.a;
                                self.registers.a = not_a;
                                self.registers.set_flag(H, true);
                                self.registers.set_flag(N, true);
                                1
                            },
                            0b110 => { // 00 110 111 - SCF
                                self.registers.set_flag(C, true);
                                self.registers.set_flag(H, false);
                                self.registers.set_flag(N, false);
                                1
                            },
                            _ => { // 00 111 111 - CCF
                                let not_c = !self.registers.get_flag(C);
                                self.registers.set_flag(C, not_c);
                                self.registers.set_flag(H, false);
                                self.registers.set_flag(N, false);
                                1
                            }
                        }
                    }
                }
            },
            0b01 => {
                match second {
                    0b110 => {
                        match third {
                            0b110 => { // 01 110 110 - HALT
                                // HALT to be implemented.
                                1
                            },
                            _ => { // 01 110 r - LD (HL), r
                                let r = self.registers.get_reg(third);
                                let addr = self.registers.hl();
                                self.mmu.write_byte(addr, r);
                                2
                            }
                        }
                    },
                    _ => { // 01 r r' - LD r, r'
                        let val : u8;
                        if third == 0b110 {
                            let addr = self.registers.hl();
                            val = self.mmu.read_byte(addr);
                        } else {
                            val = self.registers.get_reg(third);
                        }
                        self.registers.set_reg(third, val);
                        if third == 0b110 {
                            2
                        } else {
                            1
                        }
                    }
                }
            },
            0b10 => {
                let val : u8;
                if third == 0b110 {
                    let addr = self.registers.hl();
                    val = self.mmu.read_byte(addr);
                } else {
                    val = self.registers.get_reg(third);
                }
                let a = self.registers.a;
                if second == 0b111 {
                    self.alu_cp(a, val);
                } else {
                    let result = match second {
                        0b000 => self.alu_add(a, val),
                        0b001 => self.alu_adc(a, val),
                        0b010 => self.alu_sub(a, val),
                        0b011 => self.alu_sbc(a, val),
                        0b100 => self.alu_and(a, val),
                        0b101 => self.alu_xor(a, val),
                        _ => self.alu_or(a, val)
                    };
                    self.registers.a = result;
                }
                if third == 0b110 {
                    2
                } else {
                    1
                }
            },
            _ => {
                match third {
                    0b000 => {
                        match second {
                            0b000 ... 0b011 => {
                                if (second == 0b000 && !self.registers.get_flag(Z)) ||
                                    (second == 0b001 && self.registers.get_flag(Z)) ||
                                    (second == 0b010 && !self.registers.get_flag(C)) ||
                                    (second == 0b011 && self.registers.get_flag(C)) {
                                    self.pc = self.pop();
                                    5
                                } else {
                                    2
                                }
                            },
                            0b100 => { // 11 100 000 - LD (0xFF00+n), A
                                let addr = 0xFF00 + (self.next_byte() as u16);
                                let a = self.registers.a;
                                self.mmu.write_byte(addr, a);
                                3
                            },
                            0b101 => { // 11 101 000 - ADD SP, e
                                let e = self.next_byte() as i8;
                                let sp = self.sp;
                                let newsp = self.alu_add16(sp, e as u16);
                                self.sp = newsp;
                                4
                            },
                            0b110 => { // 11 110 000 - LD A, (0xFF00+n)
                                let addr = 0xFF00 + (self.next_byte() as u16);
                                self.registers.a = self.mmu.read_byte(addr);
                                3
                            },
                            _ => { // 11 111 000 - LDHL SP, e
                                let e = self.next_byte() as i8;
                                let sp = self.sp;
                                let spe = self.alu_add16(sp, e as u16);
                                self.registers.set_hl(spe);
                                3
                            }
                        }
                    },
                    0b001 => {
                        match second {
                            _ if second & 1 == 0 => {
                                let reg = second >> 1;
                                let popped = self.pop();
                                self.registers.set_reg16(reg, popped);
                                3
                            },
                            0b001 => { // 11 001 001 - RET
                                self.pc = self.pop();
                                4
                            },
                            0b011 => { // 11 011 001 - RETI
                                self.pc = self.pop();
                                self.ei = true;
                                4
                            },
                            0b101 => { // 11 101 001 - JP (HL)
                                self.pc = self.registers.hl();
                                1
                            },
                            _ => { // 11 111 001 - LD SP, HL
                                self.sp = self.registers.hl();
                                2
                            }
                        }
                    },
                    0b010 => {
                        match second {
                            0b000 ... 0b011 => {
                                let e = self.next_word();
                                if (second == 0b000 && !self.registers.get_flag(Z)) ||
                                    (second == 0b001 && self.registers.get_flag(Z)) ||
                                    (second == 0b010 && !self.registers.get_flag(C)) ||
                                    (second == 0b011 && self.registers.get_flag(C)) {
                                    self.pc = e;
                                    4
                                } else {
                                    3
                                }
                            },
                            0b100 => { // 11 100 010 - LD (0xFF00+C), A
                                let addr = 0xFF00 + (self.registers.c as u16);
                                let a = self.registers.a;
                                self.mmu.write_byte(addr, a);
                                2
                            },
                            0b101 => { // 11 101 010 - LD (nn), A
                                let addr = self.next_word();
                                let a = self.registers.a;
                                self.mmu.write_byte(addr, a);
                                4
                            },
                            0b110 => { // 11 110 010 - LD A, (0xFF00+C)
                                let addr = 0xFF00 + (self.registers.c as u16);
                                self.registers.a = self.mmu.read_byte(addr);
                                2
                            },
                            _ => { // 11 111 010 - LD A, (nn)
                                let addr = self.next_word();
                                self.registers.a = self.mmu.read_byte(addr);
                                4
                            }
                        }
                    },
                    0b011 => {
                        match second {
                            0b000 => { // 11 000 011 - JP nn
                                self.pc = self.next_word();
                                4
                            },
                            0b001 => { // 11 001 011 - prefix for two-byte opcodes.
                                let opcode2 = self.next_byte();
                                self.exec_opcode2(opcode2)
                            },
                            0b110 => { // 11 110 011 - DI
                                self.ei = false;
                                1
                            },
                            0b111 => { // 11 111 011 - EI
                                self.ei = true;
                                1
                            },
                            _ => panic!("Unknown opcode {:x}!", opcode)
                        }
                    },
                    0b100 => { // 11 0cc 100 - CALL cc, nn
                        if second >= 0b100 {
                            panic!("Unknown opcode {:x}!", opcode);
                        }
                        let e = self.next_word();
                        if (second == 0b000 && !self.registers.get_flag(Z)) ||
                            (second == 0b001 && self.registers.get_flag(Z)) ||
                            (second == 0b010 && !self.registers.get_flag(C)) ||
                            (second == 0b011 && self.registers.get_flag(C)) {
                            let pc = self.pc;
                            self.push(pc);
                            self.pc = e;
                            6
                        } else {
                            3
                        }
                    },
                    0b101 => {
                        match second {
                            _ if second & 1 == 0 => { // 11 rr0 101 - PUSH rr
                                let reg = second >> 1;
                                let val = self.registers.get_reg16(reg);
                                self.push(val);
                                4
                            },
                            0b001 => { // 11 001 101 - CALL nn
                                let e = self.next_word();
                                let pc = self.pc;
                                self.push(pc);
                                self.pc = e;
                                6
                            },
                            _ => panic!("Unknown opcode {:x}!", opcode)
                        }
                    },
                    0b110 => { // 8-bit arithmetic/logic operations on immediates
                        let n = self.next_byte();
                        let a = self.registers.a;
                        if second == 0b111 {
                            self.alu_cp(a, n);
                        } else {
                            self.registers.a = match second {
                                0b000 => self.alu_add(a, n),
                                0b001 => self.alu_adc(a, n),
                                0b010 => self.alu_sub(a, n),
                                0b011 => self.alu_sbc(a, n),
                                0b100 => self.alu_and(a, n),
                                0b101 => self.alu_xor(a, n),
                                _     => self.alu_or(a, n)
                            }
                        }
                        2
                    }
                    _ => { // 11  t  111 - RST t
                        let t = second * 0x08;
                        let pc = self.pc;
                        self.push(pc);
                        self.pc = t as u16;
                        4
                    }
                }
            }
        }
    }

    pub fn exec_opcode2(&mut self, opcode : u8) -> u8 {
        let first = opcode >> 6;
        let second = (opcode >> 3) & 0b111;
        let third = opcode & 0b111;
        match first {
            0b00 => {
                let val: u8;
                if third == 0b110 {
                    let addr = self.registers.hl();
                    val = self.mmu.read_byte(addr);
                } else {
                    val = self.registers.get_reg(third);
                }
                let result = match second {
                    0b000 => self.alu_rlc(val),
                    0b001 => self.alu_rrc(val),
                    0b010 => self.alu_rl(val),
                    0b011 => self.alu_rr(val),
                    0b100 => self.alu_sla(val),
                    0b101 => self.alu_sra(val),
                    0b110 => self.alu_swap(val),
                    _ => self.alu_srl(val)
                };
                if third == 0b110 {
                    let addr = self.registers.hl();
                    self.mmu.write_byte(addr, result);
                    4
                } else {
                    self.registers.set_reg(third, result);
                    2
                }
            },
            0b01 => { // 01 b r - BIT r, b
                let val: u8;
                if third == 0b110 {
                    let addr = self.registers.hl();
                    val = self.mmu.read_byte(addr);
                } else {
                    val = self.registers.get_reg(third);
                }
                self.bit_info(val, second);
                if third == 0b110 {
                    3
                } else {
                    2
                }
            },
            0b10 => { // 10 b r - RES r, b
                let val: u8;
                if third == 0b110 {
                    let addr = self.registers.hl();
                    val = self.mmu.read_byte(addr);
                } else {
                    val = self.registers.get_reg(third);
                }
                let result = self.bit_reset(val, second);
                if third == 0b110 {
                    let addr = self.registers.hl();
                    self.mmu.write_byte(addr, result);
                    4
                } else {
                    self.registers.set_reg(third, result);
                    2
                }
            },
            _ => { // 11 b r - SET r, b
                let val: u8;
                if third == 0b110 {
                    let addr = self.registers.hl();
                    val = self.mmu.read_byte(addr);
                } else {
                    val = self.registers.get_reg(third);
                }
                let result = self.bit_set(val, second);
                if third == 0b110 {
                    let addr = self.registers.hl();
                    self.mmu.write_byte(addr, result);
                    4
                } else {
                    self.registers.set_reg(third, result);
                    2
                }
            }
        }
    }

    pub fn push(&mut self, a : u16) {
        self.mmu.write_word(self.sp-2, a);
        self.sp -= 2;
    }

    pub fn pop(&mut self) -> u16 {
        self.sp += 2;
        self.mmu.read_word(self.sp-2)
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
        let result = (a << 1) | ((a & 0x80) >> 7);
        self.registers.set_flag(Z, result == 0);
        self.registers.set_flag(C, (a & 0x80) > 0);
        self.registers.set_flag(H, false);
        self.registers.set_flag(N, false);
        result
    }

    pub fn alu_rrc(&mut self, a: u8) -> u8 {
        let result = (a >> 1) | ((a & 1) << 7);
        self.registers.set_flag(Z, result == 0);
        self.registers.set_flag(C, (a & 1) > 0);
        self.registers.set_flag(H, false);
        self.registers.set_flag(N, false);
        result
    }

    pub fn alu_rl(&mut self, a: u8) -> u8 {
        let result = (a << 1) | (self.registers.get_flag(C) as u8);
        self.registers.set_flag(Z, result == 0);
        self.registers.set_flag(C, (a & 0x80) > 0);
        self.registers.set_flag(H, false);
        self.registers.set_flag(N, false);
        result
    }

    pub fn alu_rr(&mut self, a: u8) -> u8 {
        let result = (a >> 1) | ((self.registers.get_flag(C) as u8) << 7);
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
        let result = (a >> 1) | (a & 0x80);
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
        let result = ((a & 0x0F) << 4) | ((a & 0xF0) >> 4);
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