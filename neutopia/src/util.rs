use failure::{format_err, Error};

fn pointer_to_rom_offset(data: &[u8]) -> u32 {
    assert!(data.len() >= 3);
    (((data[0] as u32) << 13) | ((data[2] as u32 & 0x1f) << 8) | (data[1] as u32)) - 0x40000
}

pub fn decode_pointer_table(data: &[u8], entries: usize) -> Result<Vec<u32>, Error> {
    if data.len() < entries * 3 {
        return Err(format_err!(
            "data only {} bytes in length.  Need at least {}",
            data.len(),
            entries * 3
        ));
    }

    let mut table = Vec::new();

    for i in 0..entries {
        let pointer = pointer_to_rom_offset(&data[(i * 3)..]);
        table.push(pointer);
    }

    Ok(table)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_pointer_to_rom_offset() {
        assert_eq!(pointer_to_rom_offset(&[0x48, 0x4e, 0x45]), 0x5054e);
        assert_eq!(pointer_to_rom_offset(&[0x49, 0x44, 0x51]), 0x53144);
    }
}
