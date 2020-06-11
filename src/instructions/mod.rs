use byteorder::{ByteOrder, LittleEndian};
use super::memory::EmulatedMemory;

pub fn get_instruction_disassembly(memory_addr: &mut u16, memory: &EmulatedMemory) -> String {
    let address = *memory_addr;
    let opcode = memory.read(address);

    match opcode {
        0x00 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - NOP", address, opcode)
        },
        0x01 => {
            let values = [memory.read(address + 1), memory.read(address + 2)];
            *memory_addr += 3;
            format!("${:04X} - ${:02X} ${:02X} ${:02X} - LD BC, ${:04X}", address, opcode, values[0], 
                values[1], LittleEndian::read_u16(&values))
        },
        0x02 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD (BC), A", address, opcode)
        },
        0x03 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - INC BC", address, opcode)
        },
        0x04 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - INC B", address, opcode)
        },
        0x05 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - DEC B", address, opcode)
        },
        0x06 => {
            let value = memory.read(address + 1);
            *memory_addr += 2;
            format!("${:04X} - ${:02X} ${:<6X} - LB B, ${:02X}", address, opcode, value, value)
        },
        0x07 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - RLCA", address, opcode)
        },
        0x08 => {
            let values = [memory.read(address + 1), memory.read(address + 2)];
            *memory_addr += 3;
            format!("${:04X} - ${:02X} ${:02X} ${:02X} - LD (${:04X}), SP", address, opcode, values[0], 
                values[1], LittleEndian::read_u16(&values))
        },
        0x09 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - ADD HL, BC", address, opcode)
        },
        0x0A => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD A, (BC)", address, opcode)
        },
        0x0B => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - DEC BC", address, opcode)
        },
        0x0C => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - INC C", address, opcode)
        },
        0x0D => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - DEC C", address, opcode)
        },
        0x0E => {
            let value = memory.read(address + 1);
            *memory_addr += 2;
            format!("${:04X} - ${:02X} ${:<6X} - LC C, ${:02X}", address, opcode, value, value)
        },
        0x0F => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - RRCA", address, opcode)
        },

        0x10 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - STOP", address, opcode)
        },
        0x11 => {
            let values = [memory.read(address + 1), memory.read(address + 2)];
            *memory_addr += 3;
            format!("${:04X} - ${:02X} ${:02X} ${:02X} - LD DE, ${:04X}", address, opcode, values[0], 
                values[1], LittleEndian::read_u16(&values))
        },
        0x12 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD (DE), A", address, opcode)
        },
        0x13 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - INC DE", address, opcode)
        },
        0x14 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - INC D", address, opcode)
        },
        0x15 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - DEC D", address, opcode)
        },
        0x16 => {
            let value = memory.read(address + 1);
            *memory_addr += 2;
            format!("${:04X} - ${:02X} ${:<6X} - LB D, ${:02X}", address, opcode, value, value)
        },
        0x17 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - RLA", address, opcode)
        },
        0x18 => {
            let value = memory.read(address + 1);
            *memory_addr += 2;
            format!("${:04X} - ${:02X} ${:<6X} - JR ${:02X}", address, opcode, value, value as i8)
        },
        0x19 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - ADD HL, DE", address, opcode)
        },
        0x1A => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD A, (DE)", address, opcode)
        },
        0x1B => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - DEC DE", address, opcode)
        },
        0x1C => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - INC E", address, opcode)
        },
        0x1D => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - DEC E", address, opcode)
        },
        0x1E => {
            let value = memory.read(address + 1);
            *memory_addr += 2;
            format!("${:04X} - ${:02X} ${:<6X} - LC E, ${:02X}", address, opcode, value, value)
        },
        0x1F => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - RRA", address, opcode)
        },

        0x20 => {
            let value = memory.read(address + 1);
            *memory_addr += 2;
            format!("${:04X} - ${:02X} ${:<6X} - JR NZ, ${:02X}", address, opcode, value, value as i8)
        },
        0x21 => {
            let values = [memory.read(address + 1), memory.read(address + 2)];
            *memory_addr += 3;
            format!("${:04X} - ${:02X} ${:02X} ${:02X} - LD HL, ${:04X}", address, opcode, values[0], 
                values[1], LittleEndian::read_u16(&values))
        },
        0x22 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD (HL+), A", address, opcode)
        },
        0x23 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - INC HL", address, opcode)
        },
        0x24 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - INC H", address, opcode)
        },
        0x25 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - DEC H", address, opcode)
        },
        0x26 => {
            let value = memory.read(address + 1);
            *memory_addr += 2;
            format!("${:04X} - ${:02X} ${:<6X} - LB H, ${:02X}", address, opcode, value, value)
        },
        0x27 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - DAA", address, opcode)
        },
        0x28 => {
            let value = memory.read(address + 1);
            *memory_addr += 2;
            format!("${:04X} - ${:02X} ${:<6X} - JR Z, ${:02X}", address, opcode, value, value as i8)
        },
        0x29 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - ADD HL, HL", address, opcode)
        },
        0x2A => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD A, (HL+)", address, opcode)
        },
        0x2B => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - DEC HL", address, opcode)
        },
        0x2C => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - INC L", address, opcode)
        },
        0x2D => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - DEC L", address, opcode)
        },
        0x2E => {
            let value = memory.read(address + 1);
            *memory_addr += 2;
            format!("${:04X} - ${:02X} ${:<6X} - LC L, ${:02X}", address, opcode, value, value)
        },
        0x2F => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - CPL", address, opcode)
        },

        0x30 => {
            let value = memory.read(address + 1);
            *memory_addr += 2;
            format!("${:04X} - ${:02X} ${:<6X} - JR NC, ${:02X}", address, opcode, value, value as i8)
        },
        0x31 => {
            let values = [memory.read(address + 1), memory.read(address + 2)];
            *memory_addr += 3;
            format!("${:04X} - ${:02X} ${:02X} ${:02X} - LD SP, ${:04X}", address, opcode, values[0], 
                values[1], LittleEndian::read_u16(&values))
        },
        0x32 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD (HL-), A", address, opcode)
        },
        0x33 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - INC SP", address, opcode)
        },
        0x34 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - INC (HL)", address, opcode)
        },
        0x35 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - DEC (HL)", address, opcode)
        },
        0x36 => {
            let value = memory.read(address + 1);
            *memory_addr += 2;
            format!("${:04X} - ${:02X} ${:<6X} - LB (HL), ${:02X}", address, opcode, value, value)
        },
        0x37 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - SCF", address, opcode)
        },
        0x38 => {
            let value = memory.read(address + 1);
            *memory_addr += 2;
            format!("${:04X} - ${:02X} ${:<6X} - JR C, ${:02X}", address, opcode, value, value as i8)
        },
        0x39 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - ADD HL, SP", address, opcode)
        },
        0x3A => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD A, (HL-)", address, opcode)
        },
        0x3B => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - DEC SP", address, opcode)
        },
        0x3C => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - INC A", address, opcode)
        },
        0x3D => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - DEC A", address, opcode)
        },
        0x3E => {
            let value = memory.read(address + 1);
            *memory_addr += 2;
            format!("${:04X} - ${:02X} ${:<6X} - LC A, ${:02X}", address, opcode, value, value)
        },
        0x3F => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - CCF", address, opcode)
        },

        0x40 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD B, B", address, opcode)
        },
        0x41 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD B, C", address, opcode)
        },
        0x42 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD B, D", address, opcode)
        },
        0x43 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD B, E", address, opcode)
        },
        0x44 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD B, H", address, opcode)
        },
        0x45 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD B, L", address, opcode)
        },
        0x46 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD B, (HL)", address, opcode)
        },
        0x47 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD B, A", address, opcode)
        },
        0x48 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD C, B", address, opcode)
        }
        0x49 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD C, C", address, opcode)
        },
        0x4A => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD C, D", address, opcode)
        },
        0x4B => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD C, E", address, opcode)
        },
        0x4C => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD C, H", address, opcode)
        },
        0x4D => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD C, L", address, opcode)
        },
        0x4E => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD C, (HL)", address, opcode)
        },
        0x4F => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD C, A", address, opcode)
        },

        0x50 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD D, B", address, opcode)
        },
        0x51 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD D, C", address, opcode)
        },
        0x52 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD D, D", address, opcode)
        },
        0x53 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD D, E", address, opcode)
        },
        0x54 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD D, H", address, opcode)
        },
        0x55 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD D, L", address, opcode)
        },
        0x56 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD D, (HL)", address, opcode)
        },
        0x57 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD D, A", address, opcode)
        },
        0x58 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD E, B", address, opcode)
        }
        0x59 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD E, C", address, opcode)
        },
        0x5A => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD E, D", address, opcode)
        },
        0x5B => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD E, E", address, opcode)
        },
        0x5C => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD E, H", address, opcode)
        },
        0x5D => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD E, L", address, opcode)
        },
        0x5E => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD E, (HL)", address, opcode)
        },
        0x5F => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD E, A", address, opcode)
        },

        0x60 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD H, B", address, opcode)
        },
        0x61 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD H, C", address, opcode)
        },
        0x62 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD H, D", address, opcode)
        },
        0x63 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD H, E", address, opcode)
        },
        0x64 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD H, H", address, opcode)
        },
        0x65 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD H, L", address, opcode)
        },
        0x66 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD H, (HL)", address, opcode)
        },
        0x67 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD H, A", address, opcode)
        },
        0x68 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD L, B", address, opcode)
        }
        0x69 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD L, C", address, opcode)
        },
        0x6A => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD L, D", address, opcode)
        },
        0x6B => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD L, E", address, opcode)
        },
        0x6C => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD L, H", address, opcode)
        },
        0x6D => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD L, L", address, opcode)
        },
        0x6E => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD L, (HL)", address, opcode)
        },
        0x6F => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD L, A", address, opcode)
        },

        0x70 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD (HL), B", address, opcode)
        },
        0x71 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD (HL), C", address, opcode)
        },
        0x72 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD (HL), D", address, opcode)
        },
        0x73 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD (HL), E", address, opcode)
        },
        0x74 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD (HL), H", address, opcode)
        },
        0x75 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD (HL), L", address, opcode)
        },
        0x76 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - HALT", address, opcode)
        },
        0x77 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD A, A", address, opcode)
        },
        0x78 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD A, B", address, opcode)
        }
        0x79 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD A, C", address, opcode)
        },
        0x7A => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD A, D", address, opcode)
        },
        0x7B => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD A, E", address, opcode)
        },
        0x7C => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD A, H", address, opcode)
        },
        0x7D => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD A, L", address, opcode)
        },
        0x7E => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD A, (HL)", address, opcode)
        },
        0x7F => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD A, A", address, opcode)
        },

        0x80 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - ADD A, B", address, opcode)
        },
        0x81 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - ADD A, C", address, opcode)
        },
        0x82 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - ADD A, D", address, opcode)
        },
        0x83 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - ADD A, E", address, opcode)
        },
        0x84 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - ADD A, H", address, opcode)
        },
        0x85 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - ADD A, L", address, opcode)
        },
        0x86 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - ADD A (HL)", address, opcode)
        },
        0x87 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - ADD A, A", address, opcode)
        },
        0x88 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - ADC A, B", address, opcode)
        }
        0x89 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - ADC A, C", address, opcode)
        },
        0x8A => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - ADC A, D", address, opcode)
        },
        0x8B => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - ADC A, E", address, opcode)
        },
        0x8C => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - ADC A, H", address, opcode)
        },
        0x8D => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - ADC A, L", address, opcode)
        },
        0x8E => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - ADC A, (HL)", address, opcode)
        },
        0x8F => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - ADC A, A", address, opcode)
        },

        0x90 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - SUB A, B", address, opcode)
        },
        0x91 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - SUB A, C", address, opcode)
        },
        0x92 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - SUB A, D", address, opcode)
        },
        0x93 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - SUB A, E", address, opcode)
        },
        0x94 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - SUB A, H", address, opcode)
        },
        0x95 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - SUB A, L", address, opcode)
        },
        0x96 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - SUB A (HL)", address, opcode)
        },
        0x97 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - SUB A, A", address, opcode)
        },
        0x98 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - SBC A, B", address, opcode)
        }
        0x99 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - SBC A, C", address, opcode)
        },
        0x9A => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - SBC A, D", address, opcode)
        },
        0x9B => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - SBC A, E", address, opcode)
        },
        0x9C => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - SBC A, H", address, opcode)
        },
        0x9D => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - SBC A, L", address, opcode)
        },
        0x9E => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - SBC A, (HL)", address, opcode)
        },
        0x9F => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - SBC A, A", address, opcode)
        },

        0xA0 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - AND A, B", address, opcode)
        },
        0xA1 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - AND A, C", address, opcode)
        },
        0xA2 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - AND A, D", address, opcode)
        },
        0xA3 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - AND A, E", address, opcode)
        },
        0xA4 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - AND A, H", address, opcode)
        },
        0xA5 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - AND A, L", address, opcode)
        },
        0xA6 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - AND A (HL)", address, opcode)
        },
        0xA7 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - AND A, A", address, opcode)
        },
        0xA8 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - XOR A, B", address, opcode)
        }
        0xA9 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - XOR A, C", address, opcode)
        },
        0xAA => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - XOR A, D", address, opcode)
        },
        0xAB => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - XOR A, E", address, opcode)
        },
        0xAC => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - XOR A, H", address, opcode)
        },
        0xAD => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - XOR A, L", address, opcode)
        },
        0xAE => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - XOR A, (HL)", address, opcode)
        },
        0xAF => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - XOR A, A", address, opcode)
        },

        0xB0 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - OR A, B", address, opcode)
        },
        0xB1 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - OR A, C", address, opcode)
        },
        0xB2 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - OR A, D", address, opcode)
        },
        0xB3 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - OR A, E", address, opcode)
        },
        0xB4 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - OR A, H", address, opcode)
        },
        0xB5 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - OR A, L", address, opcode)
        },
        0xB6 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - OR A (HL)", address, opcode)
        },
        0xB7 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - OR A, A", address, opcode)
        },
        0xB8 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - CP A, B", address, opcode)
        }
        0xB9 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - CP A, C", address, opcode)
        },
        0xBA => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - CP A, D", address, opcode)
        },
        0xBB => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - CP A, E", address, opcode)
        },
        0xBC => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - CP A, H", address, opcode)
        },
        0xBD => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - CP A, L", address, opcode)
        },
        0xBE => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - CP A, (HL)", address, opcode)
        },
        0xBF => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - CP A, A", address, opcode)
        },

        0xC0 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - RET NZ", address, opcode)
        },
        0xC1 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - POP BC", address, opcode)
        },
        0xC2 => {
            let values = [memory.read(address + 1), memory.read(address + 2)];
            *memory_addr += 3;
            format!("${:04X} - ${:02X} ${:02X} ${:02X} - JP NZ, ${:04X}", address, opcode, values[0], 
                values[1], LittleEndian::read_u16(&values))
        },
        0xC3 => {
            let values = [memory.read(address + 1), memory.read(address + 2)];
            *memory_addr += 3;
            format!("${:04X} - ${:02X} ${:02X} ${:02X} - JP, ${:04X}", address, opcode, values[0], 
                values[1], LittleEndian::read_u16(&values))
        },
        0xC4 => {
            let values = [memory.read(address + 1), memory.read(address + 2)];
            *memory_addr += 3;
            format!("${:04X} - ${:02X} ${:02X} ${:02X} - CALL NZ, ${:04X}", address, opcode, values[0], 
                values[1], LittleEndian::read_u16(&values))
        },
        0xC5 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - PUSH BC", address, opcode)
        },
        0xC6 => {
            let value = memory.read(address + 1);
            *memory_addr += 2;
            format!("${:04X} - ${:02X} ${:<6X} - ADD A, ${:02X}", address, opcode, value, value)
        },
        0xC7 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - RST $00", address, opcode)
        },
        0xC8 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - RET Z", address, opcode)
        }
        0xC9 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - RET", address, opcode)
        },
        0xCA => {
            let values = [memory.read(address + 1), memory.read(address + 2)];
            *memory_addr += 3;
            format!("${:04X} - ${:02X} ${:02X} ${:02X} - JP Z, ${:04X}", address, opcode, values[0], 
                values[1], LittleEndian::read_u16(&values))
        },
        0xCB => {
            get_prefixed_instruction_disassembly(memory_addr, memory)
        },
        0xCC => {
            let values = [memory.read(address + 1), memory.read(address + 2)];
            *memory_addr += 3;
            format!("${:04X} - ${:02X} ${:02X} ${:02X} - CALL Z, ${:04X}", address, opcode, values[0], 
                values[1], LittleEndian::read_u16(&values))
        },
        0xCD => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - CP A, L", address, opcode)
        },
        0xCE => {
            let value = memory.read(address + 1);
            *memory_addr += 2;
            format!("${:04X} - ${:02X} ${:<6X} - ADC A, ${:02X}", address, opcode, value, value)
        },
        0xCF => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - RST $08", address, opcode)
        },

        0xD0 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - RET NC", address, opcode)
        },
        0xD1 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - POP DE", address, opcode)
        },
        0xD2 => {
            let values = [memory.read(address + 1), memory.read(address + 2)];
            *memory_addr += 3;
            format!("${:04X} - ${:02X} ${:02X} ${:02X} - JP NC, ${:04X}", address, opcode, values[0], 
                values[1], LittleEndian::read_u16(&values))
        },
        0xD3 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - illegal opcode", address, opcode)
        },
        0xD4 => {
            let values = [memory.read(address + 1), memory.read(address + 2)];
            *memory_addr += 3;
            format!("${:04X} - ${:02X} ${:02X} ${:02X} - CALL NC, ${:04X}", address, opcode, values[0], 
                values[1], LittleEndian::read_u16(&values))
        },
        0xD5 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - PUSH DE", address, opcode)
        },
        0xD6 => {
            let value = memory.read(address + 1);
            *memory_addr += 2;
            format!("${:04X} - ${:02X} ${:<6X} - SUB A, ${:02X}", address, opcode, value, value)
        },
        0xD7 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - RST $10", address, opcode)
        },
        0xD8 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - RET C", address, opcode)
        }
        0xD9 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - RETI", address, opcode)
        },
        0xDA => {
            let values = [memory.read(address + 1), memory.read(address + 2)];
            *memory_addr += 3;
            format!("${:04X} - ${:02X} ${:02X} ${:02X} - JP C, ${:04X}", address, opcode, values[0], 
                values[1], LittleEndian::read_u16(&values))
        },
        0xDB => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - illegal opcode", address, opcode)
        },
        0xDC => {
            let values = [memory.read(address + 1), memory.read(address + 2)];
            *memory_addr += 3;
            format!("${:04X} - ${:02X} ${:02X} ${:02X} - CALL C, ${:04X}", address, opcode, values[0], 
                values[1], LittleEndian::read_u16(&values))
        },
        0xDD => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - illegal opcode", address, opcode)
        },
        0xDE => {
            let value = memory.read(address + 1);
            *memory_addr += 2;
            format!("${:04X} - ${:02X} ${:<6X} - SBC A, ${:02X}", address, opcode, value, value)
        },
        0xDF => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - RST $18", address, opcode)
        },


        0xE0 => {
            let value = memory.read(address + 1);
            *memory_addr += 2;
            format!("${:04X} - ${:02X} ${:<6X} - LD ($FF00+${:02X}), A", address, opcode, value, value)
        },
        0xE1 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - POP HL", address, opcode)
        },
        0xE2 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD ($FF00+C), A", address, opcode)
        },
        0xE3 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - illegal opcode", address, opcode)
        },
        0xE4 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - illegal opcode", address, opcode)
        },
        0xE5 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - PUSH HL", address, opcode)
        },
        0xE6 => {
            let value = memory.read(address + 1);
            *memory_addr += 2;
            format!("${:04X} - ${:02X} ${:<6X} - AND A, ${:02X}", address, opcode, value, value)
        },
        0xE7 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - RST $20", address, opcode)
        },
        0xE8 => {
            let value = memory.read(address + 1);
            *memory_addr += 2;
            format!("${:04X} - ${:02X} ${:<6X} - ADD SP, ${:02X}", address, opcode, value, value as i8)
        }
        0xE9 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - JP HL", address, opcode)
        },
        0xEA => {
            let values = [memory.read(address + 1), memory.read(address + 2)];
            *memory_addr += 3;
            format!("${:04X} - ${:02X} ${:02X} ${:02X} - LD ${:04X}, A", address, opcode, values[0], 
                values[1], LittleEndian::read_u16(&values))
        },
        0xEB => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - illegal opcode", address, opcode)
        },
        0xEC => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - illegal opcode", address, opcode)
        },
        0xED => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - illegal opcode", address, opcode)
        },
        0xEE => {
            let value = memory.read(address + 1);
            *memory_addr += 2;
            format!("${:04X} - ${:02X} ${:<6X} - XOR A, ${:02X}", address, opcode, value, value)
        },
        0xEF => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - RST $28", address, opcode)
        },


        0xF0 => {
            let value = memory.read(address + 1);
            *memory_addr += 2;
            format!("${:04X} - ${:02X} ${:<6X} - LD A, ($FF00+${:02X})", address, opcode, value, value)
        },
        0xF1 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - POP AF", address, opcode)
        },
        0xF2 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD A, ($FF00+C)", address, opcode)
        },
        0xF3 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - DI", address, opcode)
        },
        0xF4 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - illegal opcode", address, opcode)
        },
        0xF5 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - PUSH AF", address, opcode)
        },
        0xF6 => {
            let value = memory.read(address + 1);
            *memory_addr += 2;
            format!("${:04X} - ${:02X} ${:<6X} - OR A, ${:02X}", address, opcode, value, value)
        },
        0xF7 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - RST $30", address, opcode)
        },
        0xF8 => {
            let value = memory.read(address + 1);
            *memory_addr += 2;
            format!("${:04X} - ${:02X} ${:<6X} - LD HL, SP + ${:02X}", address, opcode, value, value as i8)
        }
        0xF9 => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - LD SP, HL", address, opcode)
        },
        0xFA => {
            let values = [memory.read(address + 1), memory.read(address + 2)];
            *memory_addr += 3;
            format!("${:04X} - ${:02X} ${:02X} ${:02X} - LD A, ${:04X}", address, opcode, values[0], 
                values[1], LittleEndian::read_u16(&values))
        },
        0xFB => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - EI", address, opcode)
        },
        0xFC => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - illegal opcode", address, opcode)
        },
        0xFD => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - illegal opcode", address, opcode)
        },
        0xFE => {
            let value = memory.read(address + 1);
            *memory_addr += 2;
            format!("${:04X} - ${:02X} ${:<6X} - CP A, ${:02X}", address, opcode, value, value)
        },
        0xFF => {
            *memory_addr += 1;
            format!("${:04X} - ${:<10X} - RST $38", address, opcode)
        }
    }
}

