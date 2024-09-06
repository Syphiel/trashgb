#![allow(dead_code)]

use crate::ppu::{self, print_tiles, Ppu};
use crate::registers::{Flags, R8OrMem, Registers, R16, R8};
use std::cell::Cell;
use std::time::Instant;

use crate::registers::{R16mem, R16stk};

#[derive(Debug)]
enum AfterInstruction {
    Increment,
    Decrement,
    None,
}

#[derive(Debug)]
pub struct Cpu {
    pub registers: Registers,
    pub memory: Vec<u8>,
    pub pc: u16,
    pub sp: u16,
    pub last_frame: Instant,
    pub ppu: Ppu,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            registers: Registers::new(),
            // memory: vec![0; 0xFFFF],
            memory: include_bytes!("../roms/bootstrap.gb").to_vec(),
            pc: 0,
            sp: 0,
            last_frame: Instant::now(),
            ppu: Ppu::new(),
        }
    }

    pub fn step(&mut self) -> u8 {
        let opcode = self.memory[self.pc as usize];

        if opcode == 0xCB {
            let opcode = self.memory[self.pc as usize + 1];
            match (opcode & 0b1100_0000) >> 6 {
                0b00 => {
                    let operand = R8::from_u8(opcode & 0b0000_0111);
                    let operand = self.registers.get_r8(operand);
                    let operand = match operand {
                        R8OrMem::Ptr(ptr) => R8OrMem::Mem(&mut self.memory[ptr]),
                        _ => operand,
                    };

                    match (opcode & 0b0011_1000) >> 3 {
                        0b000 => {
                            // println!("{:#04x}: rlc r8", self.pc);
                            rlc_r8(operand, &self.registers.flags);
                            self.pc += 2;
                            return 2;
                        }
                        0b001 => {
                            // println!("{:#04x}: rrc r8", self.pc);
                            rrc_r8(operand, &self.registers.flags);
                            self.pc += 2;
                            return 2;
                        }
                        0b010 => {
                            // println!("{:#04x}: rl r8", self.pc);
                            rl_r8(operand, &self.registers.flags);
                            self.pc += 2;
                            return 2;
                        }
                        0b011 => {
                            // println!("{:#04x}: rr r8", self.pc);
                            rr_r8(operand, &self.registers.flags);
                            self.pc += 2;
                            return 2;
                        }
                        0b100 => {
                            // println!("{:#04x}: sla r8", self.pc);
                            todo!()
                        }
                        0b101 => {
                            // println!("{:#04x}: sra r8", self.pc);
                            todo!()
                        }
                        0b110 => {
                            // println!("{:#04x}: swap r8", self.pc);
                            todo!()
                        }
                        0b111 => {
                            // println!("{:#04x}: srl r8", self.pc);
                            todo!()
                        }
                        _ => todo!("CB Instructions"),
                    }
                }
                0b01 => {
                    // println!("{:#04x}: bit b3, r8", self.pc);
                    let bit_index = (opcode & 0b0011_1000) >> 3;
                    let operand = R8::from_u8(opcode & 0b0000_0111);
                    let operand = self.registers.get_r8(operand);
                    let operand = match operand {
                        R8OrMem::Ptr(ptr) => R8OrMem::Mem(&mut self.memory[ptr]),
                        _ => operand,
                    };
                    bit_b3_r8(bit_index, operand, &self.registers.flags);
                    self.pc += 2;
                    return 2;
                }
                0b10 => {
                    // println!("{:#04x}: res b3, r8", self.pc);
                    let bit_index = (opcode & 0b0011_1000) >> 3;
                    let operand = R8::from_u8(opcode & 0b0000_0111);
                    let operand = self.registers.get_r8(operand);
                    let operand = match operand {
                        R8OrMem::Ptr(ptr) => R8OrMem::Mem(&mut self.memory[ptr]),
                        _ => operand,
                    };
                    res_b3_r8(bit_index, operand);
                    self.pc += 2;
                    return 2;
                }
                0b11 => {
                    // println!("{:#04x}: set b3, r8", self.pc);
                    let bit_index = (opcode & 0b0011_1000) >> 3;
                    let operand = R8::from_u8(opcode & 0b0000_0111);
                    let operand = self.registers.get_r8(operand);
                    let operand = match operand {
                        R8OrMem::Ptr(ptr) => R8OrMem::Mem(&mut self.memory[ptr]),
                        _ => operand,
                    };
                    set_b3_r8(bit_index, operand);
                    self.pc += 2;
                    return 2;
                }
                _ => unreachable!(),
            }
        }
        match (opcode & 0b1100_0000) >> 6 {
            0b00 => {
                /* Block 0 */
                if opcode == 0 {
                    // println!("{:#04x}: nop", self.pc);
                    self.pc += 1;
                    1
                } else if opcode == 24 {
                    // println!("{:#04x}: jr imm8", self.pc);
                    let imm8 = self.memory[self.pc as usize + 1] as i8;
                    self.pc = (self.pc as i16 + imm8 as i16) as u16;
                    self.pc += 2;
                    return 3;
                } else if opcode & 0b0010_0111 == 32 {
                    // println!("{:#04x}: jr cond, imm8", self.pc);
                    let condition = (opcode & 0b0001_1000) >> 3;
                    let condition = self.registers.flags.get_condition(condition);
                    if condition {
                        let imm8 = self.memory[self.pc as usize + 1] as i8;
                        self.pc = (self.pc as i16 + imm8 as i16) as u16;
                        self.pc += 2;
                        return 3;
                    }
                    self.pc += 2;
                    return 2;
                } else {
                    match opcode & 0b0000_1111 {
                        0b0001 => {
                            // println!("{:#04x}: ld r16, imm16", self.pc);
                            let imm16 = self.memory[self.pc as usize + 1] as u16
                                | (self.memory[self.pc as usize + 2] as u16) << 8;
                            let dest = R16::from_u8((opcode & 0b0011_0000) >> 4);
                            match dest {
                                R16::SP => self.sp = imm16,
                                _ => {
                                    let dest = self.registers.get_r16(dest);
                                    ld_r16_imm16(dest, imm16);
                                }
                            }
                            self.pc += 3;
                            return 3;
                        }
                        0b0010 => {
                            // println!("{:#04x}: ld [r16mem], a", self.pc);
                            let dest = R16mem::from_u8((opcode & 0b0011_0000) >> 4);
                            let action = match dest {
                                R16mem::HLi => AfterInstruction::Increment,
                                R16mem::HLd => AfterInstruction::Decrement,
                                _ => AfterInstruction::None,
                            };
                            let dest = self.registers.get_r16mem(dest);
                            let dest = &mut self.memory
                                [dest.1.get() as usize | (dest.0.get() as usize) << 8];
                            ld_r16mem_a(dest, &self.registers.a);

                            match action {
                                AfterInstruction::Increment => {
                                    inc_r16((&self.registers.h, &self.registers.l));
                                }
                                AfterInstruction::Decrement => {
                                    dec_r16((&self.registers.h, &self.registers.l));
                                }
                                AfterInstruction::None => {}
                            }
                            self.pc += 1;
                            return 2;
                        }
                        0b1010 => {
                            // println!("{:#04x}: ld a, [r16mem]", self.pc);
                            let source = R16mem::from_u8((opcode & 0b0011_0000) >> 4);
                            let action = match source {
                                R16mem::HLi => AfterInstruction::Increment,
                                R16mem::HLd => AfterInstruction::Decrement,
                                _ => AfterInstruction::None,
                            };
                            let source = self.registers.get_r16mem(source);
                            let source = self.memory
                                [source.1.get() as usize | (source.0.get() as usize) << 8];
                            ld_a_r16mem(&self.registers.a, source);

                            match action {
                                AfterInstruction::Increment => {
                                    inc_r16((&self.registers.h, &self.registers.l))
                                }
                                AfterInstruction::Decrement => {
                                    dec_r16((&self.registers.h, &self.registers.l))
                                }
                                AfterInstruction::None => {}
                            }
                            self.pc += 1;
                            return 2;
                        }
                        0b1000 => {
                            // println!("{:#04x}: ld [imm16], sp", self.pc);
                            let imm16 = self.memory[self.pc as usize + 1] as u16
                                | (self.memory[self.pc as usize + 2] as u16) << 8;
                            let imm16 = &mut self.memory[imm16 as usize..imm16 as usize + 1];
                            ld_imm16_sp(imm16, self.sp);
                            self.pc += 3;
                            return 5;
                        }
                        0b0011 => {
                            // println!("{:#04x}: inc r16", self.pc);
                            let operand = R16::from_u8((opcode & 0b0011_0000) >> 4);
                            let operand = self.registers.get_r16(operand);
                            inc_r16(operand);
                            self.pc += 1;
                            return 2;
                        }
                        0b1011 => {
                            // println!("{:#04x}: dec r16", self.pc);
                            let operand = R16::from_u8((opcode & 0b0011_0000) >> 4);
                            let operand = self.registers.get_r16(operand);
                            dec_r16(operand);
                            self.pc += 1;
                            return 2;
                        }
                        0b1001 => {
                            // println!("{:#04x}: add hl, r16", self.pc);
                            let operand = R16::from_u8((opcode & 0b0011_0000) >> 4);
                            let operand = self.registers.get_r16(operand);
                            add_hl_r16(
                                (&self.registers.h, &self.registers.l),
                                operand,
                                &self.registers.flags,
                            );
                            self.pc += 1;
                            return 2;
                        }
                        0b0100 | 0b1100 => {
                            // println!("{:#04x}: inc r8", self.pc);
                            let operand = R8::from_u8((opcode & 0b0011_1000) >> 3);
                            let operand = self.registers.get_r8(operand);
                            let operand = match operand {
                                R8OrMem::Ptr(ptr) => R8OrMem::Mem(&mut self.memory[ptr]),
                                _ => operand,
                            };
                            inc_r8(operand, &self.registers.flags);
                            self.pc += 1;
                            return 1;
                        }
                        0b0101 | 0b1101 => {
                            // println!("{:#04x}: dec r8", self.pc);
                            let operand = R8::from_u8((opcode & 0b0011_1000) >> 3);
                            let operand = self.registers.get_r8(operand);
                            let operand = match operand {
                                R8OrMem::Ptr(ptr) => R8OrMem::Mem(&mut self.memory[ptr]),
                                _ => operand,
                            };
                            dec_r8(operand, &self.registers.flags);
                            self.pc += 1;
                            return 1;
                        }
                        0b1110 | 0b0110 => {
                            // println!("{:#04x}: ld r8, imm8", self.pc);
                            let imm8 = self.memory[self.pc as usize + 1];
                            let operand = R8::from_u8((opcode & 0b0011_1000) >> 3);
                            let operand = self.registers.get_r8(operand);
                            let operand = match operand {
                                R8OrMem::Ptr(ptr) => R8OrMem::Mem(&mut self.memory[ptr]),
                                _ => operand,
                            };
                            ld_r8_imm8(operand, imm8);
                            self.pc += 2;
                            return 2;
                        }
                        0b0111 | 0b1111 => match (opcode & 0b0011_1000) >> 3 {
                            0b000 => {
                                // println!("{:#04x}: rlca", self.pc);
                                rlc_r8(R8OrMem::R8(&self.registers.a), &self.registers.flags);
                                self.pc += 1;
                                return 1;
                            }
                            0b001 => {
                                // println!("{:#04x}: rrca", self.pc);
                                rrc_r8(R8OrMem::R8(&self.registers.a), &self.registers.flags);
                                self.pc += 1;
                                return 1;
                            }
                            0b010 => {
                                // println!("{:#04x}: rla", self.pc);
                                rl_r8(R8OrMem::R8(&self.registers.a), &self.registers.flags);
                                self.pc += 1;
                                return 1;
                            }
                            0b011 => {
                                // println!("{:#04x}: rra", self.pc);
                                rr_r8(R8OrMem::R8(&self.registers.a), &self.registers.flags);
                                self.pc += 1;
                                return 1;
                            }
                            0b100 => {
                                // println!("{:#04x}: daa", self.pc);
                                todo!()
                            }
                            0b101 => {
                                // println!("{:#04x}: cpl", self.pc);
                                todo!()
                            }
                            0b110 => {
                                // println!("{:#04x}: scf", self.pc);
                                todo!()
                            }
                            0b111 => {
                                // println!("{:#04x}: ccf", self.pc);
                                todo!()
                            }
                            _ => unreachable!(),
                        },
                        _ => {
                            panic!("{:#04x}: Unknown opcode {}", self.pc, opcode);
                        }
                    }
                }
            }
            0b01 => {
                /* Block 1 */
                // println!("{:#04x}: ld r8, r8", self.pc);

                let dest = R8::from_u8((opcode & 0b0011_1000) >> 3);
                let dest = self.registers.get_r8(dest);

                let source = R8::from_u8(opcode & 0b0000_0111);
                let source = self.registers.get_r8(source);

                let (source, dest) = match (&source, &dest) {
                    (R8OrMem::Ptr(ptr), R8OrMem::R8(_)) => {
                        (R8OrMem::Mem(&mut self.memory[*ptr]), dest)
                    }
                    (R8OrMem::R8(_), R8OrMem::Ptr(ptr)) => {
                        (source, R8OrMem::Mem(&mut self.memory[*ptr]))
                    }
                    _ => (source, dest),
                };

                ld_r8_r8(dest, source);

                self.pc += 1;
                1
            }
            0b10 => {
                /* Block 2 */
                let operand = R8::from_u8(opcode & 0b0000_0111);

                let a = &self.registers.a;
                let r8 = self.registers.get_r8(operand);
                let r8 = match r8 {
                    R8OrMem::Ptr(ptr) => R8OrMem::Mem(&mut self.memory[ptr]),
                    _ => r8,
                };

                match (opcode & 0b0011_1000) >> 3 {
                    0b0000 => {
                        // println!("{:#04x}: add a, r8", self.pc);
                        add_a_r8(a, r8, &self.registers.flags);
                        self.pc += 1;
                        1
                    }
                    0b0001 => {
                        // println!("{:#04x}: adc a, r8", self.pc);
                        adc_a_r8(a, r8, &self.registers.flags);
                        self.pc += 1;
                        1
                    }
                    0b0010 => {
                        // println!("{:#04x}: sub a, r8", self.pc);
                        sub_a_r8(a, r8, &self.registers.flags);
                        self.pc += 1;
                        1
                    }
                    0b0011 => {
                        // println!("{:#04x}: sbc a, r8", self.pc);
                        sbc_a_r8(a, r8, &self.registers.flags);
                        self.pc += 1;
                        1
                    }
                    0b0100 => {
                        // println!("{:#04x}: and a, r8", self.pc);
                        and_a_r8(a, r8, &self.registers.flags);
                        self.pc += 1;
                        1
                    }
                    0b0101 => {
                        // println!("{:#04x}: xor a, r8", self.pc);
                        xor_a_r8(a, r8, &self.registers.flags);
                        self.pc += 1;
                        1
                    }
                    0b0110 => {
                        // println!("{:#04x}: or a, r8", self.pc);
                        or_a_r8(a, r8, &self.registers.flags);
                        self.pc += 1;
                        1
                    }
                    0b0111 => {
                        // println!("{:#04x}: cp a, r8", self.pc);
                        cp_a_r8(a, r8, &self.registers.flags);
                        self.pc += 1;
                        1
                    }
                    _ => {
                        // println!("{:#04x}: Unknown opcode {}", self.pc, opcode);
                        unreachable!()
                    }
                }
            }
            0b11 => {
                /* Block 3 */
                if opcode & 0b0000_0111 == 0b110 {
                    let imm8 = self.memory[self.pc as usize + 1];
                    let a = &self.registers.a;
                    match (opcode & 0b0011_1000) >> 3 {
                        0b000 => {
                            // println!("{:#04x}: add a, imm8", self.pc);
                            add_a_imm8(a, imm8, &self.registers.flags);
                            self.pc += 2;
                            return 2;
                        }
                        0b001 => {
                            // println!("{:#04x}: adc a, imm8", self.pc);
                            adc_a_imm8(a, imm8, &self.registers.flags);
                            self.pc += 2;
                            return 2;
                        }
                        0b010 => {
                            // println!("{:#04x}: sub a, imm8", self.pc);
                            sub_a_imm8(a, imm8, &self.registers.flags);
                            self.pc += 2;
                            return 2;
                        }
                        0b011 => {
                            // println!("{:#04x}: sbc a, imm8", self.pc);
                            sbc_a_imm8(a, imm8, &self.registers.flags);
                            self.pc += 2;
                            return 2;
                        }
                        0b100 => {
                            // println!("{:#04x}: and a, imm8", self.pc);
                            and_a_imm8(a, imm8, &self.registers.flags);
                            self.pc += 2;
                            return 2;
                        }
                        0b101 => {
                            // println!("{:#04x}: xor a, imm8", self.pc);
                            xor_a_imm8(a, imm8, &self.registers.flags);
                            self.pc += 2;
                            return 2;
                        }
                        0b110 => {
                            // println!("{:#04x}: or a, imm8", self.pc);
                            or_a_imm8(a, imm8, &self.registers.flags);
                            self.pc += 2;
                            return 2;
                        }
                        0b111 => {
                            // println!("{:#04x}: cp a, imm8", self.pc);
                            cp_a_imm8(a, imm8, &self.registers.flags);
                            self.pc += 2;
                            return 2;
                        }
                        _ => {
                            unreachable!();
                        }
                    }
                }

                match opcode {
                    0b1110_0010 => {
                        // println!("{:#04x}: ld (c), a", self.pc);
                        let a = self.registers.a.get();
                        let c = self.registers.c.get();
                        let c = &mut self.memory[0xFF00 + c as usize];
                        ld_c_a(c, a);
                        self.pc += 1;
                        return 2;
                    }
                    0b1111_0010 => {
                        // println!("{:#04x}: ld a, (c)", self.pc);
                        let a = &self.registers.a;
                        let c = self.registers.c.get();
                        let c = self.memory[0xFF00 + c as usize];
                        ld_a_c(a, c);
                        self.pc += 1;
                        return 2;
                    }
                    0b1110_0000 => {
                        // println!("{:#04x}: ldh [imm8], a", self.pc);
                        let imm8 = self.memory[self.pc as usize + 1];
                        let imm8 = &mut self.memory[0xFF00 + imm8 as usize];
                        ldh_imm8_a(imm8, &self.registers.a);
                        self.pc += 2;
                        return 3;
                    }
                    0b1111_0000 => {
                        // println!("{:#04x}: ldh a, [imm8]", self.pc);
                        let imm8 = self.memory[self.pc as usize + 1];
                        let imm8 = self.memory[0xFF00 + imm8 as usize];
                        ldh_a_imm8(&self.registers.a, imm8);
                        self.pc += 2;
                        return 3;
                    }
                    0b1110_1010 => {
                        // println!("{:#04x}: ld [imm16], a", self.pc);
                        let imm16 = self.memory[self.pc as usize + 1] as u16
                            | (self.memory[self.pc as usize + 2] as u16) << 8;
                        let imm16 = &mut self.memory[imm16 as usize];
                        ld_imm16_a(imm16, &self.registers.a);
                        self.pc += 3;
                        return 4;
                    }
                    0b1100_1101 => {
                        // println!("{:#04x}: call imm16", self.pc);
                        self.memory[self.sp as usize - 1] = (self.pc + 3) as u8;
                        self.memory[self.sp as usize - 2] = ((self.pc + 3) >> 8) as u8;
                        self.sp -= 2;
                        self.pc = self.memory[self.pc as usize + 1] as u16
                            | (self.memory[self.pc as usize + 2] as u16) << 8;
                        self.pc += 0;
                        return 6;
                    }
                    0b1100_1001 => {
                        // println!("{:#04x}: ret", self.pc);
                        self.pc = self.memory[self.sp as usize + 1] as u16
                            | (self.memory[self.sp as usize] as u16) << 8;
                        // println!("{:#04x}", self.pc);
                        self.sp += 2;
                        return 4;
                    }
                    _ => {}
                }

                match opcode & 0b0000_1111 {
                    0b0001 => {
                        // println!("{:#04x}: pop r16stk", self.pc);
                        let register = R16stk::from_u8((opcode & 0b0011_0000) >> 4);
                        let register = self.registers.get_r16stk(register);
                        let lo = self.memory[self.sp as usize];
                        let hi = self.memory[self.sp as usize + 1];
                        ld_r16_imm16(register, (hi as u16) << 8 | lo as u16);
                        self.sp += 2;
                        self.pc += 1;
                        return 4;
                    }
                    0b0101 => {
                        // println!("{:#04x}: push r16stk", self.pc);
                        let register = R16stk::from_u8((opcode & 0b0011_0000) >> 4);
                        let register = self.registers.get_r16stk(register);
                        let lo = register.1.get();
                        let hi = register.0.get();
                        self.memory[self.sp as usize - 1] = hi;
                        self.memory[self.sp as usize - 2] = lo;
                        self.sp -= 2;
                        self.pc += 1;
                        return 3;
                    }
                    _ => {}
                }
                todo!("{:#04x}: Block 3 - Opcode {:#04x}", self.pc, opcode);
            }
            _ => unreachable!(),
        }
    }

    pub fn game_loop(&mut self, frame: &mut [u8]) -> bool {
        if self.last_frame.elapsed().as_millis() < 16 {
            return false;
        }

        if self.pc < self.memory.len() as u16 && self.memory[self.pc as usize] != 0x00 {
            let mut ticks = 0;
            for line in 0..154 {
                while ticks < 456 {
                    if self.memory[self.pc as usize] == 0x00 {
                        return false;
                    }
                    ticks += self.step() as u32;
                }
                ticks = 0;
                if line < 144 {
                    ppu::draw_scanline(
                        &self.memory[0x8000..0x9000],
                        &self.memory[0x9800..0x9C00],
                        frame,
                        self.memory[0xFF43],
                        self.memory[0xFF42],
                        line,
                    );
                }
                self.memory[0xFF44] = line as u8;
            }
        }
        self.last_frame = Instant::now();
        true
    }

    pub fn render_frame(&mut self, frame: &mut [u8]) -> bool {
        // print_tiles(&self.memory[0x8000..0x9000]);
        if self.last_frame.elapsed().as_millis() >= 16 {
            ppu::draw_background(
                &self.memory[0x8000..0x9000],
                &self.memory[0x9800..0x9C00],
                frame,
                220,
                0,
            );
            self.last_frame = Instant::now();
            return true;
        }
        false
    }

    pub fn execute(&mut self) {
        // println!("{:?}", self.memory[0x95]);
        loop {
            self.step();
            if self.pc >= self.memory.len() as u16 || self.memory[self.pc as usize] == 0xf0 {
                // println!("{:#04x}: End of program", self.pc);
                print_tiles(&self.memory[0x8000..0x9000]);
                // ppu::test();
                // println!("map");
                // print_tiles(&self.memory[0x9800..0x9C00]);
                // println!("{}", self.memory.len());
                // println!("{}", self.memory[self.pc as usize]);
                // println!("{:?}", self.memory[0x95]);
                break;
            }
        }
    }
}

