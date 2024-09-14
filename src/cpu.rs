#![allow(dead_code)]

use crate::mmu::Mmu;
use crate::ppu::{self, Ppu};
use crate::registers::{Flags, R16OrSP, R8OrMem, Registers, R16, R8};
use std::cell::Cell;
use std::time::Instant;

use crate::registers::{R16mem, R16stk};

#[derive(Debug)]
enum AfterInstruction {
    Increment,
    Decrement,
    None,
}

#[derive(Debug, PartialEq)]
pub enum State {
    Bootstrap,
    Running,
    Halted,
}

#[derive(Debug)]
pub struct Cpu {
    pub registers: Registers,
    pub bootstrap: Vec<u8>,
    pub memory: Vec<u8>,
    pub pc: u16,
    pub sp: u16,
    pub last_frame: Instant,
    pub mmu: Mmu,
    pub ppu: Ppu,
    pub ime: bool,
    pub state: State,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            registers: Registers::new(),
            bootstrap: vec![],
            memory: include_bytes!("../roms/bootstrap.gb").to_vec(),
            pc: 0,
            sp: 0,
            last_frame: Instant::now(),
            mmu: Mmu::new(),
            ppu: Ppu::new(),
            ime: false,
            state: State::Bootstrap,
        }
    }

    pub fn step(&mut self) -> u8 {
        let opcode = self.mmu.read_byte(self.pc);

        if opcode == 0xCB {
            let opcode = self.mmu.read_byte(self.pc + 1);
            match (opcode & 0b1100_0000) >> 6 {
                0b00 => {
                    let operand = R8::from_u8(opcode & 0b0000_0111);
                    let operand = self.registers.get_r8(operand);
                    let operand = match operand {
                        R8OrMem::Ptr(ptr) => R8OrMem::Mem(self.mmu.get_mut_byte(ptr)),
                        _ => operand,
                    };

                    match (opcode & 0b0011_1000) >> 3 {
                        0b000 => {
                            // ## println!("{:#04x}: rlc r8", self.pc);
                            rlc_r8(operand, &self.registers.flags);
                            self.pc += 2;
                            return 2;
                        }
                        0b001 => {
                            // ## println!("{:#04x}: rrc r8", self.pc);
                            rrc_r8(operand, &self.registers.flags);
                            self.pc += 2;
                            return 2;
                        }
                        0b010 => {
                            // ## println!("{:#04x}: rl r8", self.pc);
                            rl_r8(operand, &self.registers.flags);
                            self.pc += 2;
                            return 2;
                        }
                        0b011 => {
                            // ## println!("{:#04x}: rr r8", self.pc);
                            rr_r8(operand, &self.registers.flags);
                            self.pc += 2;
                            return 2;
                        }
                        0b100 => {
                            // ## println!("{:#04x}: sla r8", self.pc);
                            sla_r8(operand, &self.registers.flags);
                            self.pc += 2;
                            return 2;
                        }
                        0b101 => {
                            // ## println!("{:#04x}: sra r8", self.pc);
                            sra_r8(operand, &self.registers.flags);
                            self.pc += 2;
                            return 2;
                        }
                        0b110 => {
                            // ## println!("{:#04x}: swap r8", self.pc);
                            let operand = R8::from_u8(opcode & 0b0000_0111);
                            let operand = self.registers.get_r8(operand);
                            let operand = match operand {
                                R8OrMem::Ptr(ptr) => R8OrMem::Mem(self.mmu.get_mut_byte(ptr)),
                                _ => operand,
                            };
                            swap_r8(operand, &self.registers.flags);
                            self.pc += 2;
                            return 2;
                        }
                        0b111 => {
                            // ## println!("{:#04x}: srl r8", self.pc);
                            let operand = R8::from_u8(opcode & 0b0000_0111);
                            let operand = self.registers.get_r8(operand);
                            let operand = match operand {
                                R8OrMem::Ptr(ptr) => R8OrMem::Mem(self.mmu.get_mut_byte(ptr)),
                                _ => operand,
                            };
                            srl_r8(operand, &self.registers.flags);
                            self.pc += 2;
                            return 2;
                        }
                        _ => panic!("Bad CB Instructions"),
                    }
                }
                0b01 => {
                    // ## println!("{:#04x}: bit b3, r8", self.pc);
                    let bit_index = (opcode & 0b0011_1000) >> 3;
                    let operand = R8::from_u8(opcode & 0b0000_0111);
                    let operand = self.registers.get_r8(operand);
                    let operand = match operand {
                        R8OrMem::Ptr(ptr) => R8OrMem::Mem(self.mmu.get_mut_byte(ptr)),
                        _ => operand,
                    };
                    bit_b3_r8(bit_index, operand, &self.registers.flags);
                    self.pc += 2;
                    return 2;
                }
                0b10 => {
                    // ## println!("{:#04x}: res b3, r8", self.pc);
                    let bit_index = (opcode & 0b0011_1000) >> 3;
                    let operand = R8::from_u8(opcode & 0b0000_0111);
                    let operand = self.registers.get_r8(operand);
                    let operand = match operand {
                        R8OrMem::Ptr(ptr) => R8OrMem::Mem(self.mmu.get_mut_byte(ptr)),
                        _ => operand,
                    };
                    res_b3_r8(bit_index, operand);
                    self.pc += 2;
                    return 2;
                }
                0b11 => {
                    // ## println!("{:#04x}: set b3, r8", self.pc);
                    let bit_index = (opcode & 0b0011_1000) >> 3;
                    let operand = R8::from_u8(opcode & 0b0000_0111);
                    let operand = self.registers.get_r8(operand);
                    let operand = match operand {
                        R8OrMem::Ptr(ptr) => R8OrMem::Mem(self.mmu.get_mut_byte(ptr)),
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
                    // ## println!("{:#04x}: nop", self.pc);
                    self.pc += 1;
                    1
                } else if opcode == 24 {
                    // ## println!("{:#04x}: jr imm8", self.pc);
                    let imm8 = self.mmu.read_byte(self.pc + 1) as i8;
                    self.pc = (self.pc as i16 + imm8 as i16) as u16;
                    self.pc += 2;
                    return 3;
                } else if opcode & 0b0010_0111 == 32 {
                    // ## println!("{:#04x}: jr cond, imm8", self.pc);
                    let condition = (opcode & 0b0001_1000) >> 3;
                    let condition = self.registers.flags.get_condition(condition);
                    if condition {
                        let imm8 = self.mmu.read_byte(self.pc + 1) as i8;
                        self.pc = (self.pc as i16 + imm8 as i16) as u16;
                        self.pc += 2;
                        return 3;
                    }
                    self.pc += 2;
                    return 2;
                } else if opcode == 0b0001_0000 {
                    // ## println!("{:#04x}: stop", self.pc);
                    self.pc += 2;
                    return 1;
                } else {
                    match opcode & 0b0000_1111 {
                        0b0001 => {
                            // ## println!("{:#04x}: ld r16, imm16", self.pc);
                            let imm16 = self.mmu.read_word(self.pc + 1);
                            let dest = R16::from_u8((opcode & 0b0011_0000) >> 4);
                            let dest = self.registers.get_r16(dest);
                            match dest {
                                R16OrSP::SP => self.sp = imm16,
                                R16OrSP::R16(hi, lo) => {
                                    ld_r16_imm16((hi, lo), imm16);
                                }
                            }
                            self.pc += 3;
                            return 3;
                        }
                        0b0010 => {
                            // ## println!("{:#04x}: ld [r16mem], a", self.pc);
                            let dest = R16mem::from_u8((opcode & 0b0011_0000) >> 4);
                            let action = match dest {
                                R16mem::HLi => AfterInstruction::Increment,
                                R16mem::HLd => AfterInstruction::Decrement,
                                _ => AfterInstruction::None,
                            };
                            let dest = self.registers.get_r16mem(dest);
                            let dest = dest.1.get() as u16 | (dest.0.get() as u16) << 8;
                            self.mmu.write_byte(dest, self.registers.a.get());

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
                            // ## println!("{:#04x}: ld a, [r16mem]", self.pc);
                            let source = R16mem::from_u8((opcode & 0b0011_0000) >> 4);
                            let action = match source {
                                R16mem::HLi => AfterInstruction::Increment,
                                R16mem::HLd => AfterInstruction::Decrement,
                                _ => AfterInstruction::None,
                            };
                            let source = self.registers.get_r16mem(source);
                            let source = self
                                .mmu
                                .read_byte(source.1.get() as u16 | (source.0.get() as u16) << 8);
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
                            // ## println!("{:#04x}: ld [imm16], sp", self.pc);
                            let imm16 = self.mmu.read_word(self.pc + 1);
                            self.mmu.write_word(imm16, self.sp);
                            self.pc += 3;
                            return 5;
                        }
                        0b0011 => {
                            // ## println!("{:#04x}: inc r16", self.pc);
                            let operand = R16::from_u8((opcode & 0b0011_0000) >> 4);
                            let operand = self.registers.get_r16(operand);
                            match operand {
                                R16OrSP::SP => self.sp += 1,
                                R16OrSP::R16(hi, lo) => inc_r16((hi, lo)),
                            }
                            self.pc += 1;
                            return 2;
                        }
                        0b1011 => {
                            // ## println!("{:#04x}: dec r16", self.pc);
                            let operand = R16::from_u8((opcode & 0b0011_0000) >> 4);
                            let operand = self.registers.get_r16(operand);
                            match operand {
                                R16OrSP::SP => self.sp -= 1,
                                R16OrSP::R16(hi, lo) => dec_r16((hi, lo)),
                            }
                            self.pc += 1;
                            return 2;
                        }
                        0b1001 => {
                            // ## println!("{:#04x}: add hl, r16", self.pc);
                            let operand = R16::from_u8((opcode & 0b0011_0000) >> 4);
                            let operand = self.registers.get_r16(operand);
                            match operand {
                                R16OrSP::SP => add_hl_sp(
                                    (&self.registers.h, &self.registers.l),
                                    self.sp,
                                    &self.registers.flags,
                                ),
                                R16OrSP::R16(hi, lo) => add_hl_r16(
                                    (&self.registers.h, &self.registers.l),
                                    (hi, lo),
                                    &self.registers.flags,
                                ),
                            }
                            self.pc += 1;
                            return 2;
                        }
                        0b0100 | 0b1100 => {
                            // ## println!("{:#04x}: inc r8", self.pc);
                            let operand = R8::from_u8((opcode & 0b0011_1000) >> 3);
                            let operand = self.registers.get_r8(operand);
                            let operand = match operand {
                                R8OrMem::Ptr(ptr) => R8OrMem::Mem(self.mmu.get_mut_byte(ptr)),
                                _ => operand,
                            };
                            inc_r8(operand, &self.registers.flags);
                            self.pc += 1;
                            return 1;
                        }
                        0b0101 | 0b1101 => {
                            // ## println!("{:#04x}: dec r8", self.pc);
                            let operand = R8::from_u8((opcode & 0b0011_1000) >> 3);
                            let operand = self.registers.get_r8(operand);
                            let operand = match operand {
                                R8OrMem::Ptr(ptr) => R8OrMem::Mem(self.mmu.get_mut_byte(ptr)),
                                _ => operand,
                            };
                            dec_r8(operand, &self.registers.flags);
                            self.pc += 1;
                            return 1;
                        }
                        0b1110 | 0b0110 => {
                            // ## println!("{:#04x}: ld r8, imm8", self.pc);
                            let imm8 = self.mmu.read_byte(self.pc + 1);
                            let operand = R8::from_u8((opcode & 0b0011_1000) >> 3);
                            let operand = self.registers.get_r8(operand);
                            let operand = match operand {
                                R8OrMem::Ptr(ptr) => R8OrMem::Mem(self.mmu.get_mut_byte(ptr)),
                                _ => operand,
                            };
                            ld_r8_imm8(operand, imm8);
                            self.pc += 2;
                            return 2;
                        }
                        0b0111 | 0b1111 => match (opcode & 0b0011_1000) >> 3 {
                            0b000 => {
                                // ## println!("{:#04x}: rlca", self.pc);
                                rlc_r8(R8OrMem::R8(&self.registers.a), &self.registers.flags);
                                self.registers.flags.zero.set(false);
                                self.pc += 1;
                                return 1;
                            }
                            0b001 => {
                                // ## println!("{:#04x}: rrca", self.pc);
                                rrc_r8(R8OrMem::R8(&self.registers.a), &self.registers.flags);
                                self.registers.flags.zero.set(false);
                                self.pc += 1;
                                return 1;
                            }
                            0b010 => {
                                // ## println!("{:#04x}: rla", self.pc);
                                rl_r8(R8OrMem::R8(&self.registers.a), &self.registers.flags);
                                self.registers.flags.zero.set(false);
                                self.pc += 1;
                                return 1;
                            }
                            0b011 => {
                                // ## println!("{:#04x}: rra", self.pc);
                                rr_r8(R8OrMem::R8(&self.registers.a), &self.registers.flags);
                                self.pc += 1;
                                self.registers.flags.zero.set(false);
                                return 1;
                            }
                            0b100 => {
                                // ## println!("{:#04x}: daa", self.pc);
                                daa(&self.registers.a, &self.registers.flags);
                                self.pc += 1;
                                return 1;
                            }
                            0b101 => {
                                // ## println!("{:#04x}: cpl", self.pc);
                                cpl(&self.registers.a, &self.registers.flags);
                                self.pc += 1;
                                return 1;
                            }
                            0b110 => {
                                // ## println!("{:#04x}: scf", self.pc);
                                self.registers.flags.carry.set(true);
                                self.registers.flags.subtract.set(false);
                                self.registers.flags.half_carry.set(false);
                                self.pc += 1;
                                return 1;
                            }
                            0b111 => {
                                // ## println!("{:#04x}: ccf", self.pc);
                                let carry = self.registers.flags.carry.get();
                                self.registers.flags.carry.set(!carry);
                                self.registers.flags.subtract.set(false);
                                self.registers.flags.half_carry.set(false);
                                self.pc += 1;
                                return 1;
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
                // ## println!("{:#04x}: ld r8, r8", self.pc);

                let dest = R8::from_u8((opcode & 0b0011_1000) >> 3);
                let dest = self.registers.get_r8(dest);

                let source = R8::from_u8(opcode & 0b0000_0111);
                let source = self.registers.get_r8(source);

                let (source, dest) = match (&source, &dest) {
                    (R8OrMem::Ptr(ptr), R8OrMem::R8(_)) => {
                        (R8OrMem::Mem(self.mmu.get_mut_byte(*ptr)), dest)
                    }
                    (R8OrMem::R8(_), R8OrMem::Ptr(ptr)) => {
                        (source, R8OrMem::Mem(self.mmu.get_mut_byte(*ptr)))
                    }
                    (R8OrMem::Ptr(src), R8OrMem::Ptr(ptr)) => (
                        R8OrMem::Ptr(self.mmu.read_byte(*src) as u16),
                        R8OrMem::Mem(self.mmu.get_mut_byte(*ptr)),
                    ),
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
                    R8OrMem::Ptr(ptr) => R8OrMem::Mem(self.mmu.get_mut_byte(ptr)),
                    _ => r8,
                };

                match (opcode & 0b0011_1000) >> 3 {
                    0b0000 => {
                        // ## println!("{:#04x}: add a, r8", self.pc);
                        add_a_r8(a, r8, &self.registers.flags);
                        self.pc += 1;
                        1
                    }
                    0b0001 => {
                        // ## println!("{:#04x}: adc a, r8", self.pc);
                        adc_a_r8(a, r8, &self.registers.flags);
                        self.pc += 1;
                        1
                    }
                    0b0010 => {
                        // ## println!("{:#04x}: sub a, r8", self.pc);
                        sub_a_r8(a, r8, &self.registers.flags);
                        self.pc += 1;
                        1
                    }
                    0b0011 => {
                        // ## println!("{:#04x}: sbc a, r8", self.pc);
                        sbc_a_r8(a, r8, &self.registers.flags);
                        self.pc += 1;
                        1
                    }
                    0b0100 => {
                        // ## println!("{:#04x}: and a, r8", self.pc);
                        and_a_r8(a, r8, &self.registers.flags);
                        self.pc += 1;
                        1
                    }
                    0b0101 => {
                        // ## println!("{:#04x}: xor a, r8", self.pc);
                        xor_a_r8(a, r8, &self.registers.flags);
                        self.pc += 1;
                        1
                    }
                    0b0110 => {
                        // ## println!("{:#04x}: or a, r8", self.pc);
                        or_a_r8(a, r8, &self.registers.flags);
                        self.pc += 1;
                        1
                    }
                    0b0111 => {
                        // ## println!("{:#04x}: cp a, r8", self.pc);
                        cp_a_r8(a, r8, &self.registers.flags);
                        self.pc += 1;
                        1
                    }
                    _ => {
                        // ## println!("{:#04x}: Unknown opcode {}", self.pc, opcode);
                        unreachable!()
                    }
                }
            }
            0b11 => {
                /* Block 3 */
                if opcode & 0b0000_0111 == 0b110 {
                    let imm8 = self.mmu.read_byte(self.pc + 1);
                    let a = &self.registers.a;
                    match (opcode & 0b0011_1000) >> 3 {
                        0b000 => {
                            // ## println!("{:#04x}: add a, imm8", self.pc);
                            add_a_imm8(a, imm8, &self.registers.flags);
                            self.pc += 2;
                            return 2;
                        }
                        0b001 => {
                            // ## println!("{:#04x}: adc a, imm8", self.pc);
                            adc_a_imm8(a, imm8, &self.registers.flags);
                            self.pc += 2;
                            return 2;
                        }
                        0b010 => {
                            // ## println!("{:#04x}: sub a, imm8", self.pc);
                            sub_a_imm8(a, imm8, &self.registers.flags);
                            self.pc += 2;
                            return 2;
                        }
                        0b011 => {
                            // ## println!("{:#04x}: sbc a, imm8", self.pc);
                            sbc_a_imm8(a, imm8, &self.registers.flags);
                            self.pc += 2;
                            return 2;
                        }
                        0b100 => {
                            // ## println!("{:#04x}: and a, imm8", self.pc);
                            and_a_imm8(a, imm8, &self.registers.flags);
                            self.pc += 2;
                            return 2;
                        }
                        0b101 => {
                            // ## println!("{:#04x}: xor a, imm8", self.pc);
                            xor_a_imm8(a, imm8, &self.registers.flags);
                            self.pc += 2;
                            return 2;
                        }
                        0b110 => {
                            // ## println!("{:#04x}: or a, imm8", self.pc);
                            or_a_imm8(a, imm8, &self.registers.flags);
                            self.pc += 2;
                            return 2;
                        }
                        0b111 => {
                            // ## println!("{:#04x}: cp a, imm8", self.pc);
                            cp_a_imm8(a, imm8, &self.registers.flags);
                            self.pc += 2;
                            return 2;
                        }
                        _ => {
                            unreachable!();
                        }
                    }
                }

                if opcode & 0b0000_0111 == 0b111 {
                    // ## println!("{:#04x}: rst n", self.pc);
                    let n = (opcode & 0b0011_1000) >> 3;
                    self.mmu.write_word(self.sp - 2, self.pc + 1);
                    self.sp -= 2;
                    self.pc = n as u16 * 8;
                    return 4;
                }

                match opcode {
                    0b1110_0010 => {
                        // ## println!("{:#04x}: ld (c), a", self.pc);
                        let a = self.registers.a.get();
                        let c = self.registers.c.get();
                        self.mmu.write_byte(0xFF00 + c as u16, a);
                        self.pc += 1;
                        return 2;
                    }
                    0b1111_0010 => {
                        // ## println!("{:#04x}: ld a, (c)", self.pc);
                        let a = &self.registers.a;
                        let c = self.registers.c.get() as u16;
                        let c = self.mmu.read_byte(0xFF00_u16 + c);
                        ld_a_c(a, c);
                        self.pc += 1;
                        return 2;
                    }
                    0b1110_0000 => {
                        // ## println!("{:#04x}: ldh [imm8], a", self.pc);
                        let imm8 = self.mmu.read_byte(self.pc + 1);
                        self.mmu
                            .write_byte(0xFF00 + imm8 as u16, self.registers.a.get());
                        self.pc += 2;
                        return 3;
                    }
                    0b1111_0000 => {
                        // ## println!("{:#04x}: ldh a, [imm8]", self.pc);
                        let imm8 = self.mmu.read_byte(self.pc + 1);
                        if imm8 == 0x00 {
                            // TODO: joystick input
                            self.registers.a.set(0xef);
                            self.pc += 2;
                            return 3;
                        }
                        // else if imm8 == 0x44 {
                        //     // TEMP: For testing
                        //     self.registers.a.set(0x90);
                        //     self.pc += 2;
                        //     return 3;
                        // }
                        let imm8 = self.mmu.read_byte(0xFF00 + imm8 as u16);
                        ldh_a_imm8(&self.registers.a, imm8);
                        self.pc += 2;
                        return 3;
                    }
                    0b1110_1010 => {
                        // ## println!("{:#04x}: ld [imm16], a", self.pc);
                        let imm16 = self.mmu.read_word(self.pc + 1);
                        self.mmu.write_byte(imm16, self.registers.a.get());
                        self.pc += 3;
                        return 4;
                    }
                    0b1100_1010 => {
                        // ## println!("{:#04x}: jp z, imm16", self.pc);
                        if self.registers.flags.zero.get() {
                            self.pc = self.mmu.read_word(self.pc + 1);
                            return 4;
                        }
                        self.pc += 3;
                        return 3;
                    }
                    0b1100_0010 => {
                        // ## println!("{:#04x}: jp nz, imm16", self.pc);
                        if !self.registers.flags.zero.get() {
                            self.pc = self.mmu.read_word(self.pc + 1);
                            return 4;
                        }
                        self.pc += 3;
                        return 3;
                    }
                    0b1101_1010 => {
                        // ## println!("{:#04x}: jp c, imm16", self.pc);
                        if self.registers.flags.carry.get() {
                            self.pc = self.mmu.read_word(self.pc + 1);
                            return 4;
                        }
                        self.pc += 3;
                        return 3;
                    }
                    0b1101_0010 => {
                        // ## println!("{:#04x}: jp nc, imm16", self.pc);
                        if !self.registers.flags.carry.get() {
                            self.pc = self.mmu.read_word(self.pc + 1);
                            return 4;
                        }
                        self.pc += 3;
                        return 3;
                    }
                    0b1100_0100 => {
                        // ## println!("{:#04x}: call nz, imm16", self.pc);
                        if !self.registers.flags.zero.get() {
                            self.mmu.write_word(self.sp - 2, self.pc + 3);
                            self.sp -= 2;
                            self.pc = self.mmu.read_word(self.pc + 1);
                            return 6;
                        }
                        self.pc += 3;
                        return 3;
                    }
                    0b1100_1100 => {
                        // ## println!("{:#04x}: call z, imm16", self.pc);
                        if self.registers.flags.zero.get() {
                            self.mmu.write_word(self.sp - 2, self.pc + 3);
                            self.sp -= 2;
                            self.pc = self.mmu.read_word(self.pc + 1);
                            return 6;
                        }
                        self.pc += 3;
                        return 3;
                    }
                    0b1101_1100 => {
                        // ## println!("{:#04x}: call c, imm16", self.pc);
                        if self.registers.flags.carry.get() {
                            self.mmu.write_word(self.sp - 2, self.pc + 3);
                            self.sp -= 2;
                            self.pc = self.mmu.read_word(self.pc + 1);
                            return 6;
                        }
                        self.pc += 3;
                        return 3;
                    }
                    0b1101_0100 => {
                        // ## println!("{:#04x}: call nc, imm16", self.pc);
                        if !self.registers.flags.carry.get() {
                            self.mmu.write_word(self.sp - 2, self.pc + 3);
                            self.sp -= 2;
                            self.pc = self.mmu.read_word(self.pc + 1);
                            return 6;
                        }
                        self.pc += 3;
                        return 3;
                    }
                    0b1111_1010 => {
                        // ## println!("{:#04x}: ld a, [imm16]", self.pc);
                        let imm16 = self.mmu.read_word(self.pc + 1);
                        let imm16 = self.mmu.read_byte(imm16);
                        ld_a_imm16(&self.registers.a, imm16);
                        self.pc += 3;
                        return 4;
                    }
                    0b1111_1001 => {
                        // ## println!("{:#04x}: ld sp, hl", self.pc);
                        self.sp =
                            self.registers.l.get() as u16 | (self.registers.h.get() as u16) << 8;
                        self.pc += 1;
                        return 2;
                    }
                    0b1111_1000 => {
                        // ## println!("{:#04x}: ld hl, sp + imm8", self.pc);
                        let imm8 = self.mmu.read_byte(self.pc + 1) as i8;
                        add_hl_sp_imm8(
                            (&self.registers.h, &self.registers.l),
                            self.sp,
                            imm8,
                            &self.registers.flags,
                        );
                        self.pc += 2;
                        return 3;
                    }
                    0b1100_1101 => {
                        // ## println!("{:#04x}: call imm16", self.pc);
                        self.mmu.write_word(self.sp - 2, self.pc + 3);
                        self.sp -= 2;
                        self.pc = self.mmu.read_word(self.pc + 1);
                        return 6;
                    }
                    0b1100_1001 => {
                        // ## println!("{:#04x}: ret", self.pc);
                        self.pc = self.mmu.read_word(self.sp);
                        self.sp += 2;
                        return 4;
                    }
                    0b1101_1001 => {
                        // ## println!("{:#04x}: reti", self.pc);
                        self.pc = self.mmu.read_word(self.sp);
                        self.sp += 2;
                        self.ime = true;
                        return 4;
                    }
                    0b1100_0011 => {
                        // ## println!("{:#04x}: jp imm16", self.pc);
                        self.pc = self.mmu.read_word(self.pc + 1);
                        return 4;
                    }
                    0b1110_1000 => {
                        // ## println!("{:#04x}: add sp, imm8", self.pc);
                        let imm8 = self.mmu.read_byte(self.pc + 1) as i8;
                        self.sp = add_sp_imm8(self.sp, imm8, &self.registers.flags);
                        self.pc += 2;
                        return 4;
                    }
                    0b1110_1001 => {
                        // ## println!("{:#04x}: jp hl", self.pc);
                        self.pc =
                            (self.registers.h.get() as u16) << 8 | self.registers.l.get() as u16;
                        return 1;
                    }
                    0b1100_0000 => {
                        // ## println!("{:#04x}: ret nz", self.pc);
                        if !self.registers.flags.zero.get() {
                            self.pc = self.mmu.read_word(self.sp);
                            self.sp += 2;
                            return 5;
                        }
                        self.pc += 1;
                        return 2;
                    }
                    0b1100_1000 => {
                        // ## println!("{:#04x}: ret z", self.pc);
                        if self.registers.flags.zero.get() {
                            self.pc = self.mmu.read_word(self.sp);
                            self.sp += 2;
                            return 5;
                        }
                        self.pc += 1;
                        return 2;
                    }
                    0b1101_0000 => {
                        // ## println!("{:#04x}: ret nc", self.pc);
                        if !self.registers.flags.carry.get() {
                            self.pc = self.mmu.read_word(self.sp);
                            self.sp += 2;
                            return 5;
                        }
                        self.pc += 1;
                        return 2;
                    }
                    0b1101_1000 => {
                        // ## println!("{:#04x}: ret c", self.pc);
                        if self.registers.flags.carry.get() {
                            self.pc = self.mmu.read_word(self.sp);
                            self.sp += 2;
                            return 5;
                        }
                        self.pc += 1;
                        return 2;
                    }
                    0b1111_0011 => {
                        // ## println!("{:#04x}: di", self.pc);
                        self.ime = false;
                        self.pc += 1;
                        return 1;
                    }
                    0b1111_1011 => {
                        // ## println!("{:#04x}: ei", self.pc);
                        self.ime = true;
                        self.pc += 1;
                        return 1;
                    }
                    _ => {}
                }

                match opcode & 0b0000_1111 {
                    0b0001 => {
                        // ## println!("{:#04x}: pop r16stk", self.pc);
                        let register = R16stk::from_u8((opcode & 0b0011_0000) >> 4);
                        match register {
                            R16stk::AF => {
                                let lo = self.mmu.read_byte(self.sp);
                                let hi = self.mmu.read_byte(self.sp + 1);
                                self.registers.a.set(hi);
                                self.registers.flags.set_from_u8(lo);
                            }
                            _ => {
                                let register = self.registers.get_r16stk(register);
                                let lo = self.mmu.read_byte(self.sp);
                                let hi = self.mmu.read_byte(self.sp + 1);
                                ld_r16_imm16(register, (hi as u16) << 8 | lo as u16);
                            }
                        }
                        self.sp += 2;
                        self.pc += 1;
                        return 4;
                    }
                    0b0101 => {
                        // ## println!("{:#04x}: push r16stk", self.pc);
                        let register = R16stk::from_u8((opcode & 0b0011_0000) >> 4);
                        match register {
                            R16stk::AF => {
                                let hi = self.registers.a.get();
                                let lo = self.registers.flags.to_u8();
                                self.mmu
                                    .write_word(self.sp - 2, (hi as u16) << 8 | lo as u16);
                            }
                            _ => {
                                let register = self.registers.get_r16stk(register);
                                let hi = register.0.get();
                                let lo = register.1.get();
                                self.mmu
                                    .write_word(self.sp - 2, (hi as u16) << 8 | lo as u16);
                            }
                        }
                        self.sp -= 2;
                        self.pc += 1;
                        return 3;
                    }
                    _ => {}
                }
                panic!("{:#04x}: Block 3 - Opcode {:#04x}", self.pc, opcode);
            }
            _ => unreachable!(),
        }
    }

    pub fn game_loop(&mut self, frame: &mut [u8]) -> bool {
        self.last_frame = Instant::now();

        let mut ticks = 0;
        for line in 0..154 {
            while ticks < 456 {
                if self.pc == 0x100 {
                    self.state = State::Running;
                }
                ticks += self.step() as u32;
                if self.ime {
                    if self.mmu.read_byte(0xFFFF) & self.mmu.read_byte(0xFF0F) & 0b0000_0001 != 0 {
                        /* V-Blank interrupt */
                        self.ime = false;
                        self.mmu
                            .write_byte(0xFF0F, self.mmu.read_byte(0xFF0F) & !0b0000_0001);
                        self.mmu.write_word(self.sp - 2, self.pc);
                        self.sp -= 2;
                        self.pc = 0x40;
                    } else if self.mmu.read_byte(0xFFFF) & self.mmu.read_byte(0xFF0F) & 0b0000_0010
                        != 0
                    {
                        /* LCD STAT interrupt */
                        self.ime = false;
                        self.mmu
                            .write_byte(0xFF0F, self.mmu.read_byte(0xFF0F) & !0b0000_0010);
                        self.mmu.write_word(self.sp - 2, self.pc);
                        self.sp -= 2;
                        self.pc = 0x48;
                    } else {
                        // TODO: Implement other interrupts
                    }
                }
            }
            ticks = 0;
            if line < 144 {
                let scx = self.mmu.read_byte(0xFF43);
                let scy = self.mmu.read_byte(0xFF42);
                ppu::draw_scanline(&self.mmu, frame, scx, scy, line);
            }

            if line + 1 == self.mmu.read_byte(0xFF45) && self.mmu.read_byte(0xFFFF) & 0b0000_0010 != 0 {
                self.mmu
                    .write_byte(0xFF0F, self.mmu.read_byte(0xFF0F) | 0b0000_0010);
            }

            if line == 144 && self.mmu.read_byte(0xFFFF) & 0b0000_0001 != 0 {
                self.mmu
                    .write_byte(0xFF0F, self.mmu.read_byte(0xFF0F) | 0b0000_0001);
            }
            self.mmu.write_byte(0xFF44, line);
        }
        true
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

fn adc_a_r8(reg_a: &Cell<u8>, r8: R8OrMem, flags: &Flags) {
    let a = reg_a.get() as u16;
    let imm8 = match r8 {
        R8OrMem::R8(r8) => r8.get(),
        R8OrMem::Mem(r8) => *r8,
        R8OrMem::Ptr(_) => panic!("Pointer not supported"),
    } as u16;

    let result = if !flags.carry.get() {
        flags.half_carry.set((a & 0xF) + (imm8 & 0xF) > 0xF);
        flags.carry.set(a + imm8 > 0xFF);
        (a + imm8) as u8
    } else {
        flags.half_carry.set((a & 0xF) + (imm8 & 0xF) + 1 > 0xF);
        flags.carry.set(a + imm8 + 1 > 0xFF);
        (a + imm8 + 1) as u8
    };
    flags.zero.set(result == 0);
    flags.subtract.set(false);

    reg_a.set(result);
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

fn sbc_a_r8(reg_a: &Cell<u8>, r8: R8OrMem, flags: &Flags) {
    let a = reg_a.get() as u16;
    let c = flags.carry.get() as u16;
    let imm8 = match r8 {
        R8OrMem::R8(r8) => r8.get(),
        R8OrMem::Mem(r8) => *r8,
        R8OrMem::Ptr(_) => panic!("Pointer not supported"),
    } as u16;

    let result = a.wrapping_sub(imm8 + c) as u8;
    flags.half_carry.set(a & 0xF < (imm8 & 0xF) + c);
    flags.carry.set(a < imm8 + c);
    flags.zero.set(result == 0);
    flags.subtract.set(true);

    reg_a.set(result);
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
        R8OrMem::Ptr(mem) => mem as u8,
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

fn ld_a_r16mem(a: &Cell<u8>, source: u8) {
    a.set(source);
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

    flags.subtract.set(false);
    flags.half_carry.set((hl & 0xFFF) + (r16 & 0xFFF) > 0xFFF);
    flags.carry.set(overflow);

    h.set((result >> 8) as u8);
    l.set(result as u8);
}

fn add_hl_sp((h, l): (&Cell<u8>, &Cell<u8>), sp: u16, flags: &Flags) {
    let hl = (h.get() as u16) << 8 | l.get() as u16;

    let (result, overflow) = hl.overflowing_add(sp);

    flags.subtract.set(false);
    flags.half_carry.set((hl & 0xFFF) + (sp & 0xFFF) > 0xFFF);
    flags.carry.set(overflow);

    h.set((result >> 8) as u8);
    l.set(result as u8);
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

fn adc_a_imm8(r8: &Cell<u8>, imm8: u8, flags: &Flags) {
    let a = r8.get() as u16;
    let imm8 = imm8 as u16;

    let result = if !flags.carry.get() {
        flags.half_carry.set((a & 0xF) + (imm8 & 0xF) > 0xF);
        flags.carry.set(a + imm8 > 0xFF);
        (a + imm8) as u8
    } else {
        flags.half_carry.set((a & 0xF) + (imm8 & 0xF) + 1 > 0xF);
        flags.carry.set(a + imm8 + 1 > 0xFF);
        (a + imm8 + 1) as u8
    };
    flags.zero.set(result == 0);
    flags.subtract.set(false);

    r8.set(result);
}

fn sub_a_imm8(a: &Cell<u8>, imm8: u8, flags: &Flags) {
    let (result, overflow) = a.get().overflowing_sub(imm8);
    flags.zero.set(result == 0);
    flags.carry.set(overflow);
    flags.subtract.set(true);
    flags.half_carry.set((a.get() & 0xF) < (imm8 & 0xF));
    a.set(result);
}

fn sbc_a_imm8(r8: &Cell<u8>, imm8: u8, flags: &Flags) {
    let a = r8.get() as u16;
    let c = flags.carry.get() as u16;
    let imm8 = imm8 as u16;

    let result = a.wrapping_sub(imm8 + c) as u8;
    flags.half_carry.set(a & 0xF < (imm8 & 0xF) + c);
    flags.carry.set(a < imm8 + c);
    flags.zero.set(result == 0);
    flags.subtract.set(true);

    r8.set(result);
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

fn ld_a_c(a: &Cell<u8>, c: u8) {
    a.set(c);
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
            let result = (r8.get() << 1) | flags.carry.get() as u8;
            r8.set(result);
            (result, overflow)
        }
        R8OrMem::Mem(r8) => {
            let overflow = (*r8) & 0b1000_0000 != 0;
            let result = ((*r8) << 1) | flags.carry.get() as u8;
            *r8 = result;
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
            let result = r8.get() >> 1 | (flags.carry.get() as u8) << 7;
            r8.set(result);
            (result, overflow)
        }
        R8OrMem::Mem(r8) => {
            let overflow = (*r8) & 0b0000_0001 != 0;
            let result = (*r8) >> 1 | (flags.carry.get() as u8) << 7;
            *r8 = result;
            (result, overflow)
        }
        R8OrMem::Ptr(_) => panic!("Pointer not supported"),
    };

    flags.zero.set(result == 0);
    flags.carry.set(overflow);
    flags.subtract.set(false);
    flags.half_carry.set(false);
}

fn cpl(a: &Cell<u8>, flags: &Flags) {
    a.set(!a.get());
    flags.subtract.set(true);
    flags.half_carry.set(true);
}

fn swap_r8(r8: R8OrMem, flags: &Flags) {
    let result = match r8 {
        R8OrMem::R8(r8) => {
            let result = r8.get().rotate_left(4) | r8.get().rotate_right(4);
            r8.set(result);
            result
        }
        R8OrMem::Mem(r8) => {
            let result = (*r8).rotate_left(4) | (*r8).rotate_right(4);
            *r8 = result;
            result
        }
        R8OrMem::Ptr(_) => panic!("Pointer not supported"),
    };

    flags.zero.set(result == 0);
    flags.carry.set(false);
    flags.subtract.set(false);
    flags.half_carry.set(false);
}

fn add_sp_imm8(sp: u16, imm8: i8, flags: &Flags) -> u16 {
    let sp = sp as i32;
    let imm8 = imm8 as i32;

    let result = sp.wrapping_add(imm8);
    flags.zero.set(false);
    flags.carry.set((sp & 0xFF) + (imm8 & 0xFF) > 0xFF);
    flags.subtract.set(false);
    flags.half_carry.set((sp & 0xF) + (imm8 & 0xF) > 0xF);

    result as u16
}

fn ld_a_imm16(a: &Cell<u8>, imm16: u8) {
    a.set(imm16);
}

fn srl_r8(r8: R8OrMem, flags: &Flags) {
    let result = match r8 {
        R8OrMem::R8(r8) => {
            flags.carry.set(r8.get() & 0b0000_0001 == 1);
            let result = r8.get() >> 1;
            r8.set(result);
            result
        }
        R8OrMem::Mem(r8) => {
            flags.carry.set((*r8) & 0b0000_0001 == 1);
            let result = (*r8) >> 1;
            *r8 = result;
            result
        }
        R8OrMem::Ptr(_) => panic!("Pointer not supported"),
    };

    flags.zero.set(result == 0);
    flags.subtract.set(false);
    flags.half_carry.set(false);
}

fn daa(a: &Cell<u8>, flags: &Flags) {
    let mut reg_a = a.get();
    let mut adjust = 0;
    if flags.half_carry.get() || (!flags.subtract.get() && (reg_a & 0xF) > 9) {
        adjust |= 0x06;
    }
    if flags.carry.get() || (!flags.subtract.get() && reg_a > 0x99) {
        adjust |= 0x60;
        flags.carry.set(true);
    }
    if flags.subtract.get() {
        reg_a = reg_a.wrapping_sub(adjust);
    } else {
        reg_a = reg_a.wrapping_add(adjust);
    }
    flags.zero.set(reg_a == 0);
    flags.half_carry.set(false);
    a.set(reg_a);
}

fn sla_r8(r8: R8OrMem, flags: &Flags) {
    let result = match r8 {
        R8OrMem::R8(r8) => {
            flags.carry.set(r8.get() & 0b1000_0000 != 0);
            let result = r8.get() << 1;
            r8.set(result);
            result
        }
        R8OrMem::Mem(r8) => {
            flags.carry.set((*r8) & 0b1000_0000 != 0);
            let result = (*r8) << 1;
            *r8 = result;
            result
        }
        R8OrMem::Ptr(_) => panic!("Pointer not supported"),
    };

    flags.zero.set(result == 0);
    flags.subtract.set(false);
    flags.half_carry.set(false);
}

fn sra_r8(r8: R8OrMem, flags: &Flags) {
    let result = match r8 {
        R8OrMem::R8(r8) => {
            flags.carry.set(r8.get() & 0b0000_0001 == 1);
            let result = (r8.get() & 0b1000_0000) | (r8.get() >> 1);
            r8.set(result);
            result
        }
        R8OrMem::Mem(r8) => {
            flags.carry.set((*r8) & 0b0000_0001 == 1);
            let result = ((*r8) & 0b1000_0000) | ((*r8) >> 1);
            *r8 = result;
            result
        }
        R8OrMem::Ptr(_) => panic!("Pointer not supported"),
    };

    flags.zero.set(result == 0);
    flags.subtract.set(false);
    flags.half_carry.set(false);
}

fn add_hl_sp_imm8((h, l): (&Cell<u8>, &Cell<u8>), sp: u16, imm8: i8, flags: &Flags) {
    let imm8 = imm8 as i16;
    let hl = sp.wrapping_add(imm8 as u16);
    flags.zero.set(false);
    flags.subtract.set(false);
    flags.carry.set((sp & 0xFF) + (imm8 as u16 & 0xFF) > 0xFF);
    flags.half_carry.set((sp & 0xF) + (imm8 as u16 & 0xF) > 0xF);
    h.set((hl >> 8) as u8);
    l.set(hl as u8);
}
