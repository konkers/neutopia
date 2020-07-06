use std::collections::HashMap;
use std::fs::File;
use std::io::{prelude::*, Cursor, SeekFrom};
use std::path::PathBuf;

use failure::{format_err, Error};
use radix_fmt::radix_36;
use rand::{self, prelude::*};
use rand_core::SeedableRng;
use rand_pcg::Pcg32;
use structopt::StructOpt;

use neutopia::{self, object, object::parse_object_table, Neutopia};
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    #[structopt(long, parse(from_os_str), default_value = "Neutopia (USA).pce")]
    rom: PathBuf,

    #[structopt(long, parse(from_os_str))]
    out: Option<PathBuf>,

    #[structopt(long)]
    seed: Option<String>,

    #[structopt(long)]
    global: bool,
}

#[derive(Clone, Debug)]
struct Room {
    pub warps: Vec<u8>,
    pub enemies: Vec<u8>,
    pub objects: Vec<object::TableEntry>,
}

#[derive(Clone, Debug)]
struct Area {
    pub rooms: Vec<Room>,
    pub chest_table: Vec<neutopia::Chest>,
}

#[derive(Clone, Debug)]
struct Chest {
    info: neutopia::Chest,
    area: u8,
    room: u8,
    id: usize,
}
#[derive(Clone, Debug)]
struct Conditional {
    data: Vec<object::TableEntry>,
}

struct Randomizer {
    pub areas: Vec<Area>,
    pub conditionals: HashMap<neutopia::Chest, Conditional>,
    pub rom_data: Vec<u8>,
    n: Neutopia,
}

impl Randomizer {
    pub fn new(data: &[u8]) -> Result<Randomizer, Error> {
        let mut rando = Randomizer {
            n: Neutopia::new(data)?,
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
            let mut object_table = parse_object_table(&room.object_table)?;

            // First scan for conditionals, record them, then remove them from the
            // table entries.
            if object_table.len() > 2 {
                for i in 0..(object_table.len() - 2) {
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
                            break;
                        }
                    }
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

    fn filter_chests(&self, filter: impl Fn(&Chest) -> bool) -> Vec<Chest> {
        let mut chests = Vec::new();

        for (area_idx, area) in self.areas.iter().enumerate() {
            for (room_idx, room) in area.rooms.iter().enumerate() {
                for entry in &room.objects {
                    if let Some(id) = entry.chest_id() {
                        let chest = Chest {
                            info: area.chest_table[id as usize].clone(),
                            area: area_idx as u8,
                            room: room_idx as u8,
                            id: id as usize,
                        };
                        if filter(&chest) {
                            chests.push(chest);
                        }
                    }
                }
            }
        }

        chests
    }

    fn update_chests(&mut self, chests: &[Chest]) -> Result<(), Error> {
        for chest in chests {
            let entry = self.areas[chest.area as usize]
                .chest_table
                .get_mut(chest.id)
                .ok_or_else(|| format_err!("incoherent chest id {:02x}", chest.id))?;

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
            room_ptrs.write_all(&neutopia::util::rom_offset_to_pointer(room_offset as u32))?;

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
                            if let object::TableEntry::Object(ref mut o) = entry {
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
            rom_writer.write_all(&neutopia::util::rom_offset_to_pointer(warp_table_ptr))?;
            rom_writer.write_all(&neutopia::util::rom_offset_to_pointer(enemy_table_ptr))?;
            rom_writer.write_all(&neutopia::util::rom_offset_to_pointer(object_table_ptr))?;
            rom_writer.seek(SeekFrom::Start(room_end_pos))?;
        }

        // Record the end of the area.
        let next_offset = rom_writer.position() as u32;

        // Rewind and write out the pointer to the room data.
        rom_writer.seek(SeekFrom::Start(room_ptrs_offset as u64))?;
        rom_writer.write_all(room_ptrs.get_ref())?;

        // And finally write out new area pointer.
        rom_writer.seek(SeekFrom::Start(
            neutopia::rommap::AREA_TABLE as u64 + area_idx as u64 * 3,
        ))?;
        rom_writer.write_all(&neutopia::util::rom_offset_to_pointer(
            room_ptrs_offset as u32,
        ))?;

        Ok(next_offset)
    }

    fn write(&self) -> Result<Vec<u8>, Error> {
        let mut rom_writer = Cursor::new(self.rom_data.clone());

        // For now we're only doing crypts
        let area_range = 4..=0xb;

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
                neutopia::rommap::CHEST_TABLE as u64 + 3 * area_idx as u64,
            ))?;
            let ptr = neutopia::util::rom_offset_to_pointer(offset as u32);
            rom_writer.write_all(&ptr)?;
        }

        // Write out area data

        // Beginning or area data starts where Area 4's data starts.
        let mut cur_offset = self.n.area_pointers[4];
        for area_idx in area_range {
            rom_writer.seek(SeekFrom::Start(cur_offset as u64))?;
            cur_offset = self.write_area(area_idx, &mut rom_writer)?
        }

        Ok(rom_writer.into_inner())
    }
}

