use crate::opcodes::OPCODES_MAP;

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

pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: u8,
    pub program_counter: u16,
    memory: [u8; 0xFFFF],
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: 0b0000_0000,
            program_counter: 0,
            memory: [0; 0xFFFF],
        }
    }

    fn adc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        let sum = self.register_a as u16 + value as u16 + self.get_flg(&FlgCodes::CARRY) as u16;
        self.set_flg(&FlgCodes::CARRY, if sum > 0xFF { 1 } else { 0 });

        let result = (sum & 0xFF) as u8;
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
        self.set_flg(&FlgCodes::CARRY, if value >> 7 == 0 { 0 } else { 1 });

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

    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);
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

    fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.status = 0;

        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, 0x8000);
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run();
    }

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

    pub fn run(&mut self) {
        loop {
            let code = self.mem_read(self.program_counter);
            self.program_counter += 1;
            let opcode = OPCODES_MAP
                .get(&code)
                .expect(&format!("OpCode {:x} is not recognized", code));

            match code {
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
                /* LSR_accumulator */
                0x4A => self.lsr_accumulator(),
                /* LSR others*/
                0x46 | 0x56 | 0x4E | 0x5E => self.lsr(&opcode.mode),
                /* INC */
                0xE6 | 0xF6 | 0xEE | 0xFE => self.inc(&opcode.mode),
                /* INX */
                0xE8 => self.inx(),
                /* INY */
                0xC8 => self.iny(),
                /* LDA */
                0xA9 | 0xA5 | 0xB5 | 0xAD | 0xBD | 0xB9 | 0xA1 | 0xB1 => {
                    self.lda(&opcode.mode);
                }
                0xAA => self.tax(),
                /* STA */
                0x85 | 0x95 | 0x8D | 0x9D | 0x99 | 0x81 | 0x91 => {
                    self.sta(&AddressingMode::ZeroPage);
                }
                0x00 => return,
                _ => {
                    todo!()
                }
            }
            self.program_counter += (opcode.len - 1) as u16;
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
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.register_a, 5);
        assert!(cpu.status & 0b0000_0010 == 0);
        assert!(cpu.status & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.status & 0b0000_0010 == 0b10);
    }

    #[test]
    fn test_0xa9_lda_negative_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xff, 0x00]);
        assert!(cpu.status & 0b1000_0000 == 0b1000_0000);
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x0A, 0xaa, 0x00]);

        assert_eq!(cpu.register_x, 10)
    }

    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 0xc1)
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.register_x = 0xff;
        cpu.load_and_run(vec![0xa9, 0xff, 0xaa, 0xe8, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 1)
    }

    #[test]
    fn test_lda_from_memory() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0x55);

        cpu.load_and_run(vec![0xa5, 0x10, 0x00]);

        assert_eq!(cpu.register_a, 0x55);
    }

    #[test]
    fn test_adc_no_carry_and_no_overflow() {
        let mut cpu = CPU::new();
        // load 0x01 to register_a with immediate
        cpu.load_and_run(vec![0xA9, 0x01, 0x69, 0x01]);

        assert_eq!(cpu.register_a, 0x02);
        assert_eq!(cpu.status, 0b0000_0000);
    }

    #[test]
    fn test_adc_has_carry() {
        let mut cpu = CPU::new();
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
        let mut cpu = CPU::new();
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
        let mut cpu = CPU::new();
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
        let mut cpu = CPU::new();
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
        let mut cpu = CPU::new();
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
        let mut cpu = CPU::new();
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
        let mut cpu = CPU::new();
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
        let mut cpu = CPU::new();
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
        let mut cpu = CPU::new();
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
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0x01);
        cpu.load(vec![0x16, 0x10]);
        cpu.reset();
        cpu.run();

        assert_eq!(cpu.mem_read(0x10), 0x02);
        assert_eq!(cpu.status, 0x00);
    }

    #[test]
    fn test_asl_register_x_occurs_carry() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0x80);
        cpu.load(vec![0x16, 0x10]);
        cpu.reset();
        cpu.run();

        assert_eq!(cpu.mem_read(0x10), 0x00);
        assert_eq!(cpu.status, 0x03);
    }

    #[test]
    fn test_bit_zero() {
        let mut cpu = CPU::new();
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
        let mut cpu = CPU::new();
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
        let mut cpu = CPU::new();
        cpu.load(vec![0xC9, 0x00]);
        cpu.reset();
        cpu.status = 0x00;
        cpu.register_a = 0x01;
        cpu.run();

        assert_eq!(cpu.status, 0x01);
    }

    #[test]
    fn test_cmp_registera_equal() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xC9, 0x01]);
        cpu.reset();
        cpu.status = 0x00;
        cpu.register_a = 0x01;
        cpu.run();

        assert_eq!(cpu.status, 0x03);
    }

    #[test]
    fn test_cmp_registera_smaller() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xC9, 0x01]);
        cpu.reset();
        cpu.status = 0x00;
        cpu.register_a = 0x00;
        cpu.run();

        assert_eq!(cpu.status, 0x80);
    }

    #[test]
    fn test_cmp_registerx_larger() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xE0, 0x00]);
        cpu.reset();
        cpu.status = 0x00;
        cpu.register_x = 0x01;
        cpu.run();

        assert_eq!(cpu.status, 0x01);
    }

    #[test]
    fn test_cmp_registerx_equal() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xE0, 0x01]);
        cpu.reset();
        cpu.status = 0x00;
        cpu.register_x = 0x01;
        cpu.run();

        assert_eq!(cpu.status, 0x03);
    }

    #[test]
    fn test_cmp_registerx_smaller() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xE0, 0x01]);
        cpu.reset();
        cpu.status = 0x00;
        cpu.register_x = 0x00;
        cpu.run();

        assert_eq!(cpu.status, 0x80);
    }

    #[test]
    fn test_cmp_registery_larger() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xC0, 0x00]);
        cpu.reset();
        cpu.status = 0x00;
        cpu.register_y = 0x01;
        cpu.run();

        assert_eq!(cpu.status, 0x01);
    }

    #[test]
    fn test_cmp_registery_equal() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xC0, 0x01]);
        cpu.reset();
        cpu.status = 0x00;
        cpu.register_y = 0x01;
        cpu.run();

        assert_eq!(cpu.status, 0x03);
    }

    #[test]
    fn test_cmp_registery_smaller() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xC0, 0x01]);
        cpu.reset();
        cpu.status = 0x00;
        cpu.register_y = 0x00;
        cpu.run();

        assert_eq!(cpu.status, 0x80);
    }
    #[test]
    fn test_dec() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x00, 0x01);
        cpu.load(vec![0xC6, 0x00]);
        cpu.reset();
        cpu.status = 0x00;
        cpu.run();

        assert_eq!(cpu.status, 0x02);
    }
    #[test]
    fn test_dex() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xCA]);
        cpu.reset();
        cpu.status = 0x00;
        cpu.register_x = 0x01;
        cpu.run();

        assert_eq!(cpu.status, 0x02);
    }
    #[test]
    fn test_dey() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x88]);
        cpu.reset();
        cpu.status = 0x00;
        cpu.register_y = 0x01;
        cpu.run();

        assert_eq!(cpu.status, 0x02);
    }
    #[test]
    fn test_eor() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x49, 0x80]);
        cpu.reset();
        cpu.status = 0x00;
        cpu.register_a = 0x01;
        cpu.run();

        assert_eq!(cpu.register_a, 0x81);
    }

    #[test]
    fn test_inc() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x00, 0x01);
        cpu.load(vec![0xE6, 0x00]);
        cpu.reset();
        cpu.status = 0x00;
        cpu.run();

        assert_eq!(cpu.mem_read(0x00), 0x02);
    }
    #[test]
    fn test_inx() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xE8]);
        cpu.reset();
        cpu.status = 0x00;
        cpu.register_x = 0x01;
        cpu.run();

        assert_eq!(cpu.register_x, 0x02);
    }
    #[test]
    fn test_iny() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xC8]);
        cpu.reset();
        cpu.status = 0x00;
        cpu.register_y = 0x01;
        cpu.run();

        assert_eq!(cpu.register_y, 0x02);
    }
    #[test]
    fn test_lsr_accumulator() {
        let mut cpu = CPU::new();
        cpu.load(vec![0x4A]);
        cpu.reset();
        cpu.register_a = 0x40;
        cpu.status = 0x00;
        cpu.run();

        assert_eq!(cpu.register_a, 0x20);
        assert_eq!(cpu.status, 0x00);
    }

    #[test]
    fn test_lsr_zeropage() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0x02);
        cpu.load(vec![0x46, 0x10]);
        cpu.reset();
        cpu.run();

        assert_eq!(cpu.mem_read(0x10), 0x01);
        assert_eq!(cpu.status, 0x00);
    }
}
