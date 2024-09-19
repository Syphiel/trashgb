use crate::ppu::Palette;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;

#[derive(Debug)]
pub struct Joypad {
    a: bool,
    b: bool,
    start: bool,
    select: bool,
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

#[derive(Debug)]
pub struct Mmu {
    // Memory Map
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
    // Cartridge Info
    title: [u8; 0x0F],
    cartridge_type: u8,
    rom_size: u8,
    ram_size: u8,
    // Additional ROMs
    extra_banks: Vec<[u8; 0x4000]>,
    // Current banks
    rom_bank: u8,
    ram_bank: u8,
    rom_mode: u8,
    // Misc
    window_counter: u8,
    timer: u16,
    pub joypad: Joypad,
}

impl Joypad {
    pub fn read_state(&self, select: bool) -> u8 {
        match select {
            true => {
                let mut state = 0xF;
                if self.a {
                    state &= 0b1110;
                }
                if self.b {
                    state &= 0b1101;
                }
                if self.select {
                    state &= 0b1011;
                }
                if self.start {
                    state &= 0b0111;
                }
                state
            }
            false => {
                let mut state = 0xF;
                if self.right {
                    state &= 0b1110;
                }
                if self.left {
                    state &= 0b1101;
                }
                if self.up {
                    state &= 0b1011;
                }
                if self.down {
                    state &= 0b0111;
                }
                state
            }
        }
    }
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

            title: [0; 0x0F],
            cartridge_type: 0,
            rom_size: 0,
            ram_size: 0,

            extra_banks: Vec::new(),

            rom_bank: 1,
            ram_bank: 0,
            rom_mode: 0,