fn add_a_r8(a: &Cell<u8>, r8: R8OrMem, flags: &Flags) {
    let r8 = match r8 {
        R8OrMem::R8(r8) => r8.get(),
        R8OrMem::Mem(r8) => *r8,
        R8OrMem::Ptr(_) => panic!("Pointer not supported"),
    };

    let (result, overflow) = a.get().overflowing_add(r8);
    flags.zero.set(result == 0);
    flags.carry.set(overflow);
    flags.subtract.set(false);
    flags.half_carry.set((a.get() & 0xF) + (r8 & 0xF) > 0xF);
    a.set(result);
}

fn adc_a_r8(a: &Cell<u8>, r8: R8OrMem, flags: &Flags) {
    let r8 = match r8 {
        R8OrMem::R8(r8) => r8.get(),
        R8OrMem::Mem(r8) => *r8,
        R8OrMem::Ptr(_) => panic!("Pointer not supported"),
    };

    let (result, overflow) = if flags.carry.get() {
        flags.half_carry.set((a.get() & 0xF) + (r8 & 0xF) > 0xF);
        a.get().overflowing_add(r8)
    } else {
        flags.half_carry.set((a.get() & 0xF) + (r8 & 0xF) + 1 > 0xF);
        a.get().overflowing_add(r8 + 1)
    };
    flags.zero.set(result == 0);
    flags.carry.set(overflow);
    flags.subtract.set(false);

    a.set(result);
}

