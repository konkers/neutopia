use std::fs::File;
use std::io::{prelude::*, Cursor, SeekFrom};
use std::path::PathBuf;

use failure::Error;
use rand::{self, prelude::*};
use structopt::StructOpt;

use neutopia::{object, object::parse_object_table, Neutopia};
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    #[structopt(long, parse(from_os_str), default_value = "neutopia-jp.pce")]
    rom: PathBuf,

    #[structopt(long, parse(from_os_str), default_value = "neutopia-seed.pce")]
    out: PathBuf,
}

fn shuffle_area(n: &Neutopia, area_index: usize, data: &mut [u8]) -> Result<(), Error> {
    let room_info_table = &n.room_info_tables[area_index];
    let chest_table = &n.chest_tables[&n.chest_table_pointers[area_index]];

    // First find all the chests
    let mut chest_ids = Vec::new();
    let mut chest_contents = Vec::new();
    for (_, room) in room_info_table {
        let object_table = parse_object_table(&room.object_table)?;
        for entry in &object_table {
            if let object::TableEntry::Object(info) = entry {
                if 0x4c <= info.id && info.id <= (0x4c + 8) {
                    let id = info.id - 0x4c;
                    let chest = &chest_table[id as usize];

                    // Ensure it is not the medallion.
                    if chest.item_id < 0x12 || chest.item_id >= (0x12 + 8) {
                        chest_ids.push(id);
                        chest_contents.push(chest.clone());
                    }
                }
            }
        }
    }

    // Next shuffle the chest contents.
    let mut rng = rand::thread_rng();
    chest_contents.shuffle(&mut rng);

    // Patch up a new chest table.
    let mut new_chest_table = chest_table.clone();
    for (chest_id, contents) in chest_ids.iter().zip(chest_contents.iter()) {
        new_chest_table[*chest_id as usize] = contents.clone();
    }

    let mut c = Cursor::new(data);

    // Write the new table to some unused memory.
    let offset = 0x4fe00 + (0x20 * area_index as u64);
    c.seek(SeekFrom::Start(offset))?;
    for chest in &new_chest_table {
        chest.write(&mut c)?;
    }

    // Update the area's chest table pointer.
    c.seek(SeekFrom::Start(
        neutopia::rommap::CHEST_TABLE as u64 + 3 * area_index as u64,
    ))?;
    let ptr = neutopia::util::rom_offset_to_pointer(offset as u32);
    c.write(&ptr)?;

    Ok(())
}

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();
    let mut f = File::open(&opt.rom)?;
    let mut buffer = Vec::new();
    // read the whole file
    f.read_to_end(&mut buffer)?;

    let n = Neutopia::new(&buffer)?;

    for i in 4..=0xb {
        shuffle_area(&n, i, &mut buffer)?;
    }

    let mut f = File::create(&opt.out)?;
    f.write_all(&buffer)?;
    Ok(())
}
