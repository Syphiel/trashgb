#![allow(dead_code)]

use crate::mmu::Mmu;

#[derive(Debug)]
pub struct Ppu {
    pub frame_buffer: [u8; 160 * 144 * 4],
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            frame_buffer: [0; 160 * 144 * 4],
        }
    }

    // pub fn render_frame(&mut self, frame: &mut [u8]) {
    //     frame.copy_from_slice(&self.frame_buffer);
    // }
}

#[derive(Debug)]
pub struct ObjectAttribute {
    pub y: i16,
    pub x: i16,
    pub tile: u8,
    pub flags: u8,
}

pub fn draw_sprites(mapper: &Mmu, tiles: &[u8], line: u8, output: &mut [u8]) {
    let oam_table = mapper.get_oam();
    let mut tile_count = 0;
    let line = line as i16;

    for sprite in oam_table.chunks_exact(4).map(|sprite| ObjectAttribute {
        y: sprite[0] as i16 - 16,
        x: sprite[1] as i16 - 8,
        tile: sprite[2],
        flags: sprite[3],
    }) {
        if sprite.y >= 144 || sprite.y == 0 || sprite.x >= 160 || sprite.x == 0 {
            continue;
        }
        if line > sprite.y + 8 || line < sprite.y {
            continue;
        }

        let tile_line = line - sprite.y;
        let tile_start = sprite.tile as usize * 16 + (tile_line as usize * 2);
        let tile_end = tile_start + 2;
        let tile = &tiles[tile_start..tile_end];

        for x in 0..8 {
            if sprite.x + x >= 160 {
                break;
            }
            if sprite.x + x < 0 {
                continue;
            }
            let start = (sprite.x as usize + x as usize) * 4;
            let end = start + 4;

            output[start..end].copy_from_slice(
                match (tile[0] >> (7 - x) & 0b1) << 1 | (tile[1] >> (7 - x) & 0b1) {
                    0 => &[255, 255, 255, 255],
                    1 => &[192, 192, 192, 255],
                    2 => &[96, 96, 96, 255],
                    3 => &[0, 0, 0, 255],
                    _ => unreachable!(),
                },
            );
        }

        tile_count += 1;
        if tile_count > 10 {
            break;
        }
    }
}

pub fn draw_scanline(mapper: &Mmu, frame: &mut [u8], scx: u8, scy: u8, line: u8) {
    let tiles = mapper.get_tile_data();
    let tilemap = mapper.get_tile_map();
    let sprites = &mut [0u8; 160 * 4];

    let start = line as usize * 160 * 4;
    let end = start + 160 * 4;

    // TODO: Check if objects enabled
    draw_sprites(&mapper, tiles, line, sprites);
    let sprites = sprites.chunks_exact(4);

    for (real_idx, (pixel, sprite)) in frame[start..end].chunks_exact_mut(4).zip(sprites).enumerate() {
        if sprite.iter().sum::<u8>() != 0 {
            pixel.copy_from_slice(sprite);
            continue;
        }
        let real_idx = real_idx + (start / 4);
        let idx =
            (real_idx as u16 % 160 + scx as u16) + ((real_idx as u16 / 160 + scy as u16) * 256);
        let y = idx / 256;
        let x = idx % 256;
        let tile = tilemap[((y / 8) * 32 + x / 8) as usize] as usize * 16;
        let tile = &tiles[tile..tile + 16];
        let y = y % 8;
        let x = x % 8;
        let z = ((tile[y as usize * 2] >> (7 - x) & 0b1) << 1)
            | (tile[y as usize * 2 + 1] >> (7 - x) & 0b1);
        pixel.copy_from_slice(match z {
            0 => &[255, 255, 255, 255],
            1 => &[192, 192, 192, 255],
            2 => &[96, 96, 96, 255],
            3 => &[0, 0, 0, 255],
            _ => unreachable!(),
        });
    }
}
