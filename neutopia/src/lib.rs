use std::collections::HashMap;

use failure::Error;

mod chest;
pub mod object;
pub mod rommap;
pub mod util;

pub use chest::Chest;
pub use object::ObjectInfo;

#[derive(Debug)]
pub struct Room {
    pub base_addr: u32,

    pub warp_table_pointer: u32,
    pub enemy_table_pointer: u32,
    pub object_table_pointer: u32,

    pub warp_table: Vec<u8>,
    pub enemy_table: Vec<u8>,
    pub object_table: Vec<u8>,
}

pub struct Neutopia {
    pub area_pointers: Vec<u32>,
    pub room_order_pointers: Vec<u32>,
    pub chest_table_pointers: Vec<u32>,

    pub room_order_tables: HashMap<u32, Vec<u8>>,
    pub room_info_tables: Vec<HashMap<u8, Room>>,
    pub chest_tables: HashMap<u32, Vec<Chest>>,
}

impl Neutopia {
    pub fn new(data: &[u8]) -> Result<Neutopia, Error> {
        let area_pointers =
            util::decode_pointer_table(&data[rommap::AREA_TABLE..], rommap::AREA_TABLE_COUNT)?;
        let room_order_pointers = util::decode_pointer_table(
            &data[rommap::ROOM_ORDER_TABLE..],
            rommap::ROOM_ORDER_TABLE_COUNT,
        )?;
        let chest_table_pointers =
            util::decode_pointer_table(&data[rommap::CHEST_TABLE..], rommap::CHEST_TABLE_COUNT)?;

        let mut room_info_tables = Vec::new();
        let mut room_order_tables = HashMap::new();
        let mut chest_tables = HashMap::new();

        for area_ptr in &area_pointers {
            let mut area_info = HashMap::new();
            for idx in 0..0x40 {
                let offset = (*area_ptr as usize) + (idx as usize) * 3;
                let offset = util::pointer_to_rom_offset(&data[offset..])? as usize;
                let ptrs = util::decode_pointer_table(&data[offset..], 3)?;
                let warp_table_pointer = ptrs[0];
                let enemy_table_pointer = ptrs[1];
                let object_table_pointer = ptrs[2] as usize;

                // Todo, clean this up once everything parses.
                let object_table = match object::object_table_len(&data[object_table_pointer..]) {
                    Ok(len) => data[object_table_pointer..object_table_pointer + len].to_vec(),
                    Err(_) => util::read_object_table(&data[object_table_pointer as usize..]),
                };

                area_info.insert(
                    idx as u8,
                    Room {
                        base_addr: offset as u32,
                        warp_table_pointer,
                        enemy_table_pointer,
                        object_table_pointer: object_table_pointer as u32,
                        warp_table: Vec::from(
                            &data[(warp_table_pointer as usize)..(enemy_table_pointer as usize)],
                        ),
                        enemy_table: util::read_object_table(&data[enemy_table_pointer as usize..]),
                        object_table,
                    },
                );
            }
            room_info_tables.push(area_info);
        }

        for room_order_ptr in &room_order_pointers {
            let offset = *room_order_ptr as usize;
            let table = Vec::from(&data[offset..offset + 0x40]);

            room_order_tables.insert(*room_order_ptr, table);
        }

        for chest_table_ptr in &chest_table_pointers {
            let table = chest::parse_chest_table(&data[*chest_table_ptr as usize..])?;
            chest_tables.insert(*chest_table_ptr, table);
        }

        Ok(Neutopia {
            area_pointers,
            room_order_pointers,
            chest_table_pointers,
            room_order_tables,
            room_info_tables,
            chest_tables,
        })
    }
}
#[cfg(test)]
mod tests {}
