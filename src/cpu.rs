use crate::mmu::Mmu;
use crate::ppu::draw_scanline;
use crate::registers::{Flags, R16OrSP, R8OrMem, Registers, R16, R8};
use std::cell::Cell;

use crate::registers::{R16mem, R16stk};

enum AfterInstruction {
    Increment,
    Decrement,
    None,
}

#[derive(PartialEq)]
pub enum State {
    Running,
    Halted,
    Ime,
}

pub struct Cpu {
    pub registers: Registers,
    pub pc: u16,
    pub sp: u16,
    pub mmu: Mmu,
    pub ime: bool,
    pub state: State,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            registers: Registers::new(),
            pc: 0,
            sp: 0,
            mmu: Mmu::new(),
            ime: false,
            state: State::Running,
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

                    match (opcode & 0b0011_1000) >> 3 {
                        0b000 => {
                            // ## println!("{:#04x}: rlc r8", self.pc);
                            match operand {
                                R8OrMem::R8(r8) => r8.set(rlc_r8(r8.get(), &self.registers.flags)),
                                R8OrMem::Ptr(ptr) => {
                                    let value = self.mmu.read_byte(ptr);
                                    let value = rlc_r8(value, &self.registers.flags);
                                    self.mmu.write_byte(ptr, value);
                                }
                            };
                            self.pc += 2;
                            return 2;
                        }
                        0b001 => {
                            // ## println!("{:#04x}: rrc r8", self.pc);
                            match operand {
                                R8OrMem::R8(r8) => r8.set(rrc_r8(r8.get(), &self.registers.flags)),
                                R8OrMem::Ptr(ptr) => {
                                    let value = self.mmu.read_byte(ptr);
                                    let value = rrc_r8(value, &self.registers.flags);
                                    self.mmu.write_byte(ptr, value);
                                }
                            };
                            self.pc += 2;
                            return 2;
                        }
                        0b010 => {
                            // ## println!("{:#04x}: rl r8", self.pc);
                            match operand {
                                R8OrMem::R8(r8) => r8.set(rl_r8(r8.get(), &self.registers.flags)),
                                R8OrMem::Ptr(ptr) => {
                                    let value = self.mmu.read_byte(ptr);
                                    let value = rl_r8(value, &self.registers.flags);
                                    self.mmu.write_byte(ptr, value);
                                }
                            };
                            self.pc += 2;
                            return 2;
                        }
                        0b011 => {
                            // ## println!("{:#04x}: rr r8", self.pc);
                            match operand {
                                R8OrMem::R8(r8) => r8.set(rr_r8(r8.get(), &self.registers.flags)),
                                R8OrMem::Ptr(ptr) => {
                                    let value = self.mmu.read_byte(ptr);
                                    let value = rr_r8(value, &self.registers.flags);
                                    self.mmu.write_byte(ptr, value);
                                }
                            };
                            self.pc += 2;
                            return 2;
                        }
                        0b100 => {
                            // ## println!("{:#04x}: sla r8", self.pc);
                            match operand {
                                R8OrMem::R8(r8) => r8.set(sla_r8(r8.get(), &self.registers.flags)),
                                R8OrMem::Ptr(ptr) => {
                                    let value = self.mmu.read_byte(ptr);
                                    let value = sla_r8(value, &self.registers.flags);
                                    self.mmu.write_byte(ptr, value);
                                }
                            };
                            self.pc += 2;
                            return 2;
                        }
                        0b101 => {
                            // ## println!("{:#04x}: sra r8", self.pc);
                            match operand {
                                R8OrMem::R8(r8) => r8.set(sra_r8(r8.get(), &self.registers.flags)),
                                R8OrMem::Ptr(ptr) => {
                                    let value = self.mmu.read_byte(ptr);
                                    let value = sra_r8(value, &self.registers.flags);
                                    self.mmu.write_byte(ptr, value);
                                }
                            };
                            self.pc += 2;
                            return 2;
                        }
                        0b110 => {
                            // ## println!("{:#04x}: swap r8", self.pc);
                            match operand {
                                R8OrMem::R8(r8) => r8.set(swap_r8(r8.get(), &self.registers.flags)),
                                R8OrMem::Ptr(ptr) => {
                                    let value = self.mmu.read_byte(ptr);
                                    let value = swap_r8(value, &self.registers.flags);
                                    self.mmu.write_byte(ptr, value);
                                }
                            };
                            self.pc += 2;
                            return 2;
                        }
                        0b111 => {
                            // ## println!("{:#04x}: srl r8", self.pc);
                            match operand {
                                R8OrMem::R8(r8) => r8.set(srl_r8(r8.get(), &self.registers.flags)),
                                R8OrMem::Ptr(ptr) => {
                                    let value = self.mmu.read_byte(ptr);
                                    let value = srl_r8(value, &self.registers.flags);
                                    self.mmu.write_byte(ptr, value);
                                }
                            };
                            self.pc += 2;
                            return 2;
                        }
                        _ => panic!("Bad CB Instructions"),
                    }
                }
                0b01 => {
                    // ## println!("{:#04x}: bit b3, r8", self.pc);
                    let bit_index = (opcode & 0b0011_1000) >> 3;
                    let value = R8::from_u8(opcode & 0b0000_0111);
                    let value = self.registers.get_r8(value);
                    let value = match value {
                        R8OrMem::R8(r8) => r8.get(),
                        R8OrMem::Ptr(ptr) => self.mmu.read_byte(ptr),
                    };
                    bit_b3_r8(bit_index, value, &self.registers.flags);
                    self.pc += 2;
                    return 2;
                }
                0b10 => {
                    // ## println!("{:#04x}: res b3, r8", self.pc);
                    let bit_index = (opcode & 0b0011_1000) >> 3;
                    let operand = R8::from_u8(opcode & 0b0000_0111);
                    let operand = self.registers.get_r8(operand);
                    match operand {
                        R8OrMem::R8(r8) => r8.set(res_b3_r8(bit_index, r8.get())),
                        R8OrMem::Ptr(ptr) => {
                            let value = self.mmu.read_byte(ptr);
                            let value = res_b3_r8(bit_index, value);
                            self.mmu.write_byte(ptr, value);
                        }
                    };
                    self.pc += 2;
                    return 2;
                }
                0b11 => {
                    // ## println!("{:#04x}: set b3, r8", self.pc);
                    let bit_index = (opcode & 0b0011_1000) >> 3;
                    let operand = R8::from_u8(opcode & 0b0000_0111);
                    let operand = self.registers.get_r8(operand);
                    match operand {
                        R8OrMem::R8(r8) => r8.set(set_b3_r8(bit_index, r8.get())),
                        R8OrMem::Ptr(ptr) => {
                            let value = self.mmu.read_byte(ptr);
                            let value = set_b3_r8(bit_index, value);
                            self.mmu.write_byte(ptr, value);
                        }
                    };
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
                            match operand {
                                R8OrMem::R8(r8) => r8.set(inc_r8(r8.get(), &self.registers.flags)),
                                R8OrMem::Ptr(ptr) => {
                                    let value = self.mmu.read_byte(ptr);
                                    let value = inc_r8(value, &self.registers.flags);
                                    self.mmu.write_byte(ptr, value);
                                }
                            };
                            self.pc += 1;
                            return 1;
                        }
                        0b0101 | 0b1101 => {
                            // ## println!("{:#04x}: dec r8", self.pc);
                            let operand = R8::from_u8((opcode & 0b0011_1000) >> 3);
                            let operand = self.registers.get_r8(operand);
                            match operand {
                                R8OrMem::R8(r8) => r8.set(dec_r8(r8.get(), &self.registers.flags)),
                                R8OrMem::Ptr(ptr) => {
                                    let value = self.mmu.read_byte(ptr);
                                    let value = dec_r8(value, &self.registers.flags);
                                    self.mmu.write_byte(ptr, value);
                                }
                            };
                            self.pc += 1;
                            return 1;
                        }
                        0b1110 | 0b0110 => {
                            // ## println!("{:#04x}: ld r8, imm8", self.pc);
                            let imm8 = self.mmu.read_byte(self.pc + 1);
                            let operand = R8::from_u8((opcode & 0b0011_1000) >> 3);
                            let operand = self.registers.get_r8(operand);
                            match operand {
                                R8OrMem::R8(r8) => r8.set(imm8),
                                R8OrMem::Ptr(ptr) => self.mmu.write_byte(ptr, imm8),
                            };
                            self.pc += 2;
                            return 2;
                        }
                        0b0111 | 0b1111 => match (opcode & 0b0011_1000) >> 3 {
                            0b000 => {
                                // ## println!("{:#04x}: rlca", self.pc);
                                self.registers
                                    .a
                                    .set(rlc_r8(self.registers.a.get(), &self.registers.flags));
                                self.registers.flags.zero.set(false);
                                self.pc += 1;
                                return 1;
                            }
                            0b001 => {
                                // ## println!("{:#04x}: rrca", self.pc);
                                self.registers
                                    .a
                                    .set(rrc_r8(self.registers.a.get(), &self.registers.flags));
                                self.registers.flags.zero.set(false);
                                self.pc += 1;
                                return 1;
                            }
                            0b010 => {
                                // ## println!("{:#04x}: rla", self.pc);
                                self.registers
                                    .a
                                    .set(rl_r8(self.registers.a.get(), &self.registers.flags));
                                self.registers.flags.zero.set(false);
                                self.pc += 1;
                                return 1;
                            }
                            0b011 => {
                                // ## println!("{:#04x}: rra", self.pc);
                                self.registers
                                    .a
                                    .set(rr_r8(self.registers.a.get(), &self.registers.flags));
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

                if opcode == 0b0111_0110 {
                    // ## println!("{:#04x}: halt", self.pc);
                    self.state = State::Halted;
                    self.pc += 1;
                    return 1;
                }

                // ## println!("{:#04x}: ld r8, r8", self.pc);
                let source = R8::from_u8(opcode & 0b0000_0111);
                let source = self.registers.get_r8(source);
                let source = match source {
                    R8OrMem::R8(r8) => r8.get(),
                    R8OrMem::Ptr(ptr) => self.mmu.read_byte(ptr),
                };

                let dest = R8::from_u8((opcode & 0b0011_1000) >> 3);
                let dest = self.registers.get_r8(dest);
                match dest {
                    R8OrMem::R8(r8) => r8.set(source),
                    R8OrMem::Ptr(ptr) => self.mmu.write_byte(ptr, source),
                };
                self.pc += 1;
                1
            }
            0b10 => {
                /* Block 2 */
                let operand = R8::from_u8(opcode & 0b0000_0111);

                let a = &self.registers.a;
                let value = self.registers.get_r8(operand);
                let value = match value {
                    R8OrMem::R8(r8) => r8.get(),
                    R8OrMem::Ptr(ptr) => self.mmu.read_byte(ptr),
                };

                match (opcode & 0b0011_1000) >> 3 {
                    0b0000 => {
                        // ## println!("{:#04x}: add a, r8", self.pc);
                        add_a_r8(a, value, &self.registers.flags);
                        self.pc += 1;
                        1
                    }
                    0b0001 => {
                        // ## println!("{:#04x}: adc a, r8", self.pc);
                        adc_a_r8(a, value, &self.registers.flags);
                        self.pc += 1;
                        1
                    }
                    0b0010 => {
                        // ## println!("{:#04x}: sub a, r8", self.pc);
                        sub_a_r8(a, value, &self.registers.flags);
                        self.pc += 1;
                        1
                    }
                    0b0011 => {
                        // ## println!("{:#04x}: sbc a, r8", self.pc);
                        sbc_a_r8(a, value, &self.registers.flags);
                        self.pc += 1;
                        1
                    }
                    0b0100 => {
                        // ## println!("{:#04x}: and a, r8", self.pc);
                        and_a_r8(a, value, &self.registers.flags);
                        self.pc += 1;
                        1
                    }
                    0b0101 => {
                        // ## println!("{:#04x}: xor a, r8", self.pc);
                        xor_a_r8(a, value, &self.registers.flags);
                        self.pc += 1;
                        1
                    }
                    0b0110 => {
                        // ## println!("{:#04x}: or a, r8", self.pc);
                        or_a_r8(a, value, &self.registers.flags);
                        self.pc += 1;
                        1
                    }
                    0b0111 => {
                        // ## println!("{:#04x}: cp a, r8", self.pc);
                        cp_a_r8(a, value, &self.registers.flags);
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
                        self.state = State::Ime;
                        // self.ime = true;
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
                        self.state = State::Running;
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
        frame.fill(0);
        let mut ticks = 0;
        self.mmu.set_window_counter(0);
        for line in 0..154 {
            while ticks < 456 {
                if self.state == State::Ime {
                    self.state = State::Running;
                    self.ime = true;
                }
                if self.state != State::Halted {
                    let duration = self.step() as u32;
                    ticks += duration;
                    if self.mmu.increment_timer(duration) {
                        self.mmu
                            .write_byte(0xFF0F, self.mmu.read_byte(0xFF0F) | 0b0000_0100);
                    }
                } else {
                    ticks += 1;
                    if self.mmu.increment_timer(1) {
                        self.mmu
                            .write_byte(0xFF0F, self.mmu.read_byte(0xFF0F) | 0b0000_0100);
                    }
                }
                if self.ime {
                    if self.mmu.read_byte(0xFFFF) & self.mmu.read_byte(0xFF0F) != 0 {
                        self.state = State::Running;
                    }
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
                    } else if self.mmu.read_byte(0xFFFF) & self.mmu.read_byte(0xFF0F) & 0b0000_0100
                        != 0
                    {
                        /* Timer interrupt */
                        self.ime = false;
                        self.mmu
                            .write_byte(0xFF0F, self.mmu.read_byte(0xFF0F) & !0b0000_0100);
                        self.mmu.write_word(self.sp - 2, self.pc);
                        self.sp -= 2;
                        self.pc = 0x50;
                    } else if self.mmu.read_byte(0xFFFF) & self.mmu.read_byte(0xFF0F) & 0b0000_1000
                        != 0
                    {
                        /* Serial interrupt */
                        self.ime = false;
                        self.mmu
                            .write_byte(0xFF0F, self.mmu.read_byte(0xFF0F) & !0b0000_1000);
                        self.mmu.write_word(self.sp - 2, self.pc);
                        self.sp -= 2;
                        self.pc = 0x58;
                    } else if self.mmu.read_byte(0xFFFF) & self.mmu.read_byte(0xFF0F) & 0b0001_0000
                        != 0
                    {
                        /* Joypad interrupt */
                        self.ime = false;
                        self.mmu
                            .write_byte(0xFF0F, self.mmu.read_byte(0xFF0F) & !0b0001_0000);
                        self.mmu.write_word(self.sp - 2, self.pc);
                        self.sp -= 2;
                        self.pc = 0x60;
                    }
                } else if self.state == State::Halted
                    && self.mmu.read_byte(0xFFFF) & self.mmu.read_byte(0xFF0F) != 0
                {
                    self.state = State::Running;
                }
            }
            ticks = 0;
            if line < 144 {
                let scx = self.mmu.read_byte(0xFF43);
                let scy = self.mmu.read_byte(0xFF42);
                draw_scanline(&self.mmu, frame, scx, scy, line);
                let window_line = self.mmu.get_window_counter();
                let (wy, wx) = self.mmu.get_window_pos();
                if self.mmu.get_window_enable() && wy <= line && wy < 144 && wx < 167 {
                    self.mmu.set_window_counter(window_line + 1);
                }
            }

            if line + 1 == self.mmu.read_byte(0xFF45)
                && self.mmu.read_byte(0xFFFF) & 0b0000_0010 != 0
            {
                self.mmu
                    .write_byte(0xFF0F, self.mmu.read_byte(0xFF0F) | 0b0000_0010);
                self.mmu
                    .write_byte(0xFF41, self.mmu.read_byte(0xFF41) | 0b0000_0100)
            } else {
                self.mmu
                    .write_byte(0xFF41, self.mmu.read_byte(0xFF41) & !0b0000_0100)
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

fn add_a_r8(a: &Cell<u8>, value: u8, flags: &Flags) {
    let (result, overflow) = a.get().overflowing_add(value);
    flags.zero.set(result == 0);
    flags.carry.set(overflow);
    flags.subtract.set(false);
    flags.half_carry.set((a.get() & 0xF) + (value & 0xF) > 0xF);
    a.set(result);
}

fn adc_a_r8(reg_a: &Cell<u8>, value: u8, flags: &Flags) {
    let a = reg_a.get() as u16;
    let value = value as u16;

    let result = if !flags.carry.get() {
        flags.half_carry.set((a & 0xF) + (value & 0xF) > 0xF);
        flags.carry.set(a + value > 0xFF);
        (a + value) as u8
    } else {
        flags.half_carry.set((a & 0xF) + (value & 0xF) + 1 > 0xF);
        flags.carry.set(a + value + 1 > 0xFF);
        (a + value + 1) as u8
    };
    flags.zero.set(result == 0);
    flags.subtract.set(false);

    reg_a.set(result);
}

fn sub_a_r8(a: &Cell<u8>, value: u8, flags: &Flags) {
    let (result, overflow) = a.get().overflowing_sub(value);
    flags.zero.set(result == 0);
    flags.carry.set(overflow);
    flags.subtract.set(true);
    flags.half_carry.set((a.get() & 0xF) < (value & 0xF));
    a.set(result);
}

fn sbc_a_r8(reg_a: &Cell<u8>, value: u8, flags: &Flags) {
    let a = reg_a.get() as u16;
    let c = flags.carry.get() as u16;
    let value = value as u16;

    let result = a.wrapping_sub(value + c) as u8;
    flags.half_carry.set(a & 0xF < (value & 0xF) + c);
    flags.carry.set(a < value + c);
    flags.zero.set(result == 0);
    flags.subtract.set(true);

    reg_a.set(result);
}

fn and_a_r8(a: &Cell<u8>, value: u8, flags: &Flags) {
    let result = a.get() & value;
    flags.zero.set(result == 0);
    flags.carry.set(false);
    flags.subtract.set(false);
    flags.half_carry.set(true);
    a.set(result);
}

fn xor_a_r8(a: &Cell<u8>, value: u8, flags: &Flags) {
    let result = a.get() ^ value;
    flags.zero.set(result == 0);
    flags.carry.set(false);
    flags.subtract.set(false);
    flags.half_carry.set(false);
    a.set(result);
}

fn or_a_r8(a: &Cell<u8>, value: u8, flags: &Flags) {
    let result = a.get() | value;
    flags.zero.set(result == 0);
    flags.carry.set(false);
    flags.subtract.set(false);
    flags.half_carry.set(false);
    a.set(result);
}

fn cp_a_r8(a: &Cell<u8>, value: u8, flags: &Flags) {
    let (result, overflow) = a.get().overflowing_sub(value);
    flags.zero.set(result == 0);
    flags.carry.set(overflow);
    flags.subtract.set(true);
    flags.half_carry.set((a.get() & 0xF) < (value & 0xF));
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

fn inc_r8(value: u8, flags: &Flags) -> u8 {
    let result = value.wrapping_add(1);
    flags.subtract.set(false);
    flags.zero.set(result == 0);
    flags.half_carry.set((value & 0xF) + 1 > 0xF);
    result
}

fn dec_r8(value: u8, flags: &Flags) -> u8 {
    let result = value.wrapping_sub(1);
    flags.subtract.set(true);
    flags.zero.set(result == 0);
    flags.half_carry.set((value & 0xF) < 1);
    result
}

fn bit_b3_r8(bit_index: u8, value: u8, flags: &Flags) {
    let bit = (value >> bit_index) & 1;
    flags.zero.set(bit == 0);
    flags.subtract.set(false);
    flags.half_carry.set(true);
}

fn res_b3_r8(bit_index: u8, value: u8) -> u8 {
    let mask = !(1 << bit_index);
    value & mask
}

fn set_b3_r8(bit_index: u8, value: u8) -> u8 {
    let mask = 1 << bit_index;
    value | mask
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

fn rlc_r8(value: u8, flags: &Flags) -> u8 {
    let result = value.rotate_left(1);

    flags.zero.set(result == 0);
    flags.carry.set(result & 0b0000_0001 == 1);
    flags.subtract.set(false);
    flags.half_carry.set(false);
    result
}

fn rrc_r8(value: u8, flags: &Flags) -> u8 {
    let result = value.rotate_right(1);

    flags.zero.set(result == 0);
    flags.carry.set((result & 0b1000_0000) >> 7 == 1);
    flags.subtract.set(false);
    flags.half_carry.set(false);
    result
}

fn rl_r8(value: u8, flags: &Flags) -> u8 {
    let result = value << 1 | flags.carry.get() as u8;
    let overflow = value & 0b1000_0000 != 0;

    flags.zero.set(result == 0);
    flags.carry.set(overflow);
    flags.subtract.set(false);
    flags.half_carry.set(false);

    result
}

fn rr_r8(value: u8, flags: &Flags) -> u8 {
    let result = value >> 1 | (flags.carry.get() as u8) << 7;
    let overflow = value & 0b0000_0001 != 0;

    flags.zero.set(result == 0);
    flags.carry.set(overflow);
    flags.subtract.set(false);
    flags.half_carry.set(false);

    result
}

fn cpl(a: &Cell<u8>, flags: &Flags) {
    a.set(!a.get());
    flags.subtract.set(true);
    flags.half_carry.set(true);
}

fn swap_r8(value: u8, flags: &Flags) -> u8 {
    let result = value.rotate_left(4) | value.rotate_right(4);

    flags.zero.set(result == 0);
    flags.carry.set(false);
    flags.subtract.set(false);
    flags.half_carry.set(false);

    result
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

fn srl_r8(value: u8, flags: &Flags) -> u8 {
    let result = value >> 1;
    flags.carry.set(value & 0b0000_0001 == 1);

    flags.zero.set(result == 0);
    flags.subtract.set(false);
    flags.half_carry.set(false);

    result
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

fn sla_r8(value: u8, flags: &Flags) -> u8 {
    flags.carry.set(value & 0b1000_0000 != 0);
    let result = value << 1;

    flags.zero.set(result == 0);
    flags.subtract.set(false);
    flags.half_carry.set(false);
    result
}

fn sra_r8(value: u8, flags: &Flags) -> u8 {
    flags.carry.set(value & 0b0000_0001 == 1);
    let result = (value & 0b1000_0000) | (value >> 1);

    flags.zero.set(result == 0);
    flags.subtract.set(false);
    flags.half_carry.set(false);
    result
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
