use std::collections::HashMap;

use crate::{
    bus::Bus,
    cartridge::Rom,
    opcodes::{self},
};

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPage_X,
    ZeroPage_Y,
    Absolute,
    Absolute_X,
    Absolute_Y,
    Indirect_X,
    Indirect_Y,
    NoneAddressing,
}

pub enum FlgCodes {
    CARRY,             // 0b0000_0001
    ZERO,              // 0b0000_0010
    INTERRUPT_DISABLE, // 0b0000_0100
    DECIMAL_MODE,      // 0b0000_1000
    BREAK,             // 0b0001_0000
    RESERVED,          // 0b0010_0000
    OVERFLOW,          // 0b0100_0000
    NEGATIV,           // 0b1000_0000
}

pub enum REGISTER {
    REGISTER_A,
    REGISTER_X,
    REGISTER_Y,
}

pub trait Mem {
    fn mem_read(&self, addr: u16) -> u8;

    fn mem_write(&mut self, addr: u16, data: u8);

    fn mem_read_u16(&self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }
}

impl Mem for CPU {
    fn mem_read(&self, addr: u16) -> u8 {
        self.bus.mem_read(addr)
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.bus.mem_write(addr, data)
    }
    fn mem_read_u16(&self, pos: u16) -> u16 {
        self.bus.mem_read_u16(pos)
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        self.bus.mem_write_u16(pos, data)
    }
}

const STACK: u16 = 0x0100;
const STACK_RESET: u8 = 0xfd;

pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: u8,
    pub program_counter: u16,
    pub stack_pointer: u8,
    pub bus: Bus,
}

