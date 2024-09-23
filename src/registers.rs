use std::cell::Cell;

#[derive(Debug)]
pub enum R8 {
    B,
    C,
    D,
    E,
    H,
    L,
    M,
    A,
}

impl R8 {
    pub fn from_u8(value: u8) -> Self {
        match value & 0b111 {
            0b000 => R8::B,
            0b001 => R8::C,
            0b010 => R8::D,
            0b011 => R8::E,
            0b100 => R8::H,
            0b101 => R8::L,
            0b110 => R8::M,
            0b111 => R8::A,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub enum R16 {
    BC,
    DE,
    HL,
    SP,
}

impl R16 {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0b00 => R16::BC,
            0b01 => R16::DE,
            0b10 => R16::HL,
            0b11 => R16::SP,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub enum R16stk {
    BC,
    DE,
    HL,
    AF,
}

impl R16stk {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0b00 => R16stk::BC,
            0b01 => R16stk::DE,
            0b10 => R16stk::HL,
            0b11 => R16stk::AF,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub enum R16mem {
    BC,
    DE,
    HLi,
    HLd,
}

impl R16mem {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0b00 => R16mem::BC,
            0b01 => R16mem::DE,
            0b10 => R16mem::HLi,
            0b11 => R16mem::HLd,
            _ => unreachable!(),
        }
    }
}

// #[derive(Debug)]
// pub enum Operand {
//     R8(R8),
//     R16(R16),
//     R16stk(R16stk),
//     R16mem(R16mem),
//     Cond,
//     B3,
//     Tgt3,
//     Imm8,
//     Imm16,
// }

#[derive(Debug)]
pub struct Flags {
    pub zero: Cell<bool>,
    pub subtract: Cell<bool>,
    pub half_carry: Cell<bool>,
    pub carry: Cell<bool>,
}

impl Flags {
    pub fn get_condition(&self, flag: u8) -> bool {
        match flag {
            0b000 => !self.zero.get(),
            0b001 => self.zero.get(),
            0b010 => !self.carry.get(),
            0b011 => self.carry.get(),
            _ => unreachable!(),
        }
    }

    pub fn to_u8(&self) -> u8 {
        (self.zero.get() as u8) << 7
            | (self.subtract.get() as u8) << 6
            | (self.half_carry.get() as u8) << 5
            | (self.carry.get() as u8) << 4
    }

    pub fn set_from_u8(&self, value: u8) {
        self.zero.set((value >> 7 & 0b1) == 1);
        self.subtract.set((value >> 6 & 0b1) == 1);
        self.half_carry.set((value >> 5 & 0b1) == 1);
        self.carry.set((value >> 4 & 0b1) == 1);
    }
}

pub enum R8OrMem<'a> {
    R8(&'a Cell<u8>),
    Ptr(u16),
}

pub enum R16OrSP<'a> {
    R16(&'a Cell<u8>, &'a Cell<u8>),
    SP,
}

#[derive(Debug)]
pub struct Registers {
    pub a: Cell<u8>,
    pub b: Cell<u8>,
    pub c: Cell<u8>,
    pub d: Cell<u8>,
    pub e: Cell<u8>,
    pub h: Cell<u8>,
    pub l: Cell<u8>,
    pub flags: Flags,
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            a: Cell::new(100),
            b: Cell::new(210),
            c: Cell::new(32),
            d: Cell::new(41),
            e: Cell::new(120),
            h: Cell::new(222),
            l: Cell::new(11),
            flags: Flags {
                zero: Cell::new(false),
                subtract: Cell::new(false),
                half_carry: Cell::new(false),
                carry: Cell::new(false),
            },
        }
    }

    pub fn get_r8(&self, r8: R8) -> R8OrMem {
        match r8 {
            R8::A => R8OrMem::R8(&self.a),
            R8::B => R8OrMem::R8(&self.b),
            R8::C => R8OrMem::R8(&self.c),
            R8::D => R8OrMem::R8(&self.d),
            R8::E => R8OrMem::R8(&self.e),
            R8::H => R8OrMem::R8(&self.h),
            R8::L => R8OrMem::R8(&self.l),
            R8::M => R8OrMem::Ptr((self.h.get() as u16) << 8 | self.l.get() as u16),
        }
    }

    pub fn get_r16(&self, r16: R16) -> R16OrSP {
        match r16 {
            R16::BC => R16OrSP::R16(&self.b, &self.c),
            R16::DE => R16OrSP::R16(&self.d, &self.e),
            R16::HL => R16OrSP::R16(&self.h, &self.l),
            R16::SP => R16OrSP::SP,
        }
    }

    pub fn get_r16mem(&self, r16mem: R16mem) -> (&Cell<u8>, &Cell<u8>) {
        match r16mem {
            R16mem::BC => (&self.b, &self.c),
            R16mem::DE => (&self.d, &self.e),
            R16mem::HLi => (&self.h, &self.l),
            R16mem::HLd => (&self.h, &self.l),
        }
    }

    pub fn get_r16stk(&self, r16stk: R16stk) -> (&Cell<u8>, &Cell<u8>) {
        match r16stk {
            R16stk::BC => (&self.b, &self.c),
            R16stk::DE => (&self.d, &self.e),
            R16stk::HL => (&self.h, &self.l),
            R16stk::AF => (&self.a, &self.a),
        }
    }
}
