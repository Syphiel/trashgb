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

pub fn print_tiles(memory: &[u8]) {
    let mut i = 0;
    while i < memory.len() {
        let tile = &memory[i..i + 16];
        if tile.iter().max().unwrap() == &0 {
            i += 16;
            continue;
        }
        println!("{:04x}: ", 0x8000 + i);
        for j in (0..16).step_by(2) {
            for k in 0..8 {
                let z = (tile[j] >> (7 - k) & 0b1) << 1 | tile[j + 1] >> (7 - k) & 0b1;
                match z {
                    0 => print!("  "),
                    1 => print!("░░"),
                    2 => print!("▒▒"),
                    3 => print!("▓▓"),
                    _ => unreachable!(),
                }
            }
            println!();
        }
        println!();
        i += 16;
    }
}

pub fn draw_background(tiles: &[u8], tilemap: &[u8], frame: &mut [u8], scx: u8, scy: u8) {
    for (real_idx, pixel) in frame.chunks_exact_mut(4).enumerate() {
        let idx = (real_idx as u16 % 160 + scx as u16) + ((real_idx as u16 / 160 + scy as u16) * 256);
        let y = idx / 256;
        let x = idx % 256;
        let tile = tilemap[((y / 8) * 32 + x / 8) as usize] as usize * 16;
        let tile = &tiles[tile..tile + 16];
        let y = y % 8;
        let x = x % 8;
        let z = ((tile[y as usize * 2] >> (7 - x) & 0b1) << 1) | (tile[y as usize * 2 + 1] >> (7 - x) & 0b1);
        pixel.copy_from_slice(match z {
            0 => &[255, 255, 255, 255],
            1 => &[192, 192, 192, 255],
            2 => &[96, 96, 96, 255],
            3 => &[0, 0, 0, 255],
            _ => unreachable!(),
        });
    }
}