impl CPU {
    pub fn new(bus: Bus) -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: 0b100100,
            program_counter: 0,
            stack_pointer: STACK_RESET,
            bus: bus,
        }
    }

    fn adc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        let sum = self.register_a as u16 + value as u16 + self.get_flg(&FlgCodes::CARRY) as u16;
        self.set_flg(&FlgCodes::CARRY, if sum > 0xFF { 1 } else { 0 });

        let result = (sum % 256) as u8;
        self.set_flg(
            &FlgCodes::OVERFLOW,
            if ((value & 0x80) == (self.register_a & 0x80)) & (result & 0x80 != value & 0x80) {
                1
            } else {
                0
            },
        );
        // set_register_a
        self.register_a = result;
        self.update_zero_and_negative_flags(self.register_a);
    }
    fn and(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_a &= value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn asl_accumulator(&mut self) {
        let value = self.register_a;
        self.set_flg(&FlgCodes::CARRY, if value >> 7 == 0 { 0 } else { 1 });

        self.register_a = value << 1;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn asl(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.set_flg(&FlgCodes::CARRY, if value >> 7 == 0 { 0 } else { 1 });

        self.mem_write(addr, value << 1);
        self.update_zero_and_negative_flags(value << 1);
    }

    fn bit(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        let result = self.register_a & value;

        self.set_flg(&FlgCodes::ZERO, if result == 0 { 1 } else { 0 });
        self.set_flg(&FlgCodes::OVERFLOW, result >> 6 & 1);
        self.set_flg(&FlgCodes::NEGATIV, result >> 7 & 1);
    }

    fn cmp(&mut self, mode: &AddressingMode, compare_with: u8) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.set_flg(&FlgCodes::CARRY, if compare_with >= value { 1 } else { 0 });
        self.update_zero_and_negative_flags(compare_with.wrapping_sub(value))
    }

    fn dec(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        let result = value.wrapping_sub(1);

        self.mem_write(addr, result);
        self.update_zero_and_negative_flags(result)
    }

    fn dex(&mut self, mode: &AddressingMode) {
        let result = self.register_x.wrapping_sub(1);

        self.register_x = result;
        self.update_zero_and_negative_flags(result)
    }
    fn dey(&mut self, mode: &AddressingMode) {
        let result = self.register_y.wrapping_sub(1);

        self.register_y = result;
        self.update_zero_and_negative_flags(result)
    }

    fn inc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        let result = value.wrapping_add(1);

        self.mem_write(addr, result);
        self.update_zero_and_negative_flags(result)
    }

    fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_x);
    }
    fn iny(&mut self) {
        self.register_y = self.register_y.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn eor(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        let result = self.register_a ^ value;

        self.register_a = result;
        self.update_zero_and_negative_flags(result);
    }

    fn lsr_accumulator(&mut self) {
        let value = self.register_a;
        self.set_flg(&FlgCodes::CARRY, if value & 1 == 0 { 0 } else { 1 });

        self.register_a = value >> 1;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn lsr(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.set_flg(&FlgCodes::CARRY, if value >> 7 == 0 { 0 } else { 1 });

        self.mem_write(addr, value >> 1);
        self.update_zero_and_negative_flags(value >> 1);
    }

    fn rol_accumulator(&mut self) {
        let value = self.register_a;
        let old_carry = self.get_flg(&FlgCodes::CARRY);
        self.set_flg(&FlgCodes::CARRY, if value >> 7 == 0 { 0 } else { 1 });

        self.register_a = (value << 1) | old_carry;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn rol(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        let old_carry = self.get_flg(&FlgCodes::CARRY);

        self.set_flg(&FlgCodes::CARRY, if value >> 7 == 0 { 0 } else { 1 });

        self.mem_write(addr, (value << 1) | old_carry);
        self.update_zero_and_negative_flags((value << 1) | old_carry);
    }
    fn ror_accumulator(&mut self) {
        let value = self.register_a;
        let old_carry = self.get_flg(&FlgCodes::CARRY);
        self.set_flg(&FlgCodes::CARRY, if value >> 7 == 0 { 0 } else { 1 });

        self.register_a = (value >> 1) | (old_carry << 7);
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn ror(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        let old_carry = self.get_flg(&FlgCodes::CARRY);

        self.set_flg(&FlgCodes::CARRY, if value >> 7 == 0 { 0 } else { 1 });

        self.mem_write(addr, (value >> 1) | (old_carry << 7));
        self.update_zero_and_negative_flags((value >> 1) | (old_carry << 7));
    }

    fn sbc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = ((self.mem_read(addr) as i8).wrapping_neg().wrapping_sub(1)) as u8;

        // A - M - (1 - C) = A + (-M) -1 + C
        let sum = self.register_a as u16 + value as u16 + self.get_flg(&FlgCodes::CARRY) as u16;
        self.set_flg(&FlgCodes::CARRY, if sum > 0xFF { 1 } else { 0 });

        let result = (sum % 256) as u8;
        self.set_flg(
            &FlgCodes::OVERFLOW,
            if ((value & 0x80) == (self.register_a & 0x80)) & (result & 0x80 != value & 0x80) {
                1
            } else {
                0
            },
        );
        // set_register_a
        self.register_a = result;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn ora(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        let result = self.register_a | value;

        self.register_a = result;
        self.update_zero_and_negative_flags(result);
    }

    fn ld(&mut self, mode: &AddressingMode, kind: &REGISTER) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        match kind {
            REGISTER::REGISTER_A => self.register_a = value,
            REGISTER::REGISTER_X => self.register_x = value,
            REGISTER::REGISTER_Y => self.register_y = value,
        }

        self.update_zero_and_negative_flags(value);
    }

    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }
    fn txa(&mut self) {
        self.register_a = self.register_x;
        self.update_zero_and_negative_flags(self.register_a);
    }
    fn tay(&mut self) {
        self.register_y = self.register_a;
        self.update_zero_and_negative_flags(self.register_y);
    }
    fn tya(&mut self) {
        self.register_a = self.register_y;
        self.update_zero_and_negative_flags(self.register_a);
    }
    fn tsx(&mut self) {
        self.register_x = self.stack_pointer;
        self.update_zero_and_negative_flags(self.register_x);
    }
    fn txs(&mut self) {
        self.stack_pointer = self.register_x;
    }

    fn store(&mut self, mode: &AddressingMode, kind: &REGISTER) {
        let addr = self.get_operand_address(mode);
        match kind {
            REGISTER::REGISTER_A => self.mem_write(addr, self.register_a),
            REGISTER::REGISTER_X => self.mem_write(addr, self.register_x),
            REGISTER::REGISTER_Y => self.mem_write(addr, self.register_y),
        }
    }
    fn stack_push(&mut self, data: u8) {
        self.mem_write((STACK as u16) + self.stack_pointer as u16, data);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1)
    }

    fn stack_pop(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        self.mem_read((STACK as u16) + self.stack_pointer as u16)
    }

    fn stack_push_u16(&mut self, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.stack_push(hi);
        self.stack_push(lo);
    }

    fn stack_pop_u16(&mut self) -> u16 {
        let lo = self.stack_pop() as u16;
        let hi = self.stack_pop() as u16;

        hi << 8 | lo
    }

    fn branch(&mut self, condition: bool) {
        if condition {
            let jump = self.mem_read(self.program_counter) as i8;
            let jump_addr = self
                .program_counter
                .wrapping_add(1)
                .wrapping_add(jump as u16);
            self.program_counter = jump_addr;
        }
    }

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        if result == 0 {
            self.status = self.status | 0b0000_0010;
        } else {
            self.status = self.status & 0b1111_1101;
        }

        if result & 0b1000_0000 != 0 {
            self.status = self.status | 0b1000_0000;
        } else {
            self.status = self.status & 0b0111_1111;
        }
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.status = 0b100100;

        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    pub fn load(&mut self, program: Vec<u8>) {
        for i in 0..(program.len() as u16) {
            self.mem_write(0x8000 + i, program[i as usize])
        }
        self.mem_write_u16(0xFFFC, 0x8000);
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run();
    }

    fn get_flg(&self, flgCode: &FlgCodes) -> u8 {
        match flgCode {
            FlgCodes::CARRY => self.status >> 0 & 1,
            FlgCodes::ZERO => self.status >> 1 & 1,
            FlgCodes::INTERRUPT_DISABLE => self.status >> 2 & 1,
            FlgCodes::DECIMAL_MODE => self.status >> 3 & 1,
            FlgCodes::BREAK => self.status >> 4 & 1,
            FlgCodes::RESERVED => self.status >> 5 & 1,
            FlgCodes::OVERFLOW => self.status >> 6 & 1,
            FlgCodes::NEGATIV => self.status >> 7 & 1,
        }
    }

    fn set_flg(&mut self, flgCode: &FlgCodes, value: u8) {
        if value == 1 {
            match flgCode {
                FlgCodes::CARRY => self.status |= 1 << 0,
                FlgCodes::ZERO => self.status |= 1 << 1,
                FlgCodes::INTERRUPT_DISABLE => self.status |= 1 << 2,
                FlgCodes::DECIMAL_MODE => self.status |= 1 << 3,
                FlgCodes::BREAK => self.status |= 1 << 4,
                FlgCodes::RESERVED => self.status |= 1 << 5,
                FlgCodes::OVERFLOW => self.status |= 1 << 6,
                FlgCodes::NEGATIV => self.status |= 1 << 7,
            }
        } else {
            match flgCode {
                FlgCodes::CARRY => self.status = self.status & !(1 << 0),
                FlgCodes::ZERO => self.status = self.status & !(1 << 1),
                FlgCodes::INTERRUPT_DISABLE => self.status = self.status & !(1 << 2),
                FlgCodes::DECIMAL_MODE => self.status = self.status & !(1 << 3),
                FlgCodes::BREAK => self.status = self.status & !(1 << 4),
                FlgCodes::RESERVED => self.status = self.status & !(1 << 5),
                FlgCodes::OVERFLOW => self.status = self.status & !(1 << 6),
                FlgCodes::NEGATIV => self.status = self.status & !(1 << 7),
            }
        }
    }

    // pub fn debug(&mut self, label: String) {
    //     println!("{:20}... code: {:#06x} a: {:#06x} x: {:#06x} y: {:#06x} pc: {:#06x} sp: {:#06x} status: {:#10b}", label, self.mem_read(self.program_counter), self.register_a, self.register_x, self.register_y, self.program_counter, self.stack_pointer, self.status);
    // }

    pub fn run(&mut self) {
        self.run_with_callback(|_| {});
    }

    pub fn run_with_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut CPU),
    {
        let ref opcodes: HashMap<u8, &'static opcodes::OpCode> = *opcodes::OPCODES_MAP;
        loop {
            let code = self.mem_read(self.program_counter);
            self.program_counter += 1;
            let program_counter_state = self.program_counter;
            let opcode = opcodes.get(&code).unwrap();

            match code {
                /* Transfer Instructions */
                /* LDA */
                0xA9 | 0xA5 | 0xB5 | 0xAD | 0xBD | 0xB9 | 0xA1 | 0xB1 => {
                    self.ld(&opcode.mode, &REGISTER::REGISTER_A);
                }
                /* LDX */
                0xA2 | 0xA6 | 0xB6 | 0xAE | 0xBE => {
                    self.ld(&opcode.mode, &REGISTER::REGISTER_X);
                }
                /* LDY */
                0xA0 | 0xA4 | 0xB4 | 0xAB | 0xBC => {
                    self.ld(&opcode.mode, &REGISTER::REGISTER_Y);
                }
                /* STA */
                0x85 | 0x95 | 0x8D | 0x9D | 0x99 | 0x81 | 0x91 => {
                    self.store(&opcode.mode, &REGISTER::REGISTER_A);
                }
                /* STX */
                0x86 | 0x96 | 0x8E => {
                    self.store(&opcode.mode, &REGISTER::REGISTER_X);
                }
                /* STY */
                0x84 | 0x94 | 0x8C => {
                    self.store(&opcode.mode, &REGISTER::REGISTER_Y);
                }
                /* TAX */
                0xAA => self.tax(),
                /* TXA */
                0x8A => self.txa(),
                /* TAY */
                0xA8 => self.tay(),
                /* TYA */
                0x98 => self.tya(),
                /* TSX */
                0xBA => self.tsx(),
                /* TXS */
                0x9A => self.txs(),
                /* Arithmetic Instructions */
                /* ADC */
                0x69 | 0x65 | 0x75 | 0x6D | 0x7D | 0x79 | 0x61 | 0x71 => {
                    self.adc(&opcode.mode);
                }
                /* AND */
                0x29 | 0x25 | 0x35 | 0x2D | 0x3D | 0x39 | 0x21 | 0x31 => self.and(&opcode.mode),
                /* ASL Immediate */
                0x0A => self.asl_accumulator(),
                /* ASL others */
                0x06 | 0x16 | 0x0E | 0x1E => self.asl(&opcode.mode),
                /* BIT */
                0x24 | 0x2C => self.bit(&opcode.mode),
                /* CMP */
                0xC9 | 0xC5 | 0xD5 | 0xCD | 0xDD | 0xD9 | 0xC1 | 0xD1 => {
                    self.cmp(&opcode.mode, self.register_a)
                }
                /* CMX */
                0xE0 | 0xE4 | 0xEC => self.cmp(&opcode.mode, self.register_x),
                /* CMY */
                0xC0 | 0xC4 | 0xCC => self.cmp(&opcode.mode, self.register_y),
                /* DEC */
                0xC6 | 0xD6 | 0xCE | 0xDE => self.dec(&opcode.mode),
                /* DEX */
                0xCA => self.dex(&opcode.mode),
                /* DEY */
                0x88 => self.dey(&opcode.mode),
                /* EOR */
                0x49 | 0x45 | 0x55 | 0x4D | 0x5D | 0x59 | 0x41 | 0x51 => self.eor(&opcode.mode),
                /* INC */
                0xE6 | 0xF6 | 0xEE | 0xFE => self.inc(&opcode.mode),
                /* INX */
                0xE8 => self.inx(),
                /* INY */
                0xC8 => self.iny(),
                /* LSR_accumulator */
                0x4A => self.lsr_accumulator(),
                /* LSR others*/
                0x46 | 0x56 | 0x4E | 0x5E => self.lsr(&opcode.mode),
                /* ORA */
                0x09 | 0x05 | 0x15 | 0x0D | 0x1D | 0x19 | 0x01 | 0x11 => self.ora(&opcode.mode),
                /* ROL_accumulator */
                0x2A => self.rol_accumulator(),
                /* ROL others*/
                0x26 | 0x36 | 0x2E | 0x3E => self.rol(&opcode.mode),
                /* ROR_accumulator */
                0x6A => self.ror_accumulator(),
                /* ROR others*/
                0x66 | 0x76 | 0x6E | 0x7E => self.ror(&opcode.mode),
                /* SBC */
                0xE9 | 0xE5 | 0xF5 | 0xED | 0xFD | 0xF9 | 0xE1 | 0xF1 => {
                    self.sbc(&opcode.mode);
                }
                /* Stack Instructions */
                /* PHA */
                0x48 => self.stack_push(self.register_a),
                /* PHP */
                0x08 => self.stack_push(self.status),
                /* PLA */
                0x68 => {
                    let value = self.stack_pop();
                    self.register_a = value;
                    self.update_zero_and_negative_flags(value);
                }
                /* PLP */
                0x28 => {
                    let value = self.stack_pop();
                    self.status = value;
                    self.update_zero_and_negative_flags(self.status);
                }
                /* Jump Instructions */
                /* JMP */
                0x4C => {
                    let mem_address = self.mem_read_u16(self.program_counter);
                    self.program_counter = mem_address;
                }
                /* JMP Indirect */
                0x6C => {
                    let mem_address = self.mem_read_u16(self.program_counter);

                    let indirect_ref = if mem_address & 0x00FF == 0x00FF {
                        let lo = self.mem_read(mem_address);
                        let hi = self.mem_read(mem_address & 0xFF00);
                        (hi as u16) << 8 | (lo as u16)
                    } else {
                        self.mem_read_u16(mem_address)
                    };

                    self.program_counter = indirect_ref;
                }
                /* JSR */
                0x20 => {
                    self.stack_push_u16(self.program_counter + 2 - 1);
                    let target_address = self.mem_read_u16(self.program_counter);
                    self.program_counter = target_address
                }
                /* RTS */
                0x60 => {
                    self.program_counter = self.stack_pop_u16() + 1;
                }
                /* RTI */
                0x40 => {
                    self.status = self.stack_pop();
                    self.program_counter = self.stack_pop_u16();
                }
                /* Branching Instructions */
                /* BCC */
                0x90 => self.branch(self.get_flg(&FlgCodes::CARRY) == 0),
                /* BCS */
                0xB0 => self.branch(self.get_flg(&FlgCodes::CARRY) == 1),
                /* BEQ */
                0xF0 => self.branch(self.get_flg(&FlgCodes::ZERO) == 1),
                /* BMI */
                0x30 => self.branch(self.get_flg(&FlgCodes::NEGATIV) == 1),
                /* BNE */
                0xD0 => self.branch(self.get_flg(&FlgCodes::ZERO) == 0),
                /* BPL */
                0x10 => self.branch(self.get_flg(&FlgCodes::NEGATIV) == 0),
                /* BVC */
                0x50 => self.branch(self.get_flg(&FlgCodes::OVERFLOW) == 0),
                /* BVS */
                0x70 => self.branch(self.get_flg(&FlgCodes::OVERFLOW) == 1),
                /* Flag Modification Instructions */
                /* CLC */
                0x18 => self.set_flg(&FlgCodes::CARRY, 0),
                /* CLD */
                0xD8 => self.set_flg(&FlgCodes::DECIMAL_MODE, 0),
                /* CLI */
                0x58 => self.set_flg(&FlgCodes::INTERRUPT_DISABLE, 0),
                /* CLV */
                0xB8 => self.set_flg(&FlgCodes::OVERFLOW, 0),
                /* SEC */
                0x38 => self.set_flg(&FlgCodes::CARRY, 1),
                /* SED */
                0xF8 => self.set_flg(&FlgCodes::DECIMAL_MODE, 1),
                /* SEI */
                0x78 => self.set_flg(&FlgCodes::INTERRUPT_DISABLE, 1),
                /* The Other Instructions */
                /* BRK */
                0x00 => return,
                /* NOP */
                0xEA => {}
                _ => {
                    todo!()
                }
            }
            if program_counter_state == self.program_counter {
                self.program_counter += (opcode.len - 1) as u16
            };
            callback(self);
        }
    }

    fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.program_counter,

            AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16,

            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),

            AddressingMode::ZeroPage_X => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_x) as u16;
                addr
            }
            AddressingMode::ZeroPage_Y => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_y) as u16;
                addr
            }

            AddressingMode::Absolute_X => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_x as u16);
                addr
            }
            AddressingMode::Absolute_Y => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_y as u16);
                addr
            }
            AddressingMode::Indirect_X => {
                let base = self.mem_read(self.program_counter);

                let ptr: u8 = (base as u8).wrapping_add(self.register_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)
            }
            AddressingMode::Indirect_Y => {
                let base = self.mem_read(self.program_counter);

                let lo = self.mem_read(base as u16);
                let hi = self.mem_read(base.wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let deref = deref_base.wrapping_add(self.register_y as u16);
                deref
            }
            AddressingMode::NoneAddressing => {
                panic!("mode {:?} is not supported", mode);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0xa9, 0x05, 0x00]);
        cpu.reset();
        cpu.status = 0;
        cpu.run();
        assert_eq!(cpu.register_a, 5);
        assert!(cpu.status & 0b0000_0010 == 0);
        assert!(cpu.status & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.status & 0b0000_0010 == 0b10);
    }

    #[test]
    fn test_0xa9_lda_negative_flag() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load_and_run(vec![0xa9, 0xff, 0x00]);
        assert!(cpu.status & 0b1000_0000 == 0b1000_0000);
    }
    #[test]
    fn test_ldx_negative_flag() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load_and_run(vec![0xa2, 0xff, 0x00]);
        assert!(cpu.status & 0b1000_0000 == 0b1000_0000);
    }
    #[test]
    fn test_ldy_negative_flag() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load_and_run(vec![0xa0, 0xff, 0x00]);
        assert!(cpu.status & 0b1000_0000 == 0b1000_0000);
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load_and_run(vec![0xa9, 0x0A, 0xaa, 0x00]);

        assert_eq!(cpu.register_x, 10)
    }

    #[test]
    fn test_5_ops_working_together() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 0xc1)
    }
    #[test]
    fn test_sta() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x85, 0x00]);
        cpu.reset();
        cpu.register_a = 0xff;
        cpu.run();
        assert_eq!(cpu.mem_read(0x00), 0xff)
    }
    #[test]
    fn test_stx() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x86, 0x00]);
        cpu.reset();
        cpu.register_x = 0xff;
        cpu.run();
        assert_eq!(cpu.mem_read(0x00), 0xff)
    }
    #[test]
    fn test_sty() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x84, 0x00]);
        cpu.reset();
        cpu.register_y = 0xff;
        cpu.run();
        assert_eq!(cpu.mem_read(0x00), 0xff)
    }
    #[test]
    fn test_tax() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0xAA]);
        cpu.reset();
        cpu.register_a = 0xff;
        cpu.run();
        assert_eq!(cpu.register_x, 0xff)
    }
    #[test]
    fn test_txa() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x8A]);
        cpu.reset();
        cpu.register_x = 0xff;
        cpu.run();
        assert_eq!(cpu.register_a, 0xff)
    }
    #[test]
    fn test_tay() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0xA8]);
        cpu.reset();
        cpu.register_a = 0xff;
        cpu.run();
        assert_eq!(cpu.register_y, 0xff)
    }
    #[test]
    fn test_tya() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x98]);
        cpu.reset();
        cpu.register_y = 0xff;
        cpu.run();
        assert_eq!(cpu.register_y, 0xff)
    }
    #[test]
    fn test_tsx() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0xBA]);
        cpu.reset();
        cpu.run();
        assert_eq!(cpu.register_x, STACK_RESET)
    }
    #[test]
    fn test_txs() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x9A]);
        cpu.reset();
        cpu.register_x = 0xff;
        cpu.run();
        assert_eq!(cpu.stack_pointer, 0xff)
    }
    #[test]
    fn test_inx_overflow() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.register_x = 0xff;
        cpu.load_and_run(vec![0xa9, 0xff, 0xaa, 0xe8, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 1)
    }

    #[test]
    fn test_lda_from_memory() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.mem_write(0x10, 0x55);

        cpu.load_and_run(vec![0xa5, 0x10, 0x00]);

        assert_eq!(cpu.register_a, 0x55);
    }

    #[test]
    fn test_adc_no_carry_and_no_overflow() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x69, 0x01]);
        cpu.reset();
        cpu.register_a = 0x01;
        cpu.status = 0;
        cpu.run();

        assert_eq!(cpu.register_a, 0x02);
        assert_eq!(cpu.status, 0b0000_0000);
    }

    #[test]
    fn test_adc_has_carry() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x69, 0x01]);
        cpu.reset();
        cpu.register_a = 0x01;
        cpu.status = 0x01;
        cpu.run();

        assert_eq!(cpu.register_a, 0x03);
        assert_eq!(cpu.status, 0x00);
    }

    #[test]
    fn test_adc_occurs_carry() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x69, 0xd0]);
        cpu.reset();
        cpu.register_a = 0x50;
        cpu.status = 0x00;
        cpu.run();

        assert_eq!(cpu.register_a, 0x20);
        assert_eq!(cpu.status, 0x01);
    }

    #[test]
    fn test_adc_occurs_overflow_plus() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x69, 0x50]);
        cpu.reset();
        cpu.register_a = 0x50;
        cpu.status = 0x00;
        cpu.run();

        assert_eq!(cpu.register_a, 0xA0);
        assert_eq!(cpu.status, 0xC0);
    }
    #[test]
    fn test_adc_occurs_overflow_plus_with_carry() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x69, 0x50]);
        cpu.reset();
        cpu.register_a = 0x4F;
        cpu.status = 0x01;
        cpu.run();

        assert_eq!(cpu.register_a, 0xA0);
        assert_eq!(cpu.status, 0xC0);
    }
    #[test]
    fn test_adc_occurs_no_overflow() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x69, 0x7f]);
        cpu.reset();
        cpu.register_a = 0x82;
        cpu.status = 0x00;
        cpu.run();

        assert_eq!(cpu.register_a, 0x01);
        assert_eq!(cpu.status, 0x01);
    }

    #[test]
    fn test_and() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x29, 0x01]);
        cpu.reset();
        cpu.register_a = 0x01;
        cpu.status = 0x00;
        cpu.run();

        assert_eq!(cpu.register_a, 0x01);
        assert_eq!(cpu.status, 0x00);
    }

    #[test]
    fn test_and_occurs_register_a_0() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x29, 0x00]);
        cpu.reset();
        cpu.register_a = 0x01;
        cpu.status = 0x00;
        cpu.run();

        assert_eq!(cpu.register_a, 0x00);
        assert_eq!(cpu.status, 0x02);
    }

    #[test]
    fn test_asl_immediate() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x0A]);
        cpu.reset();
        cpu.register_a = 0x01;
        cpu.status = 0x00;
        cpu.run();

        assert_eq!(cpu.register_a, 0x02);
        assert_eq!(cpu.status, 0x00);
    }

    #[test]
    fn test_asl_accumulate_occurs_carry() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x0A]);
        cpu.reset();
        cpu.register_a = 0x80;
        cpu.status = 0x00;
        cpu.run();

        assert_eq!(cpu.register_a, 0x00);
        assert_eq!(cpu.status, 0x03);
    }

    #[test]
    fn test_asl_zeropage() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.mem_write(0x10, 0x01);
        cpu.load(vec![0x16, 0x10]);
        cpu.reset();
        cpu.status = 0;
        cpu.run();

        assert_eq!(cpu.mem_read(0x10), 0x02);
        assert_eq!(cpu.status, 0x00);
    }

    #[test]
    fn test_asl_register_x_occurs_carry() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.mem_write(0x10, 0x80);
        cpu.load(vec![0x16, 0x10]);
        cpu.reset();
        cpu.status = 0;
        cpu.run();

        assert_eq!(cpu.mem_read(0x10), 0x00);
        assert_eq!(cpu.status, 0x03);
    }

    #[test]
    fn test_bit_zero() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.mem_write(0x00, 0x80);
        cpu.load(vec![0x24, 0x00]);
        cpu.reset();
        cpu.status = 0x00;
        cpu.register_a = 0x01;
        cpu.run();

        assert_eq!(cpu.status, 0x2);
    }

    #[test]
    fn test_bit_zero_neg_overflow_flags() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.mem_write(0x00, 0xc0);
        cpu.load(vec![0x24, 0x00]);
        cpu.reset();
        cpu.status = 0x00;
        cpu.register_a = 0xc0;
        cpu.run();

        assert_eq!(cpu.status, 0xc0);
    }

    #[test]
    fn test_cmp_registera_larger() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0xC9, 0x00]);
        cpu.reset();
        cpu.status = 0x00;
        cpu.register_a = 0x01;
        cpu.run();

        assert_eq!(cpu.status, 0x01);
    }

    #[test]
    fn test_cmp_registera_equal() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0xC9, 0x01]);
        cpu.reset();
        cpu.status = 0x00;
        cpu.register_a = 0x01;
        cpu.run();

        assert_eq!(cpu.status, 0x03);
    }

    #[test]
    fn test_cmp_registera_smaller() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0xC9, 0x01]);
        cpu.reset();
        cpu.status = 0x00;
        cpu.register_a = 0x00;
        cpu.run();

        assert_eq!(cpu.status, 0x80);
    }

    #[test]
    fn test_cmp_registerx_larger() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0xE0, 0x00]);
        cpu.reset();
        cpu.status = 0x00;
        cpu.register_x = 0x01;
        cpu.run();

        assert_eq!(cpu.status, 0x01);
    }

    #[test]
    fn test_cmp_registerx_equal() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0xE0, 0x01]);
        cpu.reset();
        cpu.status = 0x00;
        cpu.register_x = 0x01;
        cpu.run();

        assert_eq!(cpu.status, 0x03);
    }

    #[test]
    fn test_cmp_registerx_smaller() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0xE0, 0x01]);
        cpu.reset();
        cpu.status = 0x00;
        cpu.register_x = 0x00;
        cpu.run();

        assert_eq!(cpu.status, 0x80);
    }

    #[test]
    fn test_cmp_registery_larger() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0xC0, 0x00]);
        cpu.reset();
        cpu.status = 0x00;
        cpu.register_y = 0x01;
        cpu.run();

        assert_eq!(cpu.status, 0x01);
    }

    #[test]
    fn test_cmp_registery_equal() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0xC0, 0x01]);
        cpu.reset();
        cpu.status = 0x00;
        cpu.register_y = 0x01;
        cpu.run();

        assert_eq!(cpu.status, 0x03);
    }

    #[test]
    fn test_cmp_registery_smaller() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0xC0, 0x01]);
        cpu.reset();
        cpu.status = 0x00;
        cpu.register_y = 0x00;
        cpu.run();

        assert_eq!(cpu.status, 0x80);
    }
    #[test]
    fn test_dec() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.mem_write(0x00, 0x01);
        cpu.load(vec![0xC6, 0x00]);
        cpu.reset();
        cpu.status = 0x00;
        cpu.run();

        assert_eq!(cpu.status, 0x02);
    }
    #[test]
    fn test_dex() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0xCA]);
        cpu.reset();
        cpu.status = 0x00;
        cpu.register_x = 0x01;
        cpu.run();

        assert_eq!(cpu.status, 0x02);
    }
    #[test]
    fn test_dey() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x88]);
        cpu.reset();
        cpu.status = 0x00;
        cpu.register_y = 0x01;
        cpu.run();

        assert_eq!(cpu.status, 0x02);
    }
    #[test]
    fn test_eor() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x49, 0x80]);
        cpu.reset();
        cpu.status = 0x00;
        cpu.register_a = 0x01;
        cpu.run();

        assert_eq!(cpu.register_a, 0x81);
    }

    #[test]
    fn test_inc() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.mem_write(0x00, 0x01);
        cpu.load(vec![0xE6, 0x00]);
        cpu.reset();
        cpu.status = 0x00;
        cpu.run();

        assert_eq!(cpu.mem_read(0x00), 0x02);
    }
    #[test]
    fn test_inx() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0xE8]);
        cpu.reset();
        cpu.status = 0x00;
        cpu.register_x = 0x01;
        cpu.run();

        assert_eq!(cpu.register_x, 0x02);
    }
    #[test]
    fn test_iny() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0xC8]);
        cpu.reset();
        cpu.status = 0x00;
        cpu.register_y = 0x01;
        cpu.run();

        assert_eq!(cpu.register_y, 0x02);
    }
    #[test]
    fn test_lsr_accumulator() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x4A]);
        cpu.reset();
        cpu.register_a = 0x3;
        cpu.status = 0x00;
        cpu.run();

        assert_eq!(cpu.register_a, 0x01);
        assert_eq!(cpu.status, 0x01);
    }

    #[test]
    fn test_lsr_zeropage() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.mem_write(0x10, 0x02);
        cpu.load(vec![0x46, 0x10]);
        cpu.reset();
        cpu.status = 0;
        cpu.run();

        assert_eq!(cpu.mem_read(0x10), 0x01);
        assert_eq!(cpu.status, 0x00);
    }
    #[test]
    fn test_ora() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x09, 0x02]);
        cpu.reset();
        cpu.status = 0x00;
        cpu.register_a = 0x01;
        cpu.run();

        assert_eq!(cpu.register_a, 0x03);
    }

    #[test]
    fn test_rol_accumulator() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x2A]);
        cpu.reset();
        cpu.register_a = 0b0000_0010;
        cpu.status = 0x01;
        cpu.run();

        assert_eq!(cpu.register_a, 0b0000_0101);
        assert_eq!(cpu.status, 0x00);
    }

    #[test]
    fn test_rol_zeropage() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.mem_write(0x10, 0b0000_0001);
        cpu.load(vec![0x26, 0x10]);
        cpu.reset();
        cpu.status = 0x01;
        cpu.run();

        assert_eq!(cpu.mem_read(0x10), 0x03);
        assert_eq!(cpu.status, 0x00);
    }

    #[test]
    fn test_ror_accumulator() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x6A]);
        cpu.reset();
        cpu.register_a = 0b1000_0000;
        cpu.status = 0x01;
        cpu.run();

        assert_eq!(cpu.register_a, 0b1100_0000);
        assert_eq!(cpu.status, 0x81);
    }

    #[test]
    fn test_ror_zeropage() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.mem_write(0x10, 0b1000_0000);
        cpu.load(vec![0x66, 0x10]);
        cpu.reset();
        cpu.status = 0x01;
        cpu.run();

        assert_eq!(cpu.mem_read(0x10), 0b1100_0000);
        // assert_eq!(cpu.status, 0x81);
    }

    #[test]
    fn test_sbc_no_carry_and_no_overflow() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0xE9, 0xf0]);
        cpu.reset();
        cpu.register_a = 0x50;
        cpu.status = 0x00;
        cpu.run();

        assert_eq!(cpu.register_a, 0x5f);
        assert_eq!(cpu.status, 0b0000_0000);
    }

    #[test]
    fn test_sbc_has_carry() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0xE9, 0xf0]);
        cpu.reset();
        cpu.register_a = 0x50;
        cpu.status = 0x01;
        cpu.run();

        assert_eq!(cpu.register_a, 0x60);
        assert_eq!(cpu.status, 0b0000_0000);
    }

    #[test]
    fn test_sbc_occurs_carry() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0xE9, 0x30]);
        cpu.reset();
        cpu.register_a = 0x50;
        cpu.status = 0x00;
        cpu.run();

        assert_eq!(cpu.register_a, 0x1F);
        assert_eq!(cpu.status, 0b0000_0001);
    }

    #[test]
    fn test_sbc_occurs_overflow_plus() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0xE9, 0xb0]);
        cpu.reset();
        cpu.register_a = 0x50;
        cpu.status = 0x00;
        cpu.run();

        assert_eq!(cpu.register_a, 0x9f);
        assert_eq!(cpu.status, 0b1100_0000);
    }
    #[test]
    fn test_sbc_occurs_overflow_plus_with_carry() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0xE9, 0xb0]);
        cpu.reset();
        cpu.register_a = 0x50;
        cpu.status = 0x01;
        cpu.run();

        assert_eq!(cpu.register_a, 0xa0);
        assert_eq!(cpu.status, 0b1100_0000);
    }
    #[test]
    fn test_pha() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x48]);
        cpu.reset();
        cpu.register_a = 0xff;
        cpu.run();

        assert_eq!(
            cpu.mem_read((STACK as u16) + cpu.stack_pointer.wrapping_add(1) as u16),
            0xff
        );
    }
    #[test]
    fn test_php() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x08]);
        cpu.reset();
        cpu.status = 0xff;
        cpu.run();

        assert_eq!(
            cpu.mem_read((STACK as u16) + cpu.stack_pointer.wrapping_add(1) as u16),
            0xff
        );
    }
    #[test]
    fn test_clc_cld_cli_clv() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x18, 0xD8, 0x58, 0xB8]);
        cpu.reset();
        cpu.status = 0b0100_1101;
        cpu.run();

        assert_eq!(cpu.status, 0);
    }
    #[test]
    fn test_sec_sed_sei() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x38, 0xF8, 0x78]);
        cpu.reset();
        cpu.status = 0;
        cpu.run();

        assert_eq!(cpu.status, 0b0000_1101);
    }
    #[test]
    fn test_bcc() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x90, 0x02, 0x00, 0x00, 0xE8, 0x00]);
        cpu.reset();
        cpu.status = 0b0000_0000;
        cpu.run();

        assert_eq!(cpu.register_x, 1);
    }
    #[test]
    fn test_bcc_with_carry() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x90, 0x02, 0x00, 0x00, 0xE8, 0x00]);
        cpu.reset();
        cpu.status = 0b0000_0001;
        cpu.run();

        assert_eq!(cpu.register_x, 0);
    }
    #[test]
    fn test_bcc_negative_value() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x90, 0xfc]);
        cpu.reset();
        cpu.mem_write(0x7FFF, 0x00);
        cpu.mem_write(0x7FFE, 0xe8);
        cpu.status = 0b0000_0000;
        cpu.run();

        assert_eq!(cpu.register_x, 1);
    }
    #[test]
    fn test_bcs() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0xB0, 0x02, 0x00, 0x00, 0xE8, 0x00]);
        cpu.reset();
        cpu.status = 0b0000_0000;
        cpu.run();

        assert_eq!(cpu.register_x, 0);
    }
    #[test]
    fn test_bcs_with_carry() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0xB0, 0x02, 0x00, 0x00, 0xE8, 0x00]);
        cpu.reset();
        cpu.status = 0b0000_0001;
        cpu.run();

        assert_eq!(cpu.register_x, 1);
    }
    #[test]
    fn test_bcs_negative_value() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0xB0, 0xFC]);
        cpu.reset();
        cpu.mem_write_u16(0x7FFF, 0x00);
        cpu.mem_write_u16(0x7FFE, 0xe8);
        cpu.status = 0b0000_0001;
        cpu.run();

        assert_eq!(cpu.register_x, 0);
    }
    #[test]
    fn test_beq() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0xF0, 0x02, 0x00, 0x00, 0xE8, 0x00]);
        cpu.reset();
        cpu.status = 0b0000_0010;
        cpu.run();

        assert_eq!(cpu.register_x, 1);
    }
    #[test]
    fn test_beq_with_carry() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0xF0, 0x02, 0x00, 0x00, 0xE8, 0x00]);
        cpu.reset();
        cpu.status = 0b0000_0000;
        cpu.run();

        assert_eq!(cpu.register_x, 0);
    }
    #[test]
    fn test_beq_negative_value() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0xF0, 0xFC]);
        cpu.reset();
        cpu.mem_write(0x7FFF, 0x00);
        cpu.mem_write(0x7FFE, 0xE8);
        cpu.status = 0b0000_0010;
        cpu.run();

        assert_eq!(cpu.register_x, 1);
    }
    #[test]
    fn test_bmi() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x30, 0x02, 0x00, 0x00, 0xE8, 0x00]);
        cpu.reset();
        cpu.status = 0b0000_0000;
        cpu.run();

        assert_eq!(cpu.register_x, 0);
    }
    #[test]
    fn test_bmi_with_carry() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x30, 0x02, 0x00, 0x00, 0xE8, 0x00]);
        cpu.reset();
        cpu.status = 0b1000_0000;
        cpu.run();

        assert_eq!(cpu.register_x, 1);
    }
    #[test]
    fn test_bmi_negative_value() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x30, 0xFC]);
        cpu.reset();
        cpu.mem_write(0x7FFF, 0x00);
        cpu.mem_write(0x7FFE, 0xE8);
        cpu.status = 0b1000_0000;
        cpu.run();

        assert_eq!(cpu.register_x, 1);
    }
    #[test]
    fn test_bne() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0xD0, 0x02, 0x00, 0x00, 0xE8, 0x00]);
        cpu.reset();
        cpu.status = 0b0000_0000;
        cpu.run();

        assert_eq!(cpu.register_x, 1);
    }
    #[test]
    fn test_bne_with_carry() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0xD0, 0x02, 0x00, 0x00, 0xE8, 0x00]);
        cpu.reset();
        cpu.status = 0b0000_0010;
        cpu.run();

        assert_eq!(cpu.register_x, 0);
    }
    #[test]
    fn test_bne_negative_value() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0xD0, 0xFC]);
        cpu.reset();
        cpu.mem_write(0x7FFF, 0x00);
        cpu.mem_write(0x7FFE, 0xE8);
        cpu.status = 0b0000_0000;
        cpu.run();

        assert_eq!(cpu.register_x, 1);
    }
    #[test]
    fn test_bpl() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x10, 0x02, 0x00, 0x00, 0xE8, 0x00]);
        cpu.reset();
        cpu.status = 0b0000_0000;
        cpu.run();

        assert_eq!(cpu.register_x, 1);
    }
    #[test]
    fn test_bpl_with_carry() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x10, 0x02, 0x00, 0x00, 0xE8, 0x00]);
        cpu.reset();
        cpu.status = 0b1000_0000;
        cpu.run();

        assert_eq!(cpu.register_x, 0);
    }
    #[test]
    fn test_bpl_negative_value() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x10, 0xFC]);
        cpu.reset();
        cpu.mem_write(0x7FFF, 0x00);
        cpu.mem_write(0x7FFE, 0xE8);
        cpu.status = 0b0000_0000;
        cpu.run();

        assert_eq!(cpu.register_x, 1);
    }
    #[test]
    fn test_bvc() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x50, 0x02, 0x00, 0x00, 0xE8, 0x00]);
        cpu.reset();
        cpu.status = 0b0000_0000;
        cpu.run();

        assert_eq!(cpu.register_x, 1);
    }
    #[test]
    fn test_bvc_with_carry() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x50, 0x02, 0x00, 0x00, 0xE8, 0x00]);
        cpu.reset();
        cpu.status = 0b0100_0000;
        cpu.run();

        assert_eq!(cpu.register_x, 0);
    }
    #[test]
    fn test_bvc_negative_value() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x50, 0xFC]);
        cpu.reset();
        cpu.mem_write(0x7FFF, 0x00);
        cpu.mem_write(0x7FFE, 0xE8);
        cpu.status = 0b0000_0000;
        cpu.run();

        assert_eq!(cpu.register_x, 1);
    }
    #[test]
    fn test_bvs() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x70, 0x02, 0x00, 0x00, 0xE8, 0x00]);
        cpu.reset();
        cpu.status = 0b0000_0000;
        cpu.run();

        assert_eq!(cpu.register_x, 0);
    }
    #[test]
    fn test_bvs_with_carry() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x70, 0x02, 0x00, 0x00, 0xE8, 0x00]);
        cpu.reset();
        cpu.status = 0b0100_0000;
        cpu.run();

        assert_eq!(cpu.register_x, 1);
    }
    #[test]
    fn test_bvs_negative_value() {
         let mut cpu = CPU::new(Bus::new(Rom::empty()));
        cpu.load(vec![0x70, 0xFC]);
        cpu.reset();
        cpu.mem_write(0x7FFF, 0x00);
        cpu.mem_write(0x7FFE, 0xE8);
        cpu.status = 0b0100_0000;
        cpu.run();

        assert_eq!(cpu.register_x, 1);
    }
}
