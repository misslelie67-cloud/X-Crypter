// PE File Reader
// Reads and parses PE (Portable Executable) files

use std::fs;

/// PE file structure
/// Note: dos_header, pe_header, and sections will be used in Phase 3 for PE loader
#[derive(Debug, Clone)]
pub struct PEFile {
    #[allow(dead_code)] // Will be used in Phase 3
    pub dos_header: DOSHeader,
    #[allow(dead_code)] // Will be used in Phase 3
    pub pe_header: PEHeader,
    #[allow(dead_code)] // Will be used in Phase 3
    pub sections: Vec<SectionHeader>,
    pub raw_data: Vec<u8>, // Full file data - currently used for encryption
}

/// DOS Header (MZ signature)
#[derive(Debug, Clone)]
pub struct DOSHeader {
    pub signature: [u8; 2], // "MZ"
    pub e_lfanew: u32,      // Offset to PE header
}

/// PE Header
/// Note: Fields will be used in Phase 3 for PE loader
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PEHeader {
    pub signature: [u8; 4], // "PE\0\0"
    pub machine: u16,
    pub number_of_sections: u16,
    pub time_date_stamp: u32,
    pub pointer_to_symbol_table: u32,
    pub number_of_symbols: u32,
    pub size_of_optional_header: u16,
    pub characteristics: u16,
}

/// Section Header
/// Note: Fields will be used in Phase 3 for PE loader
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SectionHeader {
    pub name: [u8; 8],
    pub virtual_size: u32,
    pub virtual_address: u32,
    pub size_of_raw_data: u32,
    pub pointer_to_raw_data: u32,
    pub characteristics: u32,
}

/// Read and parse PE file
pub fn read_pe_file(file_path: &str) -> Result<PEFile, String> {
    // Read entire file
    let raw_data = fs::read(file_path).map_err(|e| format!("Failed to read file: {}", e))?;

    if raw_data.len() < 64 {
        return Err("File too small to be a valid PE".to_string());
    }

    // Parse DOS header
    let dos_header = parse_dos_header(&raw_data)?;

    // Check MZ signature
    if dos_header.signature != [0x4D, 0x5A] {
        return Err("Invalid DOS signature (not MZ)".to_string());
    }

    // Get PE header offset
    let pe_offset = dos_header.e_lfanew as usize;
    if pe_offset >= raw_data.len() {
        return Err("PE header offset out of bounds".to_string());
    }

    // Parse PE header
    let pe_header = parse_pe_header(&raw_data[pe_offset..])?;

    // Check PE signature
    if pe_header.signature != [0x50, 0x45, 0x00, 0x00] {
        return Err("Invalid PE signature".to_string());
    }

    // Parse section headers
    let sections_offset = pe_offset + 24 + pe_header.size_of_optional_header as usize;
    let sections = parse_sections(
        &raw_data[sections_offset..],
        pe_header.number_of_sections as usize,
    )?;

    Ok(PEFile {
        dos_header,
        pe_header,
        sections,
        raw_data,
    })
}

/// Parse DOS header
fn parse_dos_header(data: &[u8]) -> Result<DOSHeader, String> {
    if data.len() < 64 {
        return Err("Data too short for DOS header".to_string());
    }

    let signature = [data[0], data[1]];
    let e_lfanew = u32::from_le_bytes([data[60], data[61], data[62], data[63]]);

    Ok(DOSHeader {
        signature,
        e_lfanew,
    })
}

/// Parse PE header
fn parse_pe_header(data: &[u8]) -> Result<PEHeader, String> {
    if data.len() < 24 {
        return Err("Data too short for PE header".to_string());
    }

    let signature = [data[0], data[1], data[2], data[3]];
    let machine = u16::from_le_bytes([data[4], data[5]]);
    let number_of_sections = u16::from_le_bytes([data[6], data[7]]);
    let time_date_stamp = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
    let pointer_to_symbol_table = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
    let number_of_symbols = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);
    let size_of_optional_header = u16::from_le_bytes([data[20], data[21]]);
    let characteristics = u16::from_le_bytes([data[22], data[23]]);

    Ok(PEHeader {
        signature,
        machine,
        number_of_sections,
        time_date_stamp,
        pointer_to_symbol_table,
        number_of_symbols,
        size_of_optional_header,
        characteristics,
    })
}

/// Parse section headers
fn parse_sections(data: &[u8], count: usize) -> Result<Vec<SectionHeader>, String> {
    let mut sections = Vec::new();
    let section_size = 40; // Each section header is 40 bytes

    if data.len() < count * section_size {
        return Err("Data too short for section headers".to_string());
    }

    for i in 0..count {
        let offset = i * section_size;
        let section_data = &data[offset..offset + section_size];

        let mut name = [0u8; 8];
        name.copy_from_slice(&section_data[0..8]);

        let virtual_size = u32::from_le_bytes([
            section_data[8],
            section_data[9],
            section_data[10],
            section_data[11],
        ]);
        let virtual_address = u32::from_le_bytes([
            section_data[12],
            section_data[13],
            section_data[14],
            section_data[15],
        ]);
        let size_of_raw_data = u32::from_le_bytes([
            section_data[16],
            section_data[17],
            section_data[18],
            section_data[19],
        ]);
        let pointer_to_raw_data = u32::from_le_bytes([
            section_data[20],
            section_data[21],
            section_data[22],
            section_data[23],
        ]);
        let characteristics = u32::from_le_bytes([
            section_data[36],
            section_data[37],
            section_data[38],
            section_data[39],
        ]);

        sections.push(SectionHeader {
            name,
            virtual_size,
            virtual_address,
            size_of_raw_data,
            pointer_to_raw_data,
            characteristics,
        });
    }

    Ok(sections)
}

/// Extract .text section from PE file
/// Note: Will be used in Phase 3 for PE loader
#[allow(dead_code)]
pub fn extract_text_section(pe_file: &PEFile) -> Option<Vec<u8>> {
    for section in &pe_file.sections {
        let section_name = String::from_utf8_lossy(&section.name);
        if section_name.trim_end_matches('\0') == ".text" {
            let start = section.pointer_to_raw_data as usize;
            let end = start + section.size_of_raw_data as usize;
            if end <= pe_file.raw_data.len() {
                return Some(pe_file.raw_data[start..end].to_vec());
            }
        }
    }
    None
}

/// Get entire PE file data (for full encryption)
pub fn get_full_pe_data(pe_file: &PEFile) -> &[u8] {
    &pe_file.raw_data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dos_header() {
        let mut data = vec![0u8; 64];
        data[0] = 0x4D; // 'M'
        data[1] = 0x5A; // 'Z'
        data[60] = 0x80;
        data[61] = 0x00;
        data[62] = 0x00;
        data[63] = 0x00;

        let dos_header = parse_dos_header(&data).unwrap();
        assert_eq!(dos_header.signature, [0x4D, 0x5A]);
        assert_eq!(dos_header.e_lfanew, 128);
    }
}