fn sub_a_r8(a: &Cell<u8>, r8: R8OrMem, flags: &Flags) {
    let r8 = match r8 {
        R8OrMem::R8(r8) => r8.get(),
        R8OrMem::Mem(r8) => *r8,
        R8OrMem::Ptr(_) => panic!("Pointer not supported"),
    };

    let (result, overflow) = a.get().overflowing_sub(r8);
    flags.zero.set(result == 0);
    flags.carry.set(overflow);
    flags.subtract.set(true);
    flags.half_carry.set((a.get() & 0xF) < (r8 & 0xF));
    a.set(result);
}

fn sbc_a_r8(a: &Cell<u8>, r8: R8OrMem, flags: &Flags) {
    let r8 = match r8 {
        R8OrMem::R8(r8) => r8.get(),
        R8OrMem::Mem(r8) => *r8,
        R8OrMem::Ptr(_) => panic!("Pointer not supported"),
    };

    let (result, overflow) = if flags.carry.get() {
        flags.half_carry.set((a.get() & 0xF) < (r8 & 0xF));
        a.get().overflowing_sub(r8)
    } else {
        flags.half_carry.set((a.get() & 0xF) - 1 < (r8 & 0xF));
        a.get().overflowing_sub(r8 - 1)
    };
    flags.zero.set(result == 0);
    flags.carry.set(overflow);
    flags.subtract.set(true);

    a.set(result);
}