pub fn get_prefixed_instruction_disassembly(memory_addr: &mut u16, memory: &EmulatedMemory) -> String {
    let address = *memory_addr;
    let opcode = memory.read(address + 1);
    
    *memory_addr += 1;

    match opcode {
        0x00 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RLC B", address, opcode)
        },
        0x01 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RLC C", address, opcode)
        },
        0x02 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RLC D", address, opcode)
        },
        0x03 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RLC E", address, opcode)
        },
        0x04 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RLC H", address, opcode)
        },
        0x05 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RLC L", address, opcode)
        },
        0x06 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RLC (HL)", address, opcode)
        },
        0x07 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RLC A", address, opcode)
        },
        0x08 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RRC B", address, opcode)
        },
        0x09 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RRC C", address, opcode)
        },
        0x0A => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RRC D", address, opcode)
        },
        0x0B => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RRC E", address, opcode)
        },
        0x0C => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RRC H", address, opcode)
        },
        0x0D => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RRC L", address, opcode)
        },
        0x0E => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RRC (HL)", address, opcode)
        },
        0x0F => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RRC A", address, opcode)
        },


        0x10 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RL B", address, opcode)
        },

        0x11 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RL C", address, opcode)
        },
        0x12 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RL D", address, opcode)
        },
        0x13 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RL E", address, opcode)
        },
        0x14 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RL H", address, opcode)
        },
        0x15 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RL L", address, opcode)
        },
        0x16 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RL (HL)", address, opcode)
        },
        0x17 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RL A", address, opcode)
        },
        0x18 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RR B", address, opcode)
        },
        0x19 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RR C", address, opcode)
        },
        0x1A => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RR D", address, opcode)
        },
        0x1B => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RR E", address, opcode)
        },
        0x1C => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RR H", address, opcode)
        },
        0x1D => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RR L", address, opcode)
        },
        0x1E => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RR (HL)", address, opcode)
        },
        0x1F => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RR A", address, opcode)
        },


        0x20 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SLA B", address, opcode)
        },
        0x21 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SLA C", address, opcode)
        },
        0x22 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SLA D", address, opcode)
        },
        0x23 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SLA E", address, opcode)
        },
        0x24 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SLA H", address, opcode)
        },
        0x25 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SLA L", address, opcode)
        },
        0x26 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SLA (HL)", address, opcode)
        },
        0x27 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SLA A", address, opcode)
        },
        0x28 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SRA B", address, opcode)
        },
        0x29 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SRA C", address, opcode)
        },
        0x2A => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SRA D", address, opcode)
        },
        0x2B => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SRA E", address, opcode)
        },
        0x2C => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SRA H", address, opcode)
        },
        0x2D => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SRA L", address, opcode)
        },
        0x2E => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SRA (HL)", address, opcode)
        },
        0x2F => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SRA A", address, opcode)
        },


        0x30 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SWAP B", address, opcode)
        },
        0x31 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SWAP C", address, opcode)
        },
        0x32 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SWAP D", address, opcode)
        },
        0x33 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SWAP E", address, opcode)
        },
        0x34 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SWAP H", address, opcode)
        },
        0x35 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SWAP L", address, opcode)
        },
        0x36 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SWAP (HL)", address, opcode)
        },
        0x37 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SWAP A", address, opcode)
        },
        0x38 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SRL B", address, opcode)
        },
        0x39 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SRL C", address, opcode)
        },
        0x3A => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SRL D", address, opcode)
        },
        0x3B => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SRL E", address, opcode)
        },
        0x3C => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SRL H", address, opcode)
        },
        0x3D => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SRL L", address, opcode)
        },
        0x3E => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SRL (HL)", address, opcode)
        },
        0x3F => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SRL A", address, opcode)
        },


        0x40 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 0, B", address, opcode)
        },
        0x41 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 0, C", address, opcode)
        },
        0x42 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 0, D", address, opcode)
        },
        0x43 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 0, E", address, opcode)
        },
        0x44 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 0, H", address, opcode)
        },
        0x45 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 0, L", address, opcode)
        },
        0x46 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 0, (HL)", address, opcode)
        },
        0x47 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 0, A", address, opcode)
        },
        0x48 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 1, B", address, opcode)
        },
        0x49 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 1, C", address, opcode)
        },
        0x4A => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 1, D", address, opcode)
        },
        0x4B => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 1, E", address, opcode)
        },
        0x4C => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 1, H", address, opcode)
        },
        0x4D => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 1, L", address, opcode)
        },
        0x4E => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 1, (HL)", address, opcode)
        },
        0x4F => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 1, A", address, opcode)
        },


        0x50 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 2, B", address, opcode)
        },
        0x51 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 2, C", address, opcode)
        },
        0x52 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 2, D", address, opcode)
        },
        0x53 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 2, E", address, opcode)
        },
        0x54 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 2, H", address, opcode)
        },
        0x55 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 2, L", address, opcode)
        },
        0x56 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 2, (HL)", address, opcode)
        },
        0x57 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 2, A", address, opcode)
        },
        0x58 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 3, B", address, opcode)
        },
        0x59 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 3, C", address, opcode)
        },
        0x5A => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 3, D", address, opcode)
        },
        0x5B => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 3, E", address, opcode)
        },
        0x5C => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 3, H", address, opcode)
        },
        0x5D => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 3, L", address, opcode)
        },
        0x5E => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 3, (HL)", address, opcode)
        },
        0x5F => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 3, A", address, opcode)
        },


        0x60 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 4, B", address, opcode)
        },
        0x61 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 4, C", address, opcode)
        },
        0x62 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 4, D", address, opcode)
        },
        0x63 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 4, E", address, opcode)
        },
        0x64 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 4, H", address, opcode)
        },
        0x65 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 4, L", address, opcode)
        },
        0x66 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 4, (HL)", address, opcode)
        },
        0x67 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 4, A", address, opcode)
        },
        0x68 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 5, B", address, opcode)
        },
        0x69 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 5, C", address, opcode)
        },
        0x6A => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 5, D", address, opcode)
        },
        0x6B => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 5, E", address, opcode)
        },
        0x6C => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 5, H", address, opcode)
        },
        0x6D => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 5, L", address, opcode)
        },
        0x6E => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 5, (HL)", address, opcode)
        },
        0x6F => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 5, A", address, opcode)
        },


        0x70 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 6, B", address, opcode)
        },
        0x71 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 6, C", address, opcode)
        },
        0x72 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 6, D", address, opcode)
        },
        0x73 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 6, E", address, opcode)
        },
        0x74 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 6, H", address, opcode)
        },
        0x75 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 6, L", address, opcode)
        },
        0x76 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 6, (HL)", address, opcode)
        },
        0x77 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 6, A", address, opcode)
        },
        0x78 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 7, B", address, opcode)
        },
        0x79 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 7, C", address, opcode)
        },
        0x7A => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 7, D", address, opcode)
        },
        0x7B => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 7, E", address, opcode)
        },
        0x7C => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 7, H", address, opcode)
        },
        0x7D => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 7, L", address, opcode)
        },
        0x7E => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 7, (HL)", address, opcode)
        },
        0x7F => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - BIT 7, A", address, opcode)
        },


        0x80 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 0, B", address, opcode)
        },
        0x81 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 0, C", address, opcode)
        },
        0x82 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 0, D", address, opcode)
        },
        0x83 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 0, E", address, opcode)
        },
        0x84 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 0, H", address, opcode)
        },
        0x85 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 0, L", address, opcode)
        },
        0x86 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 0, (HL)", address, opcode)
        },
        0x87 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 0, A", address, opcode)
        },
        0x88 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 1, B", address, opcode)
        },
        0x89 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 1, C", address, opcode)
        },
        0x8A => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 1, D", address, opcode)
        },
        0x8B => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 1, E", address, opcode)
        },
        0x8C => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 1, H", address, opcode)
        },
        0x8D => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 1, L", address, opcode)
        },
        0x8E => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 1, (HL)", address, opcode)
        },
        0x8F => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 1, A", address, opcode)
        },


        0x90 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 2, B", address, opcode)
        },
        0x91 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 2, C", address, opcode)
        },
        0x92 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 2, D", address, opcode)
        },
        0x93 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 2, E", address, opcode)
        },
        0x94 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 2, H", address, opcode)
        },
        0x95 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 2, L", address, opcode)
        },
        0x96 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 2, (HL)", address, opcode)
        },
        0x97 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 2, A", address, opcode)
        },
        0x98 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 3, B", address, opcode)
        },
        0x99 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 3, C", address, opcode)
        },
        0x9A => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 3, D", address, opcode)
        },
        0x9B => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 3, E", address, opcode)
        },
        0x9C => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 3, H", address, opcode)
        },
        0x9D => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 3, L", address, opcode)
        },
        0x9E => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 3, (HL)", address, opcode)
        },
        0x9F => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 3, A", address, opcode)
        },


        0xA0 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 4, B", address, opcode)
        },
        0xA1 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 4, C", address, opcode)
        },
        0xA2 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 4, D", address, opcode)
        },
        0xA3 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 4, E", address, opcode)
        },
        0xA4 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 4, H", address, opcode)
        },
        0xA5 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 4, L", address, opcode)
        },
        0xA6 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 4, (HL)", address, opcode)
        },
        0xA7 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 4, A", address, opcode)
        },
        0xA8 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 5, B", address, opcode)
        },
        0xA9 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 5, C", address, opcode)
        },
        0xAA => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 5, D", address, opcode)
        },
        0xAB => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 5, E", address, opcode)
        },
        0xAC => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 5, H", address, opcode)
        },
        0xAD => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 5, L", address, opcode)
        },
        0xAE => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 5, (HL)", address, opcode)
        },
        0xAF => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 5, A", address, opcode)
        },


        0xB0 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 6, B", address, opcode)
        },
        0xB1 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 6, C", address, opcode)
        },
        0xB2 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 6, D", address, opcode)
        },
        0xB3 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 6, E", address, opcode)
        },
        0xB4 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 6, H", address, opcode)
        },
        0xB5 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 6, L", address, opcode)
        },
        0xB6 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 6, (HL)", address, opcode)
        },
        0xB7 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 6, A", address, opcode)
        },
        0xB8 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 7, B", address, opcode)
        },
        0xB9 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 7, C", address, opcode)
        },
        0xBA => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 7, D", address, opcode)
        },
        0xBB => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 7, E", address, opcode)
        },
        0xBC => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 7, H", address, opcode)
        },
        0xBD => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 7, L", address, opcode)
        },
        0xBE => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 7, (HL)", address, opcode)
        },
        0xBF => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - RES 7, A", address, opcode)
        },


        0xC0 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 0, B", address, opcode)
        },
        0xC1 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 0, C", address, opcode)
        },
        0xC2 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 0, D", address, opcode)
        },
        0xC3 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 0, E", address, opcode)
        },
        0xC4 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 0, H", address, opcode)
        },
        0xC5 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 0, L", address, opcode)
        },
        0xC6 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 0, (HL)", address, opcode)
        },
        0xC7 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 0, A", address, opcode)
        },
        0xC8 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 1, B", address, opcode)
        },
        0xC9 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 1, C", address, opcode)
        },
        0xCA => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 1, D", address, opcode)
        },
        0xCB => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 1, E", address, opcode)
        },
        0xCC => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 1, H", address, opcode)
        },
        0xCD => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 1, L", address, opcode)
        },
        0xCE => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 1, (HL)", address, opcode)
        },
        0xCF => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 1, A", address, opcode)
        },


        0xD0 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 2, B", address, opcode)
        },
        0xD1 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 2, C", address, opcode)
        },
        0xD2 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 2, D", address, opcode)
        },
        0xD3 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 2, E", address, opcode)
        },
        0xD4 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 2, H", address, opcode)
        },
        0xD5 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 2, L", address, opcode)
        },
        0xD6 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 2, (HL)", address, opcode)
        },
        0xD7 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 2, A", address, opcode)
        },
        0xD8 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 3, B", address, opcode)
        },
        0xD9 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 3, C", address, opcode)
        },
        0xDA => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 3, D", address, opcode)
        },
        0xDB => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 3, E", address, opcode)
        },
        0xDC => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 3, H", address, opcode)
        },
        0xDD => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 3, L", address, opcode)
        },
        0xDE => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 3, (HL)", address, opcode)
        },
        0xDF => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 3, A", address, opcode)
        },


        0xE0 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 4, B", address, opcode)
        },
        0xE1 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 4, C", address, opcode)
        },
        0xE2 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 4, D", address, opcode)
        },
        0xE3 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 4, E", address, opcode)
        },
        0xE4 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 4, H", address, opcode)
        },
        0xE5 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 4, L", address, opcode)
        },
        0xE6 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 4, (HL)", address, opcode)
        },
        0xE7 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 4, A", address, opcode)
        },
        0xE8 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 5, B", address, opcode)
        },
        0xE9 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 5, C", address, opcode)
        },
        0xEA => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 5, D", address, opcode)
        },
        0xEB => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 5, E", address, opcode)
        },
        0xEC => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 5, H", address, opcode)
        },
        0xED => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 5, L", address, opcode)
        },
        0xEE => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 5, (HL)", address, opcode)
        },
        0xEF => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 5, A", address, opcode)
        },


        0xF0 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 6, B", address, opcode)
        },
        0xF1 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 6, C", address, opcode)
        },
        0xF2 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 6, D", address, opcode)
        },
        0xF3 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 6, E", address, opcode)
        },
        0xF4 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 6, H", address, opcode)
        },
        0xF5 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 6, L", address, opcode)
        },
        0xF6 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 6, (HL)", address, opcode)
        },
        0xF7 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 6, A", address, opcode)
        },
        0xF8 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 7, B", address, opcode)
        },
        0xF9 => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 7, C", address, opcode)
        },
        0xFA => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 7, D", address, opcode)
        },
        0xFB => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 7, E", address, opcode)
        },
        0xFC => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 7, H", address, opcode)
        },
        0xFD => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 7, L", address, opcode)
        },
        0xFE => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 7, (HL)", address, opcode)
        },
        0xFF => {
            *memory_addr += 1;
            format!("${:04X} - $CB ${:<6X} - SET 7, A", address, opcode)
        }
    }
}