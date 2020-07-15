use std::collections::HashMap;
use std::io::{prelude::*, Cursor, SeekFrom};

use failure::{format_err, Error};

pub mod interval;
pub mod rom;
pub mod rommap;
pub mod util;
pub mod verify;

pub use rom::NeutopiaRom;
pub use verify::{verify, RomInfo};

#[derive(Clone, Debug)]
pub struct Room {
    pub warps: Vec<u8>,
    pub enemies: Vec<u8>,
    pub objects: Vec<rom::object::TableEntry>,
}

#[derive(Clone, Debug)]
pub struct Area {
    pub rooms: Vec<Room>,
    pub chest_table: Vec<rom::Chest>,
}

#[derive(Clone, Debug)]
pub struct Chest {
    pub info: rom::Chest,
    pub area: u8,
    pub room: u8,
    pub index: u8,
}

#[derive(Clone, Debug)]
pub struct Conditional {
    pub data: Vec<rom::object::TableEntry>,
}

pub struct Neutopia {
    pub areas: Vec<Area>,
    pub conditionals: HashMap<rom::Chest, Conditional>,
    pub rom_data: Vec<u8>,
    n: NeutopiaRom,
}

impl Neutopia {
    pub fn new(data: &[u8]) -> Result<Self, Error> {
        let mut rando = Self {
            n: NeutopiaRom::new(data)?,
            areas: Vec::new(),
            conditionals: HashMap::new(),
            rom_data: Vec::from(data),
        };

        for area_idx in 0..=0xf {
            rando.import_area(area_idx)?;
        }

        Ok(rando)
    }

    fn import_area(&mut self, area_idx: usize) -> Result<(), Error> {
        let room_info_table = &self.n.room_info_tables[area_idx];
        let chest_table = &self.n.chest_tables[&self.n.chest_table_pointers[area_idx]];

        let mut rooms = Vec::new();

        for room_idx in 0u8..0x40 {
            let room = &room_info_table[&room_idx];
            let mut object_table = rom::object::parse_object_table(&room.object_table)?;

            // First scan for conditionals, record them, then remove them from the
            // table entries.
            if object_table.len() > 2 {
                let mut i = 0;
                while (i + 2) < object_table.len() {
                    if let Some(id) = object_table[i].chest_id() {
                        let chest = &chest_table[id as usize];
                        let next = object_table[i + 1].clone();
                        let next_next = object_table[i + 2].clone();

                        if next.is_conditional() {
                            object_table.remove(i + 1);
                            object_table.remove(i + 1);
                            self.conditionals.insert(
                                chest.clone(),
                                Conditional {
                                    data: vec![next, next_next],
                                },
                            );
                        }
                    }
                    i += 1;
                }
            }

            rooms.push(Room {
                warps: room.warp_table.clone(),
                enemies: room.enemy_table.clone(),
                objects: object_table,
            });
        }

        self.areas.push(Area {
            rooms,
            chest_table: chest_table.clone(),
        });
        Ok(())
    }

    pub fn filter_chests(&self, filter: impl Fn(&Chest) -> bool) -> Vec<Chest> {
        let mut chests = Vec::new();

        for (area_idx, area) in self.areas.iter().enumerate() {
            for (room_idx, room) in area.rooms.iter().enumerate() {
                let mut chest_index = 0;
                for entry in &room.objects {
                    if let Some(id) = entry.chest_id() {
                        let chest = Chest {
                            info: area.chest_table[id as usize].clone(),
                            area: area_idx as u8,
                            room: room_idx as u8,
                            index: chest_index,
                        };
                        chest_index += 1;
                        if filter(&chest) {
                            chests.push(chest);
                        }
                    }
                }
            }
        }

        chests
    }

    fn get_table_id_for_chest(&self, chest: &Chest) -> Result<usize, Error> {
        let area = &self.areas[chest.area as usize];
        let room = &area.rooms[chest.room as usize];

        let mut chest_index = 0;
        for obj_entry in &room.objects {
            if let Some(id) = obj_entry.chest_id() {
                if chest_index == chest.index {
                    return Ok(id as usize);
                }
                chest_index += 1;
            }
        }

        Err(format_err!("can't find id for chest {:?}", chest))
    }

    pub fn update_chests(&mut self, chests: &[Chest]) -> Result<(), Error> {
        for chest in chests {
            let id = self.get_table_id_for_chest(chest)?;
            let entry = self.areas[chest.area as usize]
                .chest_table
                .get_mut(id as usize)
                .ok_or_else(|| format_err!("incoherent chest id {:02x}", id))?;

            *entry = chest.info.clone();
        }

        Ok(())
    }

