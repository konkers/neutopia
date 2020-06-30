use failure::Error;
use std::collections::HashMap;

mod rommap;
mod util;

pub struct Neutopia {
    pub area_pointers: Vec<u32>,
    pub room_order_pointers: Vec<u32>,

    pub room_order_tables: HashMap<u32, Vec<u8>>,
}

impl Neutopia {
    pub fn new(data: &[u8]) -> Result<Neutopia, Error> {
        let area_pointers =
            util::decode_pointer_table(&data[rommap::AREA_TABLE..], rommap::AREA_TABLE_COUNT)?;
        let room_order_pointers = util::decode_pointer_table(
            &data[rommap::ROOM_ORDER_TABLE..],
            rommap::ROOM_ORDER_TABLE_COUNT,
        )?;

        let mut room_order_tables = HashMap::new();
        for ptr in &room_order_pointers {
            let offset = *ptr as usize;
            let table = Vec::from(&data[offset..offset + 0x40]);
            room_order_tables.insert(*ptr, table);
        }

        Ok(Neutopia {
            area_pointers,
            room_order_pointers,
            room_order_tables,
        })
    }
}
#[cfg(test)]
mod tests {}