            window_counter: 0,
            timer: 0,
            joypad: Joypad {
                a: false,
                b: false,
                start: false,
                select: false,
                up: false,
                down: false,
                left: false,
                right: false,
            },
        }
    }

    pub fn load_game(&mut self, game: File) {
        for (index, byte) in BufReader::new(game).bytes().enumerate() {
            if index < 0x4000 {
                self.bank0[index] = byte.unwrap();
            } else if index < 0x8000 {
                self.bank1[index - 0x4000] = byte.unwrap();
            } else {
                if self.extra_banks.len() <= (index - 0x8000) / 0x4000 {
                    self.extra_banks.push([0; 0x4000]);
                    // println!(
                    //     "Loaded bank {} @ Index {:#06X}",
                    //     self.extra_banks.len(),
                    //     index
                    // );
                }
                self.extra_banks.last_mut().unwrap()[index % 0x4000] = byte.unwrap();
            }
        }
        self.title = self.bank0[0x134..0x143].try_into().unwrap();
        self.cartridge_type = self.bank0[0x147];
        self.rom_size = self.bank0[0x148];
        self.ram_size = self.bank0[0x149];

        // println!("Title: {:?}", std::str::from_utf8(&self.title).unwrap());
        // println!("Cartridge Type: {:X}", self.cartridge_type);
        // println!("ROM Size: {:X}", self.rom_size);
        // println!("RAM Size: {:X}", self.ram_size);

        self.rom_size = 2 << self.bank0[0x148];

        if self.rom_size != self.extra_banks.len() as u8 + 2 {
            eprintln!(
                "ROM Size ({}) does not match actual size ({})",
                self.rom_size,
                self.extra_banks.len()
            );
            self.rom_size = self.extra_banks.len() as u8 + 4;
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        if address == 0xFF00 {
            let select = self.io[0x00] & 0b0001_0000 == 0b0001_0000;
            return self.joypad.read_state(select);
        }
        match address {
            0x0000..=0x00FF => {
                if self.io[0x50] == 0x00 {
                    self.bootstrap[address as usize]
                } else {
                    self.bank0[address as usize]
                }
            }
            0x0100..=0x3FFF => self.bank0[address as usize],
            0x4000..=0x7FFF => {
                if self.rom_bank < 2 {
                    self.bank1[address as usize - 0x4000]
                } else {
                    self.extra_banks[self.rom_bank as usize - 2][address as usize - 0x4000]
                }
            }
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
        if address == 0xFF04 {
            self.timer = 0;
        }
        if address == 0xFF46 {
            /* DMA Transfer */
            let start = (value as u16) << 8;
            for i in 0..0xA0 {
                self.write_byte(0xFE00 + i, self.read_byte(start + i));
            }
            return;
        }
        match address {
            0x0000..=0x1FFF => { /* RAM Enable */ }
            0x2000..=0x3FFF => {
                /* ROM Bank */
                if self.rom_size < 2 || self.rom_size < value & 0b0001_1111 {
                    return;
                }
                if value & 0b0001_1111 == 0 {
                    self.rom_bank = 1;
                } else {
                    self.rom_bank = value & 0b0001_1111;
                }
            }
            0x4000..=0x5FFF => {
                /* RAM Bank or Upper Bits of ROM Bank */
                if self.rom_mode == 1 {
                    self.ram_bank = value & 0b0000_0011;
                } else {
                    self.rom_bank |= (value & 0b0000_0011) << 5;
                }
            }
            0x6000..=0x7FFF => {
                /* ROM/RAM Mode Select */
                if value & 0x0000_0001 == 0 {
                    self.rom_mode = 0;
                } else {
                    self.rom_mode = 1;
                }
            }
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
            0x4000..=0x7FFF => {
                if self.rom_bank < 2 {
                    &mut self.bank1[address as usize - 0x4000]
                } else {
                    &mut self.extra_banks[self.rom_bank as usize - 2][address as usize - 0x4000]
                }
            }
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

    pub fn get_bg_enable(&self) -> bool {
        self.io[0x40] & 0b0000_0001 == 0b0000_0001
    }

    pub fn get_window_enable(&self) -> bool {
        self.io[0x40] & 0b0010_0000 == 0b0010_0000
    }

    pub fn get_obj_enable(&self) -> bool {
        self.io[0x40] & 0b0000_0010 == 0b0000_0010
    }

    pub fn get_bg_map_mode(&self) -> bool {
        self.io[0x40] & 0b0000_1000 == 0b0000_1000
    }

    pub fn get_window_map_mode(&self) -> bool {
        self.io[0x40] & 0b0100_0000 == 0b0100_0000
    }

    pub fn get_tile_mode(&self) -> bool {
        self.io[0x40] & 0b0001_0000 == 0b0001_0000
    }

    pub fn get_bg_tile_data(&self) -> &[u8; 0x1000] {
        if self.get_tile_mode() {
            self.vram[0..0x1000].try_into().unwrap()
        } else {
            self.vram[0x800..0x1800].try_into().unwrap()
        }
    }

    pub fn get_bg_tile_map(&self) -> &[u8; 0x400] {
        if self.get_bg_map_mode() {
            self.vram[0x1C00..0x2000].try_into().unwrap()
        } else {
            self.vram[0x1800..0x1C00].try_into().unwrap()
        }
    }

    pub fn get_window_tile_map(&self) -> &[u8; 0x400] {
        if self.get_window_map_mode() {
            self.vram[0x1C00..0x2000].try_into().unwrap()
        } else {
            self.vram[0x1800..0x1C00].try_into().unwrap()
        }
    }

    pub fn get_window_pos(&self) -> (u8, u8) {
        (self.io[0x4A], self.io[0x4B])
    }

    pub fn get_oam(&self) -> &[u8; 0xA0] {
        &self.oam
    }

    pub fn get_oam_tile_data(&self) -> &[u8; 0x1000] {
        self.vram[0..0x1000].try_into().unwrap()
    }

    pub fn get_obj_size(&self) -> bool {
        self.io[0x40] & 0b0000_0100 == 0b0000_0100
    }

    pub fn get_bg_palette(&self) -> [Palette; 4] {
        Palette::from_u8(self.io[0x47])
    }

    pub fn get_obj_palette(&self, palette: usize) -> [Palette; 4] {
        Palette::from_u8(self.io[0x48 + (palette & 0x1)])
    }

    pub fn get_window_counter(&self) -> u8 {
        self.window_counter
    }

    pub fn set_window_counter(&mut self, value: u8) {
        self.window_counter = value;
    }

    pub fn increment_timer(&mut self, cycles: u32) -> bool {
        let cycles = cycles * 4;
        let timer = self.timer.wrapping_add(cycles as u16);
        let shift = match self.io[0x07] & 0b0000_0011 {
            0b00 => 9,
            0b01 => 3,
            0b10 => 5,
            0b11 => 7,
            _ => unreachable!(),
        };
        // if (self.timer >> shift) & 0b1 == 0b1
        //     && (timer >> shift) & 0b1 == 0b0
        if timer & (!0_u16 << shift) != self.timer & (!0_u16 << shift)
            && self.io[0x07] & 0b100 == 0b100
        {
            self.io[0x05] = self.io[0x05].wrapping_add(1);
            if self.io[0x05] == 0 {
                self.io[0x05] = self.io[0x06] + 1;
                self.timer = timer;
                self.io[0x04] = (self.timer >> 8) as u8;
                return true;
            }
        }
        self.timer = timer;
        // self.write_byte(0xFF04, (self.timer >> 8) as u8);
        self.io[0x04] = (self.timer >> 8) as u8;
        false
    }
    pub fn joypad_a(&mut self, pressed: bool) {
        self.joypad.a = pressed;
        if pressed {
            self.io[0x0F] |= 0b0001_0000;
        }
    }
    pub fn joypad_b(&mut self, pressed: bool) {
        self.joypad.b = pressed;
        if pressed {
            self.io[0x0F] |= 0b0001_0000;
        }
    }
    pub fn joypad_start(&mut self, pressed: bool) {
        self.joypad.start = pressed;
        if pressed {
            self.io[0x0F] |= 0b0001_0000;
        }
    }
    pub fn joypad_select(&mut self, pressed: bool) {
        self.joypad.select = pressed;
        if pressed {
            self.io[0x0F] |= 0b0001_0000;
        }
    }
    pub fn joypad_up(&mut self, pressed: bool) {
        self.joypad.up = pressed;
        if pressed {
            self.io[0x0F] |= 0b0001_0000;
        }
    }
    pub fn joypad_down(&mut self, pressed: bool) {
        self.joypad.down = pressed;
        if pressed {
            self.io[0x0F] |= 0b0001_0000;
        }
    }
    pub fn joypad_left(&mut self, pressed: bool) {
        self.joypad.left = pressed;
        if pressed {
            self.io[0x0F] |= 0b0001_0000;
        }
    }
    pub fn joypad_right(&mut self, pressed: bool) {
        self.joypad.right = pressed;
        if pressed {
            self.io[0x0F] |= 0b0001_0000;
        }
    }
}
