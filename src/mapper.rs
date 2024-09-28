use crate::mmu::Mmu;

pub struct Mbc1 {
    rom_size: u8,
    ram_size: u8,
    ram_enable: bool,
    rom_bank: u8,
    rom_mode: u8,
}

pub trait Mapper {
    fn write_register(&mut self, address: u16, value: u8, mmu: &mut Mmu);
}

impl Mapper for Mbc1 {
    fn write_register(&mut self, address: u16, value: u8, mmu: &mut Mmu) {
        match address {
            0x0000..=0x1FFF => {
                /* RAMG */
                self.ram_enable = value & 0x0F == 0x0A && self.ram_size > 0;
                match self.ram_enable {
                    true => mmu.eram = Some(self.rom_bank as usize >> 5),
                    false => mmu.eram = None,
                }
            }
            0x2000..=0x3FFF => {
                /* BANK1 */
                self.rom_bank &= 0b1110_0000;
                self.rom_bank |= value & 0b0001_1111;

                let bank = match self.rom_bank {
                    0..=1 => 1,
                    n if n & 0b0001_1111 == 0 => n % self.rom_size + 1,
                    n @ 2..96 if n < self.rom_size => n,
                    n => n % self.rom_size,
                };
                mmu.bank1 = bank as usize;
            }
            0x4000..=0x5FFF => {
                /* BANK2 */
                self.rom_bank &= 0b0001_1111;
                self.rom_bank |= (value & 0b0000_0011) << 5;

                if self.rom_size < 64 || self.rom_mode == 0 {
                    mmu.bank0 = 0;
                } else {
                    let bank = self.rom_bank as usize & 0b0110_0000;
                    mmu.bank0 = bank;
                }

                let mut rambank = self.rom_bank as usize >> 5;
                if rambank >= self.ram_size as usize {
                    rambank = 0;
                }
                let bank0 = (self.rom_bank as usize & 0b0110_0000) % self.rom_size as usize;
                let bank1 = match self.rom_bank {
                    0..=1 => 1,
                    n if n & 0b0001_1111 == 0 => n % self.rom_size + 1,
                    n @ 2..96 if n < self.rom_size => n,
                    n => n % self.rom_size,
                };
                match self.rom_mode {
                    0 if self.ram_enable => {
                        mmu.bank0 = 0;
                        mmu.eram = Some(0);
                    }
                    0 if !self.ram_enable => {
                        mmu.bank0 = 0;
                        mmu.eram = None;
                    }
                    1 if self.ram_enable => {
                        mmu.bank0 = bank0;
                        mmu.eram = Some(rambank);
                    }
                    1 if !self.ram_enable => {
                        mmu.bank0 = bank0;
                        mmu.eram = None;
                    }
                    _ => unreachable!(),
                }
                if self.rom_size < 64 {
                    mmu.bank0 = 0;
                }

                mmu.bank1 = bank1 as usize;
            }
            0x6000..=0x7FFF => {
                /* MODE */
                self.rom_mode = value & 0x0000_0001;

                let mut rambank = self.rom_bank as usize >> 5;
                if rambank >= self.ram_size as usize {
                    rambank = 0;
                }
                let bank0 = self.rom_bank as usize & 0b0110_0000;

                match self.rom_mode {
                    0 if self.ram_enable => {
                        mmu.bank0 = 0;
                        mmu.eram = Some(0);
                    }
                    0 if !self.ram_enable => {
                        mmu.bank0 = 0;
                        mmu.eram = None;
                    }
                    1 if self.ram_enable => {
                        mmu.bank0 = bank0;
                        mmu.eram = Some(rambank);
                    }
                    1 if !self.ram_enable => {
                        mmu.bank0 = bank0;
                        mmu.eram = None;
                    }
                    _ => unreachable!(),
                }
                if self.rom_size < 64 {
                    mmu.bank0 = 0;
                }
            }
            _ => unreachable!(),
        }
        if self.ram_size == 0 {
            mmu.eram = None;
        }
    }
}

impl Mbc1 {
    pub fn new(rom_size: u8, ram_size: u8, mmu: &mut Mmu) -> Self {
        mmu.bank0 = 0;
        mmu.bank1 = 1;
        mmu.eram = None;
        Mbc1 {
            rom_size,
            ram_size,
            ram_enable: false,
            rom_bank: 1,
            rom_mode: 0,
        }
    }
}