fn and_a_r8(a: &Cell<u8>, r8: R8OrMem, flags: &Flags) {
    let r8 = match r8 {
        R8OrMem::R8(r8) => r8.get(),
        R8OrMem::Mem(r8) => *r8,
        R8OrMem::Ptr(_) => panic!("Pointer not supported"),
    };

    let result = a.get() & r8;
    flags.zero.set(result == 0);
    flags.carry.set(false);
    flags.subtract.set(false);
    flags.half_carry.set(true);
    a.set(result);
}

fn xor_a_r8(a: &Cell<u8>, r8: R8OrMem, flags: &Flags) {
    let r8 = match r8 {
        R8OrMem::R8(r8) => r8.get(),
        R8OrMem::Mem(r8) => *r8,
        R8OrMem::Ptr(_) => panic!("Pointer not supported"),
    };

    let result = a.get() ^ r8;
    flags.zero.set(result == 0);
    flags.carry.set(false);
    flags.subtract.set(false);
    flags.half_carry.set(false);
    a.set(result);
}

fn or_a_r8(a: &Cell<u8>, r8: R8OrMem, flags: &Flags) {
    let r8 = match r8 {
        R8OrMem::R8(r8) => r8.get(),
        R8OrMem::Mem(r8) => *r8,
        R8OrMem::Ptr(_) => panic!("Pointer not supported"),
    };

    let result = a.get() | r8;
    flags.zero.set(result == 0);
    flags.carry.set(false);
    flags.subtract.set(false);
    flags.half_carry.set(false);
    a.set(result);
}

