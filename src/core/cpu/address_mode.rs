use std::fmt::Display;

#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) enum AddressMode {
    Implied,
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX,
    IndirectY,
}

const OPCODE_ADDRESS_MODES: [AddressMode; 256] = [
    // 0x00-0x0F
    AddressMode::Implied,     // 0x00 BRK
    AddressMode::IndirectX,   // 0x01 ORA (indirect,X)
    AddressMode::Implied,     // 0x02 *KIL
    AddressMode::IndirectX,   // 0x03 *SLO (indirect,X)
    AddressMode::ZeroPage,    // 0x04 *NOP zeropage
    AddressMode::ZeroPage,    // 0x05 ORA zeropage
    AddressMode::ZeroPage,    // 0x06 ASL zeropage
    AddressMode::ZeroPage,    // 0x07 *SLO zeropage
    AddressMode::Implied,     // 0x08 PHP
    AddressMode::Immediate,   // 0x09 ORA immediate
    AddressMode::Accumulator, // 0x0A ASL accumulator
    AddressMode::Immediate,   // 0x0B *ANC immediate
    AddressMode::Absolute,    // 0x0C *NOP absolute
    AddressMode::Absolute,    // 0x0D ORA absolute
    AddressMode::Absolute,    // 0x0E ASL absolute
    AddressMode::Absolute,    // 0x0F *SLO absolute
    // 0x10-0x1F
    AddressMode::Immediate, // 0x10 BPL relative
    AddressMode::IndirectY, // 0x11 ORA (indirect),Y
    AddressMode::Implied,   // 0x12 *KIL
    AddressMode::IndirectY, // 0x13 *SLO (indirect),Y
    AddressMode::ZeroPageX, // 0x14 *NOP zeropage,X
    AddressMode::ZeroPageX, // 0x15 ORA zeropage,X
    AddressMode::ZeroPageX, // 0x16 ASL zeropage,X
    AddressMode::ZeroPageX, // 0x17 *SLO zeropage,X
    AddressMode::Implied,   // 0x18 CLC
    AddressMode::AbsoluteY, // 0x19 ORA absolute,Y
    AddressMode::Implied,   // 0x1A *NOP
    AddressMode::AbsoluteY, // 0x1B *SLO absolute,Y
    AddressMode::AbsoluteX, // 0x1C *NOP absolute,X
    AddressMode::AbsoluteX, // 0x1D ORA absolute,X
    AddressMode::AbsoluteX, // 0x1E ASL absolute,X
    AddressMode::AbsoluteX, // 0x1F *SLO absolute,X
    // 0x20-0x2F
    AddressMode::Absolute,    // 0x20 JSR absolute
    AddressMode::IndirectX,   // 0x21 AND (indirect,X)
    AddressMode::Implied,     // 0x22 *KIL
    AddressMode::IndirectX,   // 0x23 *RLA (indirect,X)
    AddressMode::ZeroPage,    // 0x24 BIT zeropage
    AddressMode::ZeroPage,    // 0x25 AND zeropage
    AddressMode::ZeroPage,    // 0x26 ROL zeropage
    AddressMode::ZeroPage,    // 0x27 *RLA zeropage
    AddressMode::Implied,     // 0x28 PLP
    AddressMode::Immediate,   // 0x29 AND immediate
    AddressMode::Accumulator, // 0x2A ROL accumulator
    AddressMode::Immediate,   // 0x2B *ANC immediate
    AddressMode::Absolute,    // 0x2C BIT absolute
    AddressMode::Absolute,    // 0x2D AND absolute
    AddressMode::Absolute,    // 0x2E ROL absolute
    AddressMode::Absolute,    // 0x2F *RLA absolute
    // 0x30-0x3F
    AddressMode::Immediate, // 0x30 BMI relative
    AddressMode::IndirectY, // 0x31 AND (indirect),Y
    AddressMode::Implied,   // 0x32 *KIL
    AddressMode::IndirectY, // 0x33 *RLA (indirect),Y
    AddressMode::ZeroPageX, // 0x34 *NOP zeropage,X
    AddressMode::ZeroPageX, // 0x35 AND zeropage,X
    AddressMode::ZeroPageX, // 0x36 ROL zeropage,X
    AddressMode::ZeroPageX, // 0x37 *RLA zeropage,X
    AddressMode::Implied,   // 0x38 SEC
    AddressMode::AbsoluteY, // 0x39 AND absolute,Y
    AddressMode::Implied,   // 0x3A *NOP
    AddressMode::AbsoluteY, // 0x3B *RLA absolute,Y
    AddressMode::AbsoluteX, // 0x3C *NOP absolute,X
    AddressMode::AbsoluteX, // 0x3D AND absolute,X
    AddressMode::AbsoluteX, // 0x3E ROL absolute,X
    AddressMode::AbsoluteX, // 0x3F *RLA absolute,X
    // 0x40-0x4F
    AddressMode::Implied,     // 0x40 RTI
    AddressMode::IndirectX,   // 0x41 EOR (indirect,X)
    AddressMode::Implied,     // 0x42 *KIL
    AddressMode::IndirectX,   // 0x43 *SRE (indirect,X)
    AddressMode::ZeroPage,    // 0x44 *NOP zeropage
    AddressMode::ZeroPage,    // 0x45 EOR zeropage
    AddressMode::ZeroPage,    // 0x46 LSR zeropage
    AddressMode::ZeroPage,    // 0x47 *SRE zeropage
    AddressMode::Implied,     // 0x48 PHA
    AddressMode::Immediate,   // 0x49 EOR immediate
    AddressMode::Accumulator, // 0x4A LSR accumulator
    AddressMode::Immediate,   // 0x4B *ALR immediate
    AddressMode::Absolute,    // 0x4C JMP absolute
    AddressMode::Absolute,    // 0x4D EOR absolute
    AddressMode::Absolute,    // 0x4E LSR absolute
    AddressMode::Absolute,    // 0x4F *SRE absolute
    // 0x50-0x5F
    AddressMode::Immediate, // 0x50 BVC relative
    AddressMode::IndirectY, // 0x51 EOR (indirect),Y
    AddressMode::Implied,   // 0x52 *KIL
    AddressMode::IndirectY, // 0x53 *SRE (indirect),Y
    AddressMode::ZeroPageX, // 0x54 *NOP zeropage,X
    AddressMode::ZeroPageX, // 0x55 EOR zeropage,X
    AddressMode::ZeroPageX, // 0x56 LSR zeropage,X
    AddressMode::ZeroPageX, // 0x57 *SRE zeropage,X
    AddressMode::Implied,   // 0x58 CLI
    AddressMode::AbsoluteY, // 0x59 EOR absolute,Y
    AddressMode::Implied,   // 0x5A *NOP
    AddressMode::AbsoluteY, // 0x5B *SRE absolute,Y
    AddressMode::AbsoluteX, // 0x5C *NOP absolute,X
    AddressMode::AbsoluteX, // 0x5D EOR absolute,X
    AddressMode::AbsoluteX, // 0x5E LSR absolute,X
    AddressMode::AbsoluteX, // 0x5F *SRE absolute,X
    // 0x60-0x6F
    AddressMode::Implied,     // 0x60 RTS
    AddressMode::IndirectX,   // 0x61 ADC (indirect,X)
    AddressMode::Implied,     // 0x62 *KIL
    AddressMode::IndirectX,   // 0x63 *RRA (indirect,X)
    AddressMode::ZeroPage,    // 0x64 *NOP zeropage
    AddressMode::ZeroPage,    // 0x65 ADC zeropage
    AddressMode::ZeroPage,    // 0x66 ROR zeropage
    AddressMode::ZeroPage,    // 0x67 *RRA zeropage
    AddressMode::Implied,     // 0x68 PLA
    AddressMode::Immediate,   // 0x69 ADC immediate
    AddressMode::Accumulator, // 0x6A ROR accumulator
    AddressMode::Immediate,   // 0x6B *ARR immediate
    AddressMode::Indirect,    // 0x6C JMP indirect
    AddressMode::Absolute,    // 0x6D ADC absolute
    AddressMode::Absolute,    // 0x6E ROR absolute
    AddressMode::Absolute,    // 0x6F *RRA absolute
    // 0x70-0x7F
    AddressMode::Immediate, // 0x70 BVS relative
    AddressMode::IndirectY, // 0x71 ADC (indirect),Y
    AddressMode::Implied,   // 0x72 *KIL
    AddressMode::IndirectY, // 0x73 *RRA (indirect),Y
    AddressMode::ZeroPageX, // 0x74 *NOP zeropage,X
    AddressMode::ZeroPageX, // 0x75 ADC zeropage,X
    AddressMode::ZeroPageX, // 0x76 ROR zeropage,X
    AddressMode::ZeroPageX, // 0x77 *RRA zeropage,X
    AddressMode::Implied,   // 0x78 SEI
    AddressMode::AbsoluteY, // 0x79 ADC absolute,Y
    AddressMode::Implied,   // 0x7A *NOP
    AddressMode::AbsoluteY, // 0x7B *RRA absolute,Y
    AddressMode::AbsoluteX, // 0x7C *NOP absolute,X
    AddressMode::AbsoluteX, // 0x7D ADC absolute,X
    AddressMode::AbsoluteX, // 0x7E ROR absolute,X
    AddressMode::AbsoluteX, // 0x7F *RRA absolute,X
    // 0x80-0x8F
    AddressMode::Immediate, // 0x80 *NOP immediate
    AddressMode::IndirectX, // 0x81 STA (indirect,X)
    AddressMode::Immediate, // 0x82 *NOP immediate
    AddressMode::IndirectX, // 0x83 *SAX (indirect,X)
    AddressMode::ZeroPage,  // 0x84 STY zeropage
    AddressMode::ZeroPage,  // 0x85 STA zeropage
    AddressMode::ZeroPage,  // 0x86 STX zeropage
    AddressMode::ZeroPage,  // 0x87 *SAX zeropage
    AddressMode::Implied,   // 0x88 DEY
    AddressMode::Immediate, // 0x89 *NOP immediate
    AddressMode::Implied,   // 0x8A TXA
    AddressMode::Immediate, // 0x8B *XAA immediate
    AddressMode::Absolute,  // 0x8C STY absolute
    AddressMode::Absolute,  // 0x8D STA absolute
    AddressMode::Absolute,  // 0x8E STX absolute
    AddressMode::Absolute,  // 0x8F *SAX absolute
    // 0x90-0x9F
    AddressMode::Immediate, // 0x90 BCC relative
    AddressMode::IndirectY, // 0x91 STA (indirect),Y
    AddressMode::Implied,   // 0x92 *KIL
    AddressMode::IndirectY, // 0x93 *AHX (indirect),Y
    AddressMode::ZeroPageX, // 0x94 STY zeropage,X
    AddressMode::ZeroPageX, // 0x95 STA zeropage,X
    AddressMode::ZeroPageY, // 0x96 STX zeropage,Y
    AddressMode::ZeroPageY, // 0x97 *SAX zeropage,Y
    AddressMode::Implied,   // 0x98 TYA
    AddressMode::AbsoluteY, // 0x99 STA absolute,Y
    AddressMode::Implied,   // 0x9A TXS
    AddressMode::AbsoluteY, // 0x9B *TAS absolute,Y
    AddressMode::AbsoluteX, // 0x9C *SHY absolute,X
    AddressMode::AbsoluteX, // 0x9D STA absolute,X
    AddressMode::AbsoluteY, // 0x9E *SHX absolute,Y
    AddressMode::AbsoluteY, // 0x9F *AHX absolute,Y
    // 0xA0-0xAF
    AddressMode::Immediate, // 0xA0 LDY immediate
    AddressMode::IndirectX, // 0xA1 LDA (indirect,X)
    AddressMode::Immediate, // 0xA2 LDX immediate
    AddressMode::IndirectX, // 0xA3 *LAX (indirect,X)
    AddressMode::ZeroPage,  // 0xA4 LDY zeropage
    AddressMode::ZeroPage,  // 0xA5 LDA zeropage
    AddressMode::ZeroPage,  // 0xA6 LDX zeropage
    AddressMode::ZeroPage,  // 0xA7 *LAX zeropage
    AddressMode::Implied,   // 0xA8 TAY
    AddressMode::Immediate, // 0xA9 LDA immediate
    AddressMode::Implied,   // 0xAA TAX
    AddressMode::Immediate, // 0xAB *LAX immediate
    AddressMode::Absolute,  // 0xAC LDY absolute
    AddressMode::Absolute,  // 0xAD LDA absolute
    AddressMode::Absolute,  // 0xAE LDX absolute
    AddressMode::Absolute,  // 0xAF *LAX absolute
    // 0xB0-0xBF
    AddressMode::Immediate, // 0xB0 BCS relative
    AddressMode::IndirectY, // 0xB1 LDA (indirect),Y
    AddressMode::Implied,   // 0xB2 *KIL
    AddressMode::IndirectY, // 0xB3 *LAX (indirect),Y
    AddressMode::ZeroPageX, // 0xB4 LDY zeropage,X
    AddressMode::ZeroPageX, // 0xB5 LDA zeropage,X
    AddressMode::ZeroPageY, // 0xB6 LDX zeropage,Y
    AddressMode::ZeroPageY, // 0xB7 *LAX zeropage,Y
    AddressMode::Implied,   // 0xB8 CLV
    AddressMode::AbsoluteY, // 0xB9 LDA absolute,Y
    AddressMode::Implied,   // 0xBA TSX
    AddressMode::AbsoluteY, // 0xBB *LAS absolute,Y
    AddressMode::AbsoluteX, // 0xBC LDY absolute,X
    AddressMode::AbsoluteX, // 0xBD LDA absolute,X
    AddressMode::AbsoluteY, // 0xBE LDX absolute,Y
    AddressMode::AbsoluteY, // 0xBF *LAX absolute,Y
    // 0xC0-0xCF
    AddressMode::Immediate, // 0xC0 CPY immediate
    AddressMode::IndirectX, // 0xC1 CMP (indirect,X)
    AddressMode::Immediate, // 0xC2 *NOP immediate
    AddressMode::IndirectX, // 0xC3 *DCP (indirect,X)
    AddressMode::ZeroPage,  // 0xC4 CPY zeropage
    AddressMode::ZeroPage,  // 0xC5 CMP zeropage
    AddressMode::ZeroPage,  // 0xC6 DEC zeropage
    AddressMode::ZeroPage,  // 0xC7 *DCP zeropage
    AddressMode::Implied,   // 0xC8 INY
    AddressMode::Immediate, // 0xC9 CMP immediate
    AddressMode::Implied,   // 0xCA DEX
    AddressMode::Immediate, // 0xCB *AXS immediate
    AddressMode::Absolute,  // 0xCC CPY absolute
    AddressMode::Absolute,  // 0xCD CMP absolute
    AddressMode::Absolute,  // 0xCE DEC absolute
    AddressMode::Absolute,  // 0xCF *DCP absolute
    // 0xD0-0xDF
    AddressMode::Immediate, // 0xD0 BNE relative
    AddressMode::IndirectY, // 0xD1 CMP (indirect),Y
    AddressMode::Implied,   // 0xD2 *KIL
    AddressMode::IndirectY, // 0xD3 *DCP (indirect),Y
    AddressMode::ZeroPageX, // 0xD4 *NOP zeropage,X
    AddressMode::ZeroPageX, // 0xD5 CMP zeropage,X
    AddressMode::ZeroPageX, // 0xD6 DEC zeropage,X
    AddressMode::ZeroPageX, // 0xD7 *DCP zeropage,X
    AddressMode::Implied,   // 0xD8 CLD
    AddressMode::AbsoluteY, // 0xD9 CMP absolute,Y
    AddressMode::Implied,   // 0xDA *NOP
    AddressMode::AbsoluteY, // 0xDB *DCP absolute,Y
    AddressMode::AbsoluteX, // 0xDC *NOP absolute,X
    AddressMode::AbsoluteX, // 0xDD CMP absolute,X
    AddressMode::AbsoluteX, // 0xDE DEC absolute,X
    AddressMode::AbsoluteX, // 0xDF *DCP absolute,X
    // 0xE0-0xEF
    AddressMode::Immediate, // 0xE0 CPX immediate
    AddressMode::IndirectX, // 0xE1 SBC (indirect,X)
    AddressMode::Immediate, // 0xE2 *NOP immediate
    AddressMode::IndirectX, // 0xE3 *ISC (indirect,X)
    AddressMode::ZeroPage,  // 0xE4 CPX zeropage
    AddressMode::ZeroPage,  // 0xE5 SBC zeropage
    AddressMode::ZeroPage,  // 0xE6 INC zeropage
    AddressMode::ZeroPage,  // 0xE7 *ISC zeropage
    AddressMode::Implied,   // 0xE8 INX
    AddressMode::Immediate, // 0xE9 SBC immediate
    AddressMode::Implied,   // 0xEA NOP
    AddressMode::Immediate, // 0xEB *SBC immediate
    AddressMode::Absolute,  // 0xEC CPX absolute
    AddressMode::Absolute,  // 0xED SBC absolute
    AddressMode::Absolute,  // 0xEE INC absolute
    AddressMode::Absolute,  // 0xEF *ISC absolute
    // 0xF0-0xFF
    AddressMode::Immediate, // 0xF0 BEQ relative
    AddressMode::IndirectY, // 0xF1 SBC (indirect),Y
    AddressMode::Implied,   // 0xF2 *KIL
    AddressMode::IndirectY, // 0xF3 *ISC (indirect),Y
    AddressMode::ZeroPageX, // 0xF4 *NOP zeropage,X
    AddressMode::ZeroPageX, // 0xF5 SBC zeropage,X
    AddressMode::ZeroPageX, // 0xF6 INC zeropage,X
    AddressMode::ZeroPageX, // 0xF7 *ISC zeropage,X
    AddressMode::Implied,   // 0xF8 SED
    AddressMode::AbsoluteY, // 0xF9 SBC absolute,Y
    AddressMode::Implied,   // 0xFA *NOP
    AddressMode::AbsoluteY, // 0xFB *ISC absolute,Y
    AddressMode::AbsoluteX, // 0xFC *NOP absolute,X
    AddressMode::AbsoluteX, // 0xFD SBC absolute,X
    AddressMode::AbsoluteX, // 0xFE INC absolute,X
    AddressMode::AbsoluteX, // 0xFF *ISC absolute,X
];