    fn write_area(&self, area_idx: usize, rom_writer: &mut Cursor<Vec<u8>>) -> Result<u32, Error> {
        let area = &self.areas[area_idx];
        let cur_offset = rom_writer.position();

        let mut room_ptrs = Cursor::new(Vec::new());
        let room_ptrs_offset = cur_offset;
        let room_data_offset = cur_offset + 0x40 * 3;
        rom_writer.seek(SeekFrom::Start(room_data_offset as u64))?;
        for room_idx in 0..0x40 {
            let room = &area.rooms[room_idx];

            let room_offset = rom_writer.position();
            room_ptrs.write_all(&util::rom_offset_to_pointer(room_offset as u32))?;

            // Add conditionals back to object_table.
            let mut object_table = room.objects.clone();
            for i in 0..object_table.len() {
                if let Some(id) = object_table[i].chest_id() {
                    let chest = &area.chest_table[id as usize];
                    let loc = match object_table[i].loc() {
                        Some(loc) => loc,
                        _ => continue,
                    };
                    if let Some(cond) = self.conditionals.get(&chest) {
                        for j in 0..cond.data.len() {
                            let mut entry = cond.data[j].clone();
                            if let rom::object::TableEntry::Object(ref mut o) = entry {
                                // Patch second objects location to match
                                o.x = loc.0;
                                o.y = loc.1;
                            }
                            object_table.insert(i + j + 1, entry);
                        }
                        break;
                    }
                }
            }

            // Skip over the warp, enemy, and object table pointers for now.
            rom_writer.seek(SeekFrom::Current(3 * 3))?;

            let warp_table_ptr = rom_writer.position() as u32;
            rom_writer.write_all(&room.warps)?;

            let enemy_table_ptr = rom_writer.position() as u32;
            rom_writer.write_all(&room.enemies)?;
            rom_writer.write_all(&[0xff])?;

            let object_table_ptr = rom_writer.position() as u32;
            for o in &object_table {
                o.write(rom_writer)?;
            }
            rom_writer.write_all(&[0xff])?;

            // Rewind and write table pointers.
            let room_end_pos = rom_writer.position();
            rom_writer.seek(SeekFrom::Start(room_offset))?;
            rom_writer.write_all(&util::rom_offset_to_pointer(warp_table_ptr))?;
            rom_writer.write_all(&util::rom_offset_to_pointer(enemy_table_ptr))?;
            rom_writer.write_all(&util::rom_offset_to_pointer(object_table_ptr))?;
            rom_writer.seek(SeekFrom::Start(room_end_pos))?;
        }

        // Record the end of the area.
        let next_offset = rom_writer.position() as u32;

        // Rewind and write out the pointer to the room data.
        rom_writer.seek(SeekFrom::Start(room_ptrs_offset as u64))?;
        rom_writer.write_all(room_ptrs.get_ref())?;

        // And finally write out new area pointer.
        rom_writer.seek(SeekFrom::Start(
            rommap::AREA_TABLE as u64 + area_idx as u64 * 3,
        ))?;
        rom_writer.write_all(&util::rom_offset_to_pointer(room_ptrs_offset as u32))?;

        Ok(next_offset)
    }

    pub fn write(&self) -> Result<Vec<u8>, Error> {
        let mut rom_writer = Cursor::new(self.rom_data.clone());

        let area_range = 4..=0xf;

        // First patch chest tables
        for area_idx in area_range.clone() {
            let area = &self.areas[area_idx];
            // Relocate and write the new chest table.
            let offset = 0x4fe00 + (0x20 * area_idx as u64);
            rom_writer.seek(SeekFrom::Start(offset))?;
            for chest in &area.chest_table {
                chest.write(&mut rom_writer)?;
            }

            // Update the area's chest table pointer.
            rom_writer.seek(SeekFrom::Start(
                rommap::CHEST_TABLE as u64 + 3 * area_idx as u64,
            ))?;
            let ptr = util::rom_offset_to_pointer(offset as u32);
            rom_writer.write_all(&ptr)?;
        }

        // Write out area data

        // Beginning or area data starts where Area 4's data starts.
        let mut cur_offset = self.n.area_pointers[4];
        let mut offset_c = None;
        for area_idx in area_range {
            if area_idx == 0xc {
                offset_c = Some(cur_offset);
            }
            rom_writer.seek(SeekFrom::Start(cur_offset as u64))?;
            cur_offset = self.write_area(area_idx, &mut rom_writer)?
        }

        // Lastly, fixup area 0x10's pointers to match 0xc's
        if let Some(offset) = offset_c {
            rom_writer.seek(SeekFrom::Start(rommap::AREA_TABLE as u64 + 0x10 * 3))?;
            rom_writer.write_all(&util::rom_offset_to_pointer(offset as u32))?;
        }

        Ok(rom_writer.into_inner())
    }
}

pub fn area_name(area: u8) -> &'static str {
    match area {
        0x0 => "Land Sphere",
        0x1 => "Subterranean Sphere",
        0x2 => "Sea Sphere",
        0x3 => "Sky Sphere",
        0x4 => "Crypt 1",
        0x5 => "Crypt 2",
        0x6 => "Crypt 3",
        0x7 => "Crypt 4",
        0x8 => "Crypt 5",
        0x9 => "Crypt 6",
        0xa => "Crypt 7",
        0xb => "Crypt 8",
        0xc => "Land Sphere Rooms",
        0xd => "Subterranean Sphere Rooms",
        0xe => "Sea Sphere Rooms",
        0xf => "Sky Sphere Rooms",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {}
