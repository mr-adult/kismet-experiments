/// Low-level binary reader for Kismet bytecode
use std::collections::BTreeMap;

use super::address_index::AddressIndex;
use super::types::{Address, Name};

pub type CodeSkipSizeType = u32;

/// Low-level binary reader for script bytecode
pub struct ScriptReader<'a> {
    script: &'a [u8],
    names: &'a BTreeMap<u32, String>,
    address_index: &'a AddressIndex<'a>,
}

impl<'a> ScriptReader<'a> {
    pub fn new(
        script: &'a [u8],
        names: &'a BTreeMap<u32, String>,
        address_index: &'a AddressIndex<'a>,
    ) -> Self {
        Self {
            script,
            names,
            address_index,
        }
    }

    pub fn script(&self) -> &[u8] {
        self.script
    }

    // Primitive reads

    pub fn read_byte(&self, offset: &mut usize) -> u8 {
        let value = self.script[*offset];
        *offset += 1;
        value
    }

    pub fn read_word(&self, offset: &mut usize) -> u16 {
        let bytes: [u8; 2] = self.script[*offset..*offset + 2].try_into().unwrap();
        *offset += 2;
        u16::from_le_bytes(bytes)
    }

    pub fn read_int(&self, offset: &mut usize) -> i32 {
        let bytes: [u8; 4] = self.script[*offset..*offset + 4].try_into().unwrap();
        *offset += 4;
        i32::from_le_bytes(bytes)
    }

    pub fn read_qword(&self, offset: &mut usize) -> u64 {
        let bytes: [u8; 8] = self.script[*offset..*offset + 8].try_into().unwrap();
        *offset += 8;
        u64::from_le_bytes(bytes)
    }

    pub fn read_float(&self, offset: &mut usize) -> f32 {
        let int_value = self.read_int(offset);
        f32::from_bits(int_value as u32)
    }

    pub fn read_skip_count(&self, offset: &mut usize) -> CodeSkipSizeType {
        self.read_int(offset) as CodeSkipSizeType
    }

    // String reads

    pub fn read_string8(&self, offset: &mut usize) -> String {
        let mut result = String::new();
        loop {
            let byte = self.read_byte(offset);
            if byte == 0 {
                break;
            }
            result.push(byte as char);
        }
        result
    }

    pub fn read_string16(&self, offset: &mut usize) -> String {
        let mut result = String::new();
        loop {
            let word = self.read_word(offset);
            if word == 0 {
                break;
            }
            if let Some(ch) = char::from_u32(word as u32) {
                result.push(ch);
            }
        }
        result
    }

    // Domain-specific reads

    pub fn read_name(&self, offset: &mut usize) -> Name {
        // FScriptName structure:
        // ComparisonIndex: u32 (FNameEntryId)
        // DisplayIndex: u32 (FNameEntryId)
        // Number: u32
        let _comparison_index = self.read_int(offset) as u32;
        let display_index = self.read_int(offset) as u32;
        let number = self.read_int(offset) as u32;

        // Look up the name in the name map
        let base_name = self
            .names
            .get(&display_index)
            .cloned()
            .unwrap_or_else(|| format!("UnknownName_{}", display_index));

        // Apply the _N suffix if needed
        let name_str = if number == 0 {
            base_name
        } else {
            format!("{}_{}", base_name, number - 1)
        };

        Name::new(name_str)
    }

    pub fn read_address(&self, offset: &mut usize) -> Address {
        Address::new(self.read_qword(offset))
    }
}
