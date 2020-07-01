use failure::{format_err, Error};

pub fn pointer_to_rom_offset(data: &[u8]) -> Result<u32, Error> {
    assert!(data.len() >= 3);

    let value = ((data[0] as u32) << 13) | ((data[2] as u32 & 0x1f) << 8) | (data[1] as u32);
    if value < 0x40000 {
        Err(format_err!(
            "can't convert: {:02x} {:02x} {:02x}",
            data[0],
            data[1],
            data[2]
        ))
    } else {
        Ok(value - 0x40000)
    }
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
        let pointer = pointer_to_rom_offset(&data[(i * 3)..])?;
        table.push(pointer);
    }

    Ok(table)
}

pub fn read_object_table(data: &[u8]) -> Vec<u8> {
    let mut table = Vec::new();
    let mut i = 0;
    loop {
        let val = data[i];
        if val == 0xff {
            break;
        }
        table.push(val);

        i += 1;
    }
    table
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_pointer_to_rom_offset() {
        assert_eq!(pointer_to_rom_offset(&[0x48, 0x4e, 0x45]).unwrap(), 0x5054e);
        assert_eq!(pointer_to_rom_offset(&[0x49, 0x44, 0x51]).unwrap(), 0x53144);
    }
}
