pub enum RegisterFlags {
    C = 0b00010000,
    H = 0b00100000,
    N = 0b01000000,
    Z = 0b10000000
}

pub struct Registers {
    pub a: u8,
    f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8
}
#[allow(dead_code)]
impl Registers {
    pub fn af(&self) -> u16 {
        ((self.a as u16) << 8) | (self.f as u16)
    }
    pub fn set_af(&mut self, val: u16) {
        self.a = (val >> 8) as u8;
        self.f = (val & 0xFF) as u8;
    }

    pub fn bc(&self) -> u16 {
        ((self.b as u16) << 8) | (self.c as u16)
    }
    pub fn set_bc(&mut self, val: u16) {
        self.b = (val >> 8) as u8;
        self.c = (val & 0xFF) as u8;
    }

    pub fn de(&self) -> u16 {
        ((self.d as u16) << 8) | (self.e as u16)
    }
    pub fn set_de(&mut self, val: u16) {
        self.d = (val >> 8) as u8;
        self.e = (val & 0xFF) as u8;
    }

    pub fn hl(&self) -> u16 {
        ((self.a as u16) << 8) | (self.f as u16)
    }
    pub fn set_hl(&mut self, val: u16) {
        self.a = (val >> 8) as u8;
        self.f = (val & 0xFF) as u8;
    }

    pub fn get_reg(&mut self, code: u8) -> u8 {
        match code {
            0 => self.b,
            1 => self.c,
            2 => self.d,
            3 => self.e,
            4 => self.h,
            5 => self.l,
            7 => self.a,
            _ => panic!() // should not happen.
        }
    }

    pub fn get_reg16(&mut self, code: u8) -> u16 {
        match code {
            0 => self.bc(),
            1 => self.de(),
            2 => self.hl(),
            3 => self.af(),
            _ => panic!()
        }
    }

    pub fn set_reg(&mut self, code: u8, val: u8) {
        match code {
            0 => self.b = val,
            1 => self.c = val,
            2 => self.d = val,
            3 => self.e = val,
            4 => self.h = val,
            5 => self.l = val,
            7 => self.a = val,
            _ => panic!() // should not happen.
        }
    }

    pub fn set_reg16(&mut self, code: u8, val: u16) {
        match code {
            0 => self.set_bc(val),
            1 => self.set_de(val),
            2 => self.set_hl(val),
            3 => self.set_af(val),
            _ => panic!()
        }
    }

    pub fn set_flag(&mut self, flag: RegisterFlags, value: bool) {
        if value {
            self.f |= flag as u8;
        } else {
            self.f &= 0xFF - (flag as u8);
        }
    }

    pub fn get_flag(&mut self, flag: RegisterFlags) -> bool {
        (self.f & (flag as u8)) > 0
    }

    pub fn new() -> Registers {
        Registers {
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
        }
    }
}