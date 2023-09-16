use crate::cpu::AddressingMode;
use std::collections::HashMap;

pub struct OpCode {
    pub code: u8,
    pub mnemonic: &'static str,
    pub len: u8,
    pub cycles: u8,
    pub mode: AddressingMode,
}

impl OpCode {
    fn new(code: u8, mnemonic: &'static str, len: u8, cycles: u8, mode: AddressingMode) -> Self {
        OpCode {
            code: code,
            mnemonic: mnemonic,
            len: len,
            cycles: cycles,
            mode: mode,
        }
    }
}

lazy_static! {
    pub static ref CPU_OPS_CODES: Vec<OpCode> = vec![
        /* ADC */
        OpCode::new(0x69, "ADC", 2, 2, AddressingMode::Immediate),
        OpCode::new(0x65, "ADC", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x75, "ADC", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x6D, "ADC", 3, 4, AddressingMode::Absolute),
        OpCode::new(0x7D, "ADC", 3, 4, AddressingMode::Absolute_X),
        OpCode::new(0x79, "ADC", 3, 4, AddressingMode::Absolute_Y),
        OpCode::new(0x61, "ADC", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x71, "ADC", 2, 5, AddressingMode::Indirect_Y),
        /* AND */
        OpCode::new(0x29, "AND", 2, 2, AddressingMode::Immediate),
        OpCode::new(0x25, "AND", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x35, "AND", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x2D, "AND", 3, 4, AddressingMode::Absolute),
        OpCode::new(0x3D, "AND", 3, 4, AddressingMode::Absolute_X),
        OpCode::new(0x39, "AND", 3, 4, AddressingMode::Absolute_Y),
        OpCode::new(0x21, "AND", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x31, "AND", 2, 5, AddressingMode::Indirect_Y),
        /* ASL */
        OpCode::new(0x0A, "ASL", 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0x06, "ASL", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x16, "ASL", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x0E, "ASL", 3, 6, AddressingMode::Absolute),
        OpCode::new(0x1E, "ASL", 3, 7, AddressingMode::Absolute_X),
        /* BIT */
        OpCode::new(0x24, "BIT", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x2C, "BIT", 3, 4, AddressingMode::Absolute),
        /* CMP */
        OpCode::new(0xC9, "CMP", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xC5, "CMP", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xD5, "CMP", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0xCD, "CMP", 3, 4, AddressingMode::Absolute),
        OpCode::new(0xDD, "CMP", 3, 4, AddressingMode::Absolute_X),
        OpCode::new(0xD9, "CMP", 3, 4, AddressingMode::Absolute_Y),
        OpCode::new(0xC1, "CMP", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0xD1, "CMP", 2, 5, AddressingMode::Indirect_Y),
        /* CMX */
        OpCode::new(0xE0, "CMX", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xE4, "CMX", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xEC, "CMX", 3, 4, AddressingMode::Absolute),
        /* CMY */
        OpCode::new(0xC0, "CMY", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xC4, "CMY", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xCC, "CMY", 3, 4, AddressingMode::Absolute),
         /* DEC */
        OpCode::new(0xC6, "DEC", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0xD6, "DEC", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0xCE, "DEC", 3, 6, AddressingMode::Absolute),
        OpCode::new(0xDE, "DEC", 3, 7, AddressingMode::Absolute_X),
        /* DEX */
        OpCode::new(0xCA, "DEX", 1, 2, AddressingMode::NoneAddressing),
        /* DEY */
        OpCode::new(0x88, "DEY", 1, 2, AddressingMode::NoneAddressing),
        /* EOR */
        OpCode::new(0x49, "EOR", 2, 2, AddressingMode::Immediate),
        OpCode::new(0x45, "EOR", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x55, "EOR", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x4D, "EOR", 3, 4, AddressingMode::Absolute),
        OpCode::new(0x5D, "EOR", 3, 4, AddressingMode::Absolute_X),
        OpCode::new(0x59, "EOR", 3, 4, AddressingMode::Absolute_Y),
        OpCode::new(0x41, "EOR", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x51, "EOR", 2, 5, AddressingMode::Indirect_Y),
        /* INC */
        OpCode::new(0xE6, "INC", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0xF6, "INC", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0xEE, "INC", 3, 6, AddressingMode::Absolute),
        OpCode::new(0xFE, "INC", 3, 7, AddressingMode::Absolute_X),
        /* INX */
        OpCode::new(0xE8, "INX", 1, 2, AddressingMode::NoneAddressing),
        /* INY */
        OpCode::new(0xC8, "INY", 1, 2, AddressingMode::NoneAddressing),
        /* LSR */
        OpCode::new(0x4A, "LSR", 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0x46, "LSR", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x56, "LSR", 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x4E, "LSR", 3, 6, AddressingMode::Absolute),
        OpCode::new(0x5E, "LSR", 3, 7, AddressingMode::Absolute_X),
        /* ORA */
        OpCode::new(0x09, "ORA", 2, 2, AddressingMode::Immediate),
        OpCode::new(0x05, "ORA", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x15, "ORA", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x0D, "ORA", 3, 4, AddressingMode::Absolute),
        OpCode::new(0x1D, "ORA", 3, 4, AddressingMode::Absolute_X),
        OpCode::new(0x19, "ORA", 3, 4, AddressingMode::Absolute_Y),
        OpCode::new(0x01, "ORA", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x11, "ORA", 2, 5, AddressingMode::Indirect_Y),
        /* BRK */
        OpCode::new(0x00, "BRK", 1, 7, AddressingMode::NoneAddressing),
        /* LDA */
        OpCode::new(0xA9, "LDA", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xA5, "LDA", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xB5, "LDA", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0xAD, "LDA", 3, 4, AddressingMode::Absolute),
        OpCode::new(0xBD, "LDA", 3, 4, AddressingMode::Absolute_X),
        OpCode::new(0xB9, "LDA", 3, 4, AddressingMode::Absolute_Y),
        OpCode::new(0xA1, "LDA", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0xB1, "LDA", 2, 5, AddressingMode::Indirect_Y),
        /* STA */
        OpCode::new(0x85, "STA", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x95, "STA", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x8D, "STA", 3, 4, AddressingMode::Absolute),
        OpCode::new(0x9D, "STA", 3, 5, AddressingMode::Absolute_X),
        OpCode::new(0x99, "STA", 3, 5, AddressingMode::Absolute_Y),
        OpCode::new(0x81, "STA", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x91, "STA", 2, 6, AddressingMode::Indirect_Y),
        /* TAX */
        OpCode::new(0xAA, "TAX", 1, 2, AddressingMode::NoneAddressing),
    ];
    pub static ref OPCODES_MAP: HashMap<u8, &'static OpCode> = {
        let mut map = HashMap::new();
        for cpuop in &*CPU_OPS_CODES {
            map.insert(cpuop.code, cpuop);
        }
        map
    };
}
