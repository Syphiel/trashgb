use crate::mmu::Mmu;

#[derive(Debug)]
pub enum Palette {
    White,
    LightGray,
    DarkGray,
    Black,
}

impl Palette {
    pub fn from_u8(value: u8) -> [Self; 4] {
        core::array::from_fn(|i| match (value >> (i * 2)) & 0b11 {
            0 => Self::White,
            1 => Self::LightGray,
            2 => Self::DarkGray,
            3 => Self::Black,
            _ => unreachable!(),
        })
    }
}

#[derive(Debug)]
pub struct ObjectAttribute {
    pub y: i16,
    pub x: i16,
    pub tile: u8,
    pub priority: bool,
    pub y_flip: bool,
    pub x_flip: bool,
    pub palette: usize,
}

impl ObjectAttribute {
    pub fn from_bytes(bytes: [u8; 4]) -> Self {
        Self {
            y: bytes[0] as i16 - 16,
            x: bytes[1] as i16 - 8,
            tile: bytes[2],
            priority: bytes[3] >> 7 == 1,
            y_flip: bytes[3] >> 6 & 0b1 == 1,
            x_flip: bytes[3] >> 5 & 0b1 == 1,
            palette: (bytes[3] >> 4 & 0b1 == 1) as usize,
        }
    }
}

pub fn draw_sprites(mapper: &Mmu, line: u8, output: &mut [u8]) {
    let tiles = mapper.get_oam_tile_data();
    let offset = if mapper.get_obj_size() { 16 } else { 8 };
    let oam_table = mapper.get_oam();
    let mut tile_count = 0;
    let line = line as i16;
    let mut x_values = Vec::<i16>::new();

    for sprite in oam_table
        .chunks_exact(4)
        .map(|sprite| ObjectAttribute::from_bytes(sprite.try_into().unwrap()))
    {
        if sprite.y >= 144 || sprite.y == -16 {
            continue;
        }
        if line >= sprite.y + offset || line < sprite.y {
            continue;
        }
        if sprite.x >= 160 || sprite.x == -8 {
            tile_count += 1;
            continue;
        }

        if x_values.contains(&sprite.x) {
            tile_count += 1;
            continue;
        }

        let tile_line = match sprite.y_flip {
            true => (offset - (line - sprite.y) - 1) % offset,
            false => line - sprite.y,
        };

        let tile_start = match offset {
            8 => sprite.tile as usize * 16 + (tile_line as usize * 2),
            16 => (sprite.tile & 0xFE) as usize * 16 + (tile_line as usize * 2),
            _ => unreachable!(),
        };

        let tile_end = tile_start + 2;

        let tile = match sprite.x_flip {
            true => {
                let mut tile = [0u8; 2];
                for i in 0..8 {
                    tile[0] |= (tiles[tile_start] >> i & 0b1) << (7 - i);
                    tile[1] |= (tiles[tile_start + 1] >> i & 0b1) << (7 - i);
                }
                tile
            }
            false => tiles[tile_start..tile_end].try_into().unwrap(),
        };

        for x in 0..8 {
            if sprite.x + x >= 160 {
                continue;
            }
            if sprite.x + x < 0 {
                continue;
            }
            let start = (sprite.x as usize + x as usize) * 4;
            let end = start + 4;

            let color = ((tile[1] >> (7 - x) & 0b1) << 1) | (tile[0] >> (7 - x) & 0b1);

            if color != 0 {
                output[start..end].copy_from_slice(
                    match mapper.get_obj_palette(sprite.palette)[color as usize] {
                        Palette::White => &[232, 252, 204, 255],
                        Palette::LightGray => &[172, 212, 144, 255],
                        Palette::DarkGray => &[84, 140, 112, 255],
                        Palette::Black => &[20, 44, 56, 255],
                    },
                );
                if sprite.priority {
                    output[start + 3] = 128;
                }
            }
        }

        tile_count += 1;
        x_values.push(sprite.x);
        if tile_count >= 10 {
            break;
        }
    }
}