fn cp_a_r8(a: &Cell<u8>, r8: R8OrMem, flags: &Flags) {
    let r8 = match r8 {
        R8OrMem::R8(r8) => r8.get(),
        R8OrMem::Mem(r8) => *r8,
        R8OrMem::Ptr(_) => panic!("Pointer not supported"),
    };

    let (result, overflow) = a.get().overflowing_sub(r8);
    flags.zero.set(result == 0);
    flags.carry.set(overflow);
    flags.subtract.set(true);
    flags.half_carry.set((a.get() & 0xF) < (r8 & 0xF));
}

fn ld_r8_r8(dest: R8OrMem, source: R8OrMem) {
    let source = match source {
        R8OrMem::R8(source) => source.get(),
        R8OrMem::Mem(source) => *source,
        R8OrMem::Ptr(_) => panic!("Pointer not supported"),
    };

    match dest {
        R8OrMem::R8(dest) => dest.set(source),
        R8OrMem::Mem(dest) => *dest = source,
        R8OrMem::Ptr(_) => panic!("Pointer not supported"),
    }
}

fn ld_r16_imm16((hi, lo): (&Cell<u8>, &Cell<u8>), imm16: u16) {
    hi.set((imm16 >> 8) as u8);
    lo.set(imm16 as u8);
}