impl Display for AddressMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let display = match self {
            &AddressMode::Implied | AddressMode::Accumulator => "",
            AddressMode::Immediate => "#i",
            AddressMode::ZeroPage => "d",
            AddressMode::ZeroPageX => "d,x",
            AddressMode::ZeroPageY => "d,y",
            AddressMode::Absolute => "a",
            AddressMode::AbsoluteX => "a,x",
            AddressMode::AbsoluteY => "a,y",
            AddressMode::Indirect => "(a)",
            AddressMode::IndirectX => "(d,x)",
            AddressMode::IndirectY => "(d),y",
        };

        write!(f, "{}", display)?;

        Ok(())
    }
}

impl AddressMode {
    pub fn from_code(opcode: u8) -> AddressMode {
        OPCODE_ADDRESS_MODES[opcode as usize]
    }

    pub fn cycle_cost(&self, page_cross: bool) -> usize {
        match &self {
            AddressMode::Implied => 0,
            AddressMode::Accumulator | AddressMode::Immediate => 1,
            AddressMode::ZeroPage => 2,
            AddressMode::ZeroPageX | AddressMode::ZeroPageY | AddressMode::Absolute => 3,
            AddressMode::AbsoluteX | AddressMode::AbsoluteY => {
                if page_cross {
                    4
                } else {
                    3
                }
            }
            AddressMode::IndirectX => 5,
            AddressMode::IndirectY => {
                if page_cross {
                    5
                } else {
                    4
                }
            }
            AddressMode::Indirect => 5,
        }
    }

    pub fn byte_code_size(&self) -> u16 {
        match &self {
            AddressMode::Implied | AddressMode::Accumulator => 0,
            AddressMode::Immediate
            | AddressMode::ZeroPage
            | AddressMode::ZeroPageX
            | AddressMode::ZeroPageY
            | AddressMode::IndirectX
            | AddressMode::IndirectY => 1,
            AddressMode::Absolute
            | AddressMode::AbsoluteX
            | AddressMode::AbsoluteY
            | AddressMode::Indirect => 2,
        }
    }
}
