pub fn pointer_to_rom_offset(data: &[u8; 3]) -> u32 {
    (((data[0] as u32) << 13) | ((data[2] as u32 & 0x1f) << 8) | (data[1] as u32)) - 0x40000
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