fn ld_r16mem_a(dest: &mut u8, a: &Cell<u8>) {
    *dest = a.get();
}

fn ld_a_r16mem(a: &Cell<u8>, source: u8) {
    a.set(source);
}

fn ld_imm16_sp(imm16: &mut [u8], sp: u16) {
    imm16[0] = sp as u8;
    imm16[1] = (sp >> 8) as u8;
}

fn inc_r16((hi, lo): (&Cell<u8>, &Cell<u8>)) {
    let (result, overflow) = lo.get().overflowing_add(1);
    lo.set(result);
    if overflow {
        hi.set(hi.get().wrapping_add(1));
    }
}

fn dec_r16((hi, lo): (&Cell<u8>, &Cell<u8>)) {
    let (result, overflow) = lo.get().overflowing_sub(1);
    lo.set(result);
    if overflow {
        hi.set(hi.get().wrapping_sub(1));
    }
}

fn add_hl_r16((h, l): (&Cell<u8>, &Cell<u8>), (hi, lo): (&Cell<u8>, &Cell<u8>), flags: &Flags) {
    let hl = (h.get() as u16) << 8 | l.get() as u16;
    let r16 = (hi.get() as u16) << 8 | lo.get() as u16;

    let (result, overflow) = hl.overflowing_add(r16);
    h.set((result >> 8) as u8);
    l.set(result as u8);

    flags.half_carry.set((hl & 0xFF) + (r16 & 0xFF) > 0xFF);
    flags.carry.set(overflow);
}