// Shuffle all items within each crypt.  Does not touch overworld items.
fn crypt_rando(rng: &mut impl Rng, rom_data: &[u8]) -> Result<Vec<u8>, Error> {
    let mut rando = Randomizer::new(rom_data)?;

    for area_idx in 0x4..=0xb {
        // Find all the chest we want to randomize.
        let mut chests = rando.filter_chests(|chest| {
            // Chest is in current area
            (chest.area == area_idx)
                // Chest does not contain medallion
                && (chest.info.item_id < 0x12 || chest.info.item_id >= (0x12 + 8))
        });

        // Shuffle the chests.
        let mut randomized_chests: Vec<neutopia::Chest> =
            chests.iter().map(|chest| chest.info.clone()).collect();
        randomized_chests.shuffle(rng);

        // Update the chests' info
        for (i, chest) in chests.iter_mut().enumerate() {
            chest.info = randomized_chests[i].clone();
        }

        rando.update_chests(&chests)?;
    }

    rando.write()
}

// Shuffle all items across crypts and overworld.  Does not contain logic
// to make sure seed is completable.
fn global_rando(rng: &mut impl Rng, rom_data: &[u8]) -> Result<Vec<u8>, Error> {
    let mut rando = Randomizer::new(rom_data)?;

    let mut chests = rando.filter_chests(|chest| {
        // Chest is in current area
        (chest.area < 0x10)
                // Chest does not contain medallion, crystal ball, or key
                && (chest.info.item_id < 0x10 || chest.info.item_id >= (0x12 + 8))
    });

    // Shuffle the chests.
    let mut randomized_chests: Vec<neutopia::Chest> =
        chests.iter().map(|chest| chest.info.clone()).collect();
    randomized_chests.shuffle(rng);

    // Update the chests' info
    for (i, chest) in chests.iter_mut().enumerate() {
        chest.info = randomized_chests[i].clone();
    }

    rando.update_chests(&chests)?;
    rando.write()
}

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();

    // Let the user specify a seed in base36.  Otherwise randomly generate one.
    let seed = match &opt.seed {
        Some(s) => u64::from_str_radix(s, 36)
            .map_err(|e| format_err!("Seed name must be a valid base36 64 bit number: {}", e))?,
        None => rand::thread_rng().gen(),
    };

    let mut rng = Pcg32::seed_from_u64(seed);

    let mut f = File::open(&opt.rom)?;
    let mut buffer = Vec::new();
    // read the whole file
    f.read_to_end(&mut buffer)?;
    // drop mutability of buffer
    let buffer = buffer;

    let new_data = if opt.global {
        global_rando(&mut rng, &buffer)?
    } else {
        crypt_rando(&mut rng, &buffer)?
    };
    let filename = &opt
        .out
        .unwrap_or_else(|| PathBuf::from(format!("neutopia-randomizer-{:#}.pce", radix_36(seed))));
    let mut f = File::create(filename)?;
    f.write_all(&new_data)?;

    println!("wrote {}", filename.display());
    Ok(())
}