pub fn draw_window(mapper: &Mmu, line: u8, output: &mut [u8]) {
    let tiles = mapper.get_bg_tile_data();
    let tilemap = mapper.get_window_tile_map();
    let (win_y, win_x) = mapper.get_window_pos();

    if line < win_y {
        return;
    }

    let y = mapper.get_window_counter();

    for (index, pixel) in output.chunks_exact_mut(4).enumerate() {
        if index < win_x as usize - 7 {
            continue;
        }
        let x = index - win_x as usize + 7;
        let start = (y as usize / 8) * 32 + (x / 8);
        let start = tilemap[start] as usize;
        let tile = match mapper.get_tile_mode() {
            true => &tiles[start * 16..start * 16 + 16],
            false => {
                let start = start as i8 as i16;
                let start = (start * 16 + 0x800) as usize;
                &tiles[start..start + 16]
            }
        };
        let y = y % 8;
        let x = x % 8;
        let z = ((tile[y as usize * 2 + 1] >> (7 - x) & 0b1) << 1)
            | (tile[y as usize * 2] >> (7 - x) & 0b1);

        pixel.copy_from_slice(match mapper.get_bg_palette()[z as usize] {
            Palette::White => &[232, 252, 204, 255],
            Palette::LightGray => &[172, 212, 144, 255],
            Palette::DarkGray => &[84, 140, 112, 255],
            Palette::Black => &[20, 44, 56, 255],
        });
    }
}

pub fn draw_scanline(mapper: &Mmu, frame: &mut [u8], scx: u8, scy: u8, line: u8) {
    let tiles = mapper.get_bg_tile_data();
    let tilemap = mapper.get_bg_tile_map();
    let sprites = &mut [0u8; 160 * 4];
    let window = &mut [0u8; 160 * 4];

    let start = line as usize * 160 * 4;
    let end = start + 160 * 4;

    if mapper.get_obj_enable() {
        draw_sprites(mapper, line, sprites);
    }
    if mapper.get_window_enable() {
        draw_window(mapper, line, window);
    }

    let sprites = sprites.chunks_exact(4);
    let window = window.chunks_exact(4);

    for (real_idx, ((pixel, sprite), win)) in frame[start..end]
        .chunks_exact_mut(4)
        .zip(sprites)
        .zip(window)
        .enumerate()
    {
        if sprite.iter().any(|x| *x != 0) {
            pixel.copy_from_slice(sprite);
            if pixel[3] == 255 {
                continue;
            }
        }

        if !mapper.get_bg_enable() {
            pixel.copy_from_slice(&[232, 252, 204, 255]);
            continue;
        }

        if win.iter().sum::<u8>() != 0 {
            match pixel[3] {
                0 => {
                    pixel.copy_from_slice(win);
                    continue;
                }
                128 => {
                    if win[0] != 232 {
                        pixel.copy_from_slice(win);
                        continue;
                    }
                }
                _ => {
                    pixel.copy_from_slice(win);
                    continue;
                }
            };
        }

        let real_idx = real_idx + (start / 4);
        let idx =
            (real_idx as u16 % 160 + scx as u16) + ((real_idx as u16 / 160 + scy as u16) * 256);
        let y = idx / 256;
        let x = idx % 256;
        let tilenum = ((y / 8) * 32 + x / 8) as usize;
        let tile = tilemap[tilenum];
        let tile = match mapper.get_tile_mode() {
            true => &tiles[tile as usize * 16..tile as usize * 16 + 16],
            false => {
                let tile = tile as i8 as i16;
                let tile = (tile * 16 + 0x800) as usize;
                &tiles[tile..tile + 16]
            }
        };
        let y = y % 8;
        let x = x % 8;
        let z = ((tile[y as usize * 2 + 1] >> (7 - x) & 0b1) << 1)
            | (tile[y as usize * 2] >> (7 - x) & 0b1);
        if z == 0 && pixel[3] == 128 {
            pixel[3] = 255;
            continue;
        }
        pixel.copy_from_slice(match mapper.get_bg_palette()[z as usize] {
            Palette::White => &[232, 252, 204, 255],
            Palette::LightGray => &[172, 212, 144, 255],
            Palette::DarkGray => &[84, 140, 112, 255],
            Palette::Black => &[20, 44, 56, 255],
        });
    }
}