fn inc_r8(r8: R8OrMem, flags: &Flags) {
    flags.subtract.set(false);

    match r8 {
        R8OrMem::R8(r8) => {
            let (result, _) = r8.get().overflowing_add(1);
            flags.zero.set(result == 0);
            flags.half_carry.set((r8.get() & 0xF) + 1 > 0xF);
            r8.set(result);
        }
        R8OrMem::Mem(r8) => {
            let (result, _) = (*r8).overflowing_add(1);
            flags.zero.set(result == 0);
            flags.half_carry.set((*r8 & 0xF) + 1 > 0xF);
            *r8 = result;
        }
        R8OrMem::Ptr(_) => panic!("Pointer not supported"),
    };
}

fn dec_r8(r8: R8OrMem, flags: &Flags) {
    flags.subtract.set(true);

    match r8 {
        R8OrMem::R8(r8) => {
            let (result, _) = r8.get().overflowing_sub(1);
            flags.zero.set(result == 0);
            flags.half_carry.set((r8.get() & 0xF) < 1);
            r8.set(result);
        }
        R8OrMem::Mem(r8) => {
            let (result, _) = (*r8).overflowing_sub(1);
            flags.zero.set(result == 0);
            flags.half_carry.set((*r8 & 0xF) < 1);
            *r8 = result;
        }
        R8OrMem::Ptr(_) => panic!("Pointer not supported"),
    };
}

fn bit_b3_r8(bit_index: u8, r8: R8OrMem, flags: &Flags) {
    let r8 = match r8 {
        R8OrMem::R8(r8) => r8.get(),
        R8OrMem::Mem(r8) => *r8,
        R8OrMem::Ptr(_) => panic!("Pointer not supported"),
    };

    let bit = (r8 >> bit_index) & 1;
    flags.zero.set(bit == 0);
    flags.subtract.set(false);
    flags.half_carry.set(true);
}

fn res_b3_r8(bit_index: u8, r8: R8OrMem) {
    let mask = !(1 << bit_index);

    match r8 {
        R8OrMem::R8(r8) => {
            let result = r8.get() & mask;
            r8.set(result);
        }
        R8OrMem::Mem(mem) => {
            let result = *mem & mask;
            *mem = result;
        }
        R8OrMem::Ptr(_) => panic!("Pointer not supported"),
    }
}

fn set_b3_r8(bit_index: u8, r8: R8OrMem) {
    let mask = 1 << bit_index;

    match r8 {
        R8OrMem::R8(r8) => {
            let result = r8.get() | mask;
            r8.set(result);
        }
        R8OrMem::Mem(mem) => {
            let result = *mem | mask;
            *mem = result;
        }
        R8OrMem::Ptr(_) => panic!("Pointer not supported"),
    }
}

fn ld_r8_imm8(r8: R8OrMem, imm8: u8) {
    match r8 {
        R8OrMem::R8(r8) => r8.set(imm8),
        R8OrMem::Mem(mem) => *mem = imm8,
        R8OrMem::Ptr(_) => panic!("Pointer not supported"),
    }
}

fn add_a_imm8(a: &Cell<u8>, imm8: u8, flags: &Flags) {
    let (result, overflow) = a.get().overflowing_add(imm8);
    flags.zero.set(result == 0);
    flags.carry.set(overflow);
    flags.subtract.set(false);
    flags.half_carry.set((a.get() & 0xF) + (imm8 & 0xF) > 0xF);
    a.set(result);
}

fn adc_a_imm8(a: &Cell<u8>, imm8: u8, flags: &Flags) {
    let (result, overflow) = if flags.carry.get() {
        flags.half_carry.set((a.get() & 0xF) + (imm8 & 0xF) > 0xF);
        a.get().overflowing_add(imm8)
    } else {
        flags
            .half_carry
            .set((a.get() & 0xF) + (imm8 & 0xF) + 1 > 0xF);
        a.get().overflowing_add(imm8 + 1)
    };
    flags.zero.set(result == 0);
    flags.carry.set(overflow);
    flags.subtract.set(false);

    a.set(result);
}

fn sub_a_imm8(a: &Cell<u8>, imm8: u8, flags: &Flags) {
    let (result, overflow) = a.get().overflowing_sub(imm8);
    flags.zero.set(result == 0);
    flags.carry.set(overflow);
    flags.subtract.set(true);
    flags.half_carry.set((a.get() & 0xF) < (imm8 & 0xF));
    a.set(result);
}

