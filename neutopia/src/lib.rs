use failure::Error;
use std::io::{Cursor, Read};

mod rommap;
mod util;

pub struct Neutopia {
    pub area_pointers: Vec<u32>,
}

impl Neutopia {
    pub fn new(data: &[u8]) -> Result<Neutopia, Error> {
        let mut c = Cursor::new(&data[rommap::AREA_TABLE..]);
        let mut area_pointers = Vec::new();
        for _ in 0..rommap::AREA_TABLE_COUNT {
            let mut pointer_data = [0; 3];
            c.read_exact(&mut pointer_data)?;
            area_pointers.push(util::pointer_to_rom_offset(&pointer_data));
        }
        Ok(Neutopia { area_pointers })
    }
}
#[cfg(test)]
mod tests {}
