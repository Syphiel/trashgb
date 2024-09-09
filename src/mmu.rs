use std::fs::File;
use std::io::BufReader;
use std::io::Read;

#[derive(Debug)]
pub struct Mmu {
    bootstrap: [u8; 0x100],
    bank0: [u8; 0x4000],
    bank1: [u8; 0x4000],
    vram: [u8; 0x2000],
    eram: [u8; 0x2000],
    wram1: [u8; 0x2000],
    wram2: [u8; 0x2000],
    oam: [u8; 0x00A0],
    io: [u8; 0x0080],
    hram: [u8; 0x007F],
    ie: u8,
    dummy: u8,
}

impl Mmu {
    pub fn new() -> Self {
        Self {
            bootstrap: *include_bytes!("../roms/bootstrap.gb"),
            bank0: [0; 0x4000],
            bank1: [0; 0x4000],
            vram: [0; 0x2000],
            eram: [0; 0x2000],
            wram1: [0; 0x2000],
            wram2: [0; 0x2000],
            oam: [0; 0x00A0],
            io: [0; 0x0080],
            hram: [0; 0x007F],
            ie: 0,
            dummy: 0,
        }
    }

    pub fn load_game(&mut self, game: File) {
        for (index, byte) in BufReader::new(game).bytes().enumerate() {
            if index < 0x4000 {
                self.bank0[index] = byte.unwrap();
            } else if index < 0x8000 {
                self.bank1[index - 0x4000] = byte.unwrap();
            } else {
                break;
            }
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        if address == 0xFF00 {
            return 0xF;
        } // TODO: Implement joypad
        match address {
            0x0000..=0x00FF => {
                if self.io[0x50] == 0x00 {
                    self.bootstrap[address as usize]
                } else {
                    self.bank0[address as usize]
                }
            }
            0x0100..=0x3FFF => self.bank0[address as usize],
            0x4000..=0x7FFF => self.bank1[address as usize - 0x4000],
            0x8000..=0x9FFF => self.vram[address as usize - 0x8000],
            0xA000..=0xBFFF => self.eram[address as usize - 0xA000],
            0xC000..=0xCFFF => self.wram1[address as usize - 0xC000],
            0xD000..=0xDFFF => self.wram2[address as usize - 0xD000],
            0xE000..=0xFDFF => 0xFF,
            0xFE00..=0xFE9F => self.oam[address as usize - 0xFE00],
            0xFEA0..=0xFEFF => 0xFF,
            0xFF00..=0xFF7F => self.io[address as usize - 0xFF00],
            0xFF80..=0xFFFE => self.hram[address as usize - 0xFF80],
            0xFFFF => self.ie,
        }
    }

    pub fn read_word(&self, address: u16) -> u16 {
        let low = self.read_byte(address) as u16;
        let high = self.read_byte(address + 1) as u16;
        (high << 8) | low
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        if address == 0x2000 {
            return;
        }
        match address {
            0x0000..=0x3FFF => {}
            0x4000..=0x7FFF => {}
            0x8000..=0x9FFF => self.vram[address as usize - 0x8000] = value,
            0xA000..=0xBFFF => self.eram[address as usize - 0xA000] = value,
            0xC000..=0xCFFF => self.wram1[address as usize - 0xC000] = value,
            0xD000..=0xDFFF => self.wram2[address as usize - 0xD000] = value,
            0xE000..=0xFDFF => {}
            0xFE00..=0xFE9F => self.oam[address as usize - 0xFE00] = value,
            0xFEA0..=0xFEFF => {}
            0xFF00..=0xFF7F => self.io[address as usize - 0xFF00] = value,
            0xFF80..=0xFFFE => self.hram[address as usize - 0xFF80] = value,
            0xFFFF => self.ie = value,
        }
    }

    pub fn write_word(&mut self, address: u16, value: u16) {
        let low = value as u8;
        let high = (value >> 8) as u8;
        self.write_byte(address, low);
        self.write_byte(address + 1, high);
    }

    pub fn get_mut_byte(&mut self, address: u16) -> &mut u8 {
        match address {
            0x0000..=0x3FFF => &mut self.bank0[address as usize],
            0x4000..=0x7FFF => &mut self.bank1[address as usize - 0x4000],
            0x8000..=0x9FFF => &mut self.vram[address as usize - 0x8000],
            0xA000..=0xBFFF => &mut self.eram[address as usize - 0xA000],
            0xC000..=0xCFFF => &mut self.wram1[address as usize - 0xC000],
            0xD000..=0xDFFF => &mut self.wram2[address as usize - 0xD000],
            0xE000..=0xFDFF => &mut self.dummy,
            0xFE00..=0xFE9F => &mut self.oam[address as usize - 0xFE00],
            0xFEA0..=0xFEFF => &mut self.dummy,
            0xFF00..=0xFF7F => &mut self.io[address as usize - 0xFF00],
            0xFF80..=0xFFFE => &mut self.hram[address as usize - 0xFF80],
            0xFFFF => &mut self.ie,
        }
    }

    pub fn get_tile_data<'a>(&'a self) -> &'a [u8; 0x1000] {
        // TODO: Check FF40 bit 4
        self.vram[0..0x1000].try_into().unwrap()
    }

    pub fn get_tile_map<'a>(&'a self) -> &'a [u8; 0x400] {
        // TODO: Check FF40 bit 4
        self.vram[0x1800..0x1C00].try_into().unwrap()
    }
}