fn sbc_a_imm8(a: &Cell<u8>, imm8: u8, flags: &Flags) {
    let (result, overflow) = if flags.carry.get() {
        flags.half_carry.set((a.get() & 0xF) < (imm8 & 0xF));
        a.get().overflowing_sub(imm8)
    } else {
        flags.half_carry.set((a.get() & 0xF) - 1 < (imm8 & 0xF));
        a.get().overflowing_sub(imm8 - 1)
    };
    flags.zero.set(result == 0);
    flags.carry.set(overflow);
    flags.subtract.set(true);

    a.set(result);
}

fn and_a_imm8(a: &Cell<u8>, imm8: u8, flags: &Flags) {
    let result = a.get() & imm8;
    flags.zero.set(result == 0);
    flags.carry.set(false);
    flags.subtract.set(false);
    flags.half_carry.set(true);
    a.set(result);
}

fn xor_a_imm8(a: &Cell<u8>, imm8: u8, flags: &Flags) {
    let result = a.get() ^ imm8;
    flags.zero.set(result == 0);
    flags.carry.set(false);
    flags.subtract.set(false);
    flags.half_carry.set(false);
    a.set(result);
}

fn or_a_imm8(a: &Cell<u8>, imm8: u8, flags: &Flags) {
    let result = a.get() | imm8;
    flags.zero.set(result == 0);
    flags.carry.set(false);
    flags.subtract.set(false);
    flags.half_carry.set(false);
    a.set(result);
}

fn cp_a_imm8(a: &Cell<u8>, imm8: u8, flags: &Flags) {
    let (result, overflow) = a.get().overflowing_sub(imm8);
    flags.zero.set(result == 0);
    flags.carry.set(overflow);
    flags.subtract.set(true);
    flags.half_carry.set((a.get() & 0xF) < (imm8 & 0xF));
}

fn ld_c_a(c: &mut u8, a: u8) {
    *c = a;
}

fn ld_a_c(a: &Cell<u8>, c: u8) {
    a.set(c);
}

fn ldh_imm8_a(imm8: &mut u8, a: &Cell<u8>) {
    *imm8 = a.get();
}

fn ldh_a_imm8(a: &Cell<u8>, imm8: u8) {
    a.set(imm8);
}

fn rlc_r8(r8: R8OrMem, flags: &Flags) {
    let result = match r8 {
        R8OrMem::R8(r8) => {
            let result = r8.get().rotate_left(1);
            r8.set(result);
            result
        }
        R8OrMem::Mem(r8) => {
            let result = (*r8).rotate_left(1);
            *r8 = result;
            result
        }
        R8OrMem::Ptr(_) => panic!("Pointer not supported"),
    };

    flags.zero.set(result == 0);
    flags.carry.set(result & 0b0000_0001 == 1);
    flags.subtract.set(false);
    flags.half_carry.set(false);
}

fn rrc_r8(r8: R8OrMem, flags: &Flags) {
    let result = match r8 {
        R8OrMem::R8(r8) => {
            let result = r8.get().rotate_right(1);
            r8.set(result);
            result
        }
        R8OrMem::Mem(r8) => {
            let result = (*r8).rotate_right(1);
            *r8 = result;
            result
        }
        R8OrMem::Ptr(_) => panic!("Pointer not supported"),
    };

    flags.zero.set(result == 0);
    flags.carry.set((result & 0b1000_0000) >> 7 == 1);
    flags.subtract.set(false);
    flags.half_carry.set(false);
}

fn rl_r8(r8: R8OrMem, flags: &Flags) {
    let (result, overflow) = match r8 {
        R8OrMem::R8(r8) => {
            let overflow = r8.get() & 0b1000_0000 != 0;
            let result = r8.get() << 1;
            r8.set(result | flags.carry.get() as u8);
            (result, overflow)
        }
        R8OrMem::Mem(r8) => {
            let overflow = (*r8) & 0b1000_0000 != 0;
            let result = (*r8) << 1;
            *r8 = result | flags.carry.get() as u8;
            (result, overflow)
        }
        R8OrMem::Ptr(_) => panic!("Pointer not supported"),
    };

    flags.zero.set(result == 0);
    flags.carry.set(overflow);
    flags.subtract.set(false);
    flags.half_carry.set(false);
}

fn rr_r8(r8: R8OrMem, flags: &Flags) {
    let (result, overflow) = match r8 {
        R8OrMem::R8(r8) => {
            let overflow = r8.get() & 0b0000_0001 != 0;
            let result = r8.get() >> 1;
            r8.set(result | (flags.carry.get() as u8) << 7);
            (result, overflow)
        }
        R8OrMem::Mem(r8) => {
            let overflow = (*r8) & 0b0000_0001 != 0;
            let result = (*r8) >> 1;
            *r8 = result | (flags.carry.get() as u8) << 7;
            (result, overflow)
        }
        R8OrMem::Ptr(_) => panic!("Pointer not supported"),
    };

    flags.zero.set(result == 0);
    flags.carry.set(overflow);
    flags.subtract.set(false);
    flags.half_carry.set(false);
}

fn ld_imm16_a(imm16: &mut u8, a: &Cell<u8>) {
    *imm16 = a.get();
}
