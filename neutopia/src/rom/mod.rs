use std::collections::HashMap;

use failure::{format_err, Error};

use super::{interval::IntervalStore, rommap, util};

mod chest;
pub mod object;
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

pub struct NeutopiaRom {
    pub area_pointers: Vec<u32>,
    pub room_order_pointers: Vec<u32>,
    pub chest_table_pointers: Vec<u32>,

    pub room_order_tables: HashMap<u32, Vec<u8>>,
    pub room_info_tables: Vec<HashMap<u8, Room>>,
    pub chest_tables: HashMap<u32, Vec<Chest>>,

    pub room_info_intervals: HashMap<u8, IntervalStore<usize>>,
}

impl NeutopiaRom {
    pub fn new(data: &[u8]) -> Result<NeutopiaRom, Error> {
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
        let mut room_info_intervals = HashMap::new();

        for (area_idx, area_ptr) in area_pointers.iter().enumerate() {
            let mut room_data_intervals: IntervalStore<usize> = IntervalStore::new();
            room_data_intervals.add(*area_ptr as usize, *area_ptr as usize + 0x40 * 3);
            let mut area_info = HashMap::new();
            for idx in 0..0x40 {
                let offset = (*area_ptr as usize) + (idx as usize) * 3;
                let offset = util::pointer_to_rom_offset(&data[offset..]).map_err(|e| {
                    format_err!(
                        "can't decode room pointer {:02x}:{:02x}: {}",
                        area_idx,
                        idx,
                        e
                    )
                })? as usize;

                let ptrs = util::decode_pointer_table(&data[offset..], 3).map_err(|e| {
                    format_err!(
                        "can't decode room table pointers {:02x}:{:02x}: {}",
                        area_idx,
                        idx,
                        e
                    )
                })?;
                let warp_table_pointer = ptrs[0] as usize;
                let enemy_table_pointer = ptrs[1] as usize;
                let object_table_pointer = ptrs[2] as usize;

                room_data_intervals.add(offset, offset + 3 * 3);

                let warp_table = Vec::from(&data[warp_table_pointer..enemy_table_pointer]);
                let enemy_table = util::read_object_table(&data[enemy_table_pointer..]);
                // Todo, clean this up once everything parses.
                let len = object::object_table_len(&data[object_table_pointer..])?;
                let object_table = data[object_table_pointer..object_table_pointer + len].to_vec();

                room_data_intervals.add(warp_table_pointer, warp_table_pointer + warp_table.len());
                room_data_intervals.add(
                    enemy_table_pointer,
                    enemy_table_pointer + enemy_table.len() + 1,
                );
                room_data_intervals.add(object_table_pointer, object_table_pointer + len + 1);

                area_info.insert(
                    idx as u8,
                    Room {
                        base_addr: offset as u32,
                        warp_table_pointer: warp_table_pointer as u32,
                        enemy_table_pointer: enemy_table_pointer as u32,
                        object_table_pointer: object_table_pointer as u32,
                        warp_table,
                        enemy_table,
                        object_table,
                    },
                );
            }
            room_info_tables.push(area_info);
            room_info_intervals.insert(area_idx as u8, room_data_intervals);
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

        Ok(NeutopiaRom {
            area_pointers,
            room_order_pointers,
            chest_table_pointers,
            room_order_tables,
            room_info_tables,
            chest_tables,
            room_info_intervals,
        })
    }
}
