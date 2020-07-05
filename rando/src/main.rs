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

use neutopia::{object, object::parse_object_table, Chest, Neutopia};
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    #[structopt(long, parse(from_os_str), default_value = "neutopia-jp.pce")]
    rom: PathBuf,

    #[structopt(long, parse(from_os_str))]
    out: Option<PathBuf>,

    #[structopt(long)]
    seed: Option<String>,

    #[structopt(long)]
    global: bool,
}

struct Conditional {
    data: Vec<object::TableEntry>,
}

fn get_conditional_items_for_area(
    n: &mut Neutopia,
    area_index: usize,
) -> Result<HashMap<Chest, Conditional>, Error> {
    let mut items = HashMap::new();
    let room_info_table = n.room_info_tables.get_mut(area_index).ok_or(format_err!(
        "Can't get room info table for area {:02x}",
        area_index
    ))?;
    let chest_table = &n.chest_tables[&n.chest_table_pointers[area_index]];

    for room_id in 0u8..0x40 {
        let room = room_info_table
            .get_mut(&room_id)
            .ok_or(format_err!("Can't get room ID {:2x}", &room_id))?;
        let mut object_table = parse_object_table(&room.object_table).unwrap_or_default();

        if object_table.len() > 2 {
            for i in 0..(object_table.len() - 2) {
                if let Some(id) = object_table[i].chest_id() {
                    let chest = &chest_table[id as usize];
                    let next = object_table[i + 1].clone();
                    let next_next = object_table[i + 2].clone();

                    if next.is_conditional() {
                        object_table.remove(i + 1);
                        object_table.remove(i + 1);
                        items.insert(
                            chest.clone(),
                            Conditional {
                                data: vec![next, next_next],
                            },
                        );
                        break;
                    }
                }
            }
            // write back the object table;
            let mut data: Vec<u8> = Vec::new();
            for o in &object_table {
                o.write(&mut data)?;
            }
            room.object_table = data;
        }
    }

    Ok(items)
}

fn patch_conditional_items_for_area(
    n: &mut Neutopia,
    area_index: usize,
    conditionals: &HashMap<Chest, Conditional>,
) -> Result<(), Error> {
    let room_info_table = n.room_info_tables.get_mut(area_index).ok_or(format_err!(
        "Can't get room info table for area {:02x}",
        area_index
    ))?;
    let chest_table = &n.chest_tables[&n.chest_table_pointers[area_index]];

    for room_id in 0u8..0x40 {
        let room = room_info_table
            .get_mut(&room_id)
            .ok_or(format_err!("Can't get room ID {:2x}", &room_id))?;
        let mut object_table = parse_object_table(&room.object_table).unwrap_or_default();

        for i in 0..object_table.len() {
            if let Some(id) = object_table[i].chest_id() {
                let chest = &chest_table[id as usize];
                let loc = match object_table[i].loc() {
                    Some(loc) => loc,
                    _ => continue,
                };
                if let Some(cond) = conditionals.get(&chest) {
                    for j in 0..cond.data.len() {
                        let mut entry = cond.data[j].clone();
                        if let object::TableEntry::Object(ref mut o) = entry {
                            o.x = loc.0;
                            o.y = loc.1;
                        }
                        object_table.insert(i + j + 1, entry);
                    }
                    break;
                }
            }
        }

        // write back the object table;
        let mut data: Vec<u8> = Vec::new();
        for o in &object_table {
            o.write(&mut data)?;
        }
        room.object_table = data;
    }

    Ok(())
}

// Returns a list of chests that are allowed to be randomized (so no crypt keys, crystal balls or medallions are returned here)
// All these chests are piled on one big heap, randomized and passed into write_new_chests_for_area.
fn get_randomizable_chests_for_area(n: &Neutopia, area_index: usize) -> Vec<Chest> {
    let room_info_table = &n.room_info_tables[area_index];
    let chest_table = &n.chest_tables[&n.chest_table_pointers[area_index]];

    // find all chests that are OK to randomize
    let mut chests = Vec::new();
    for room_id in 0u8..0x40 {
        let room = &room_info_table[&room_id];
        let object_table = parse_object_table(&room.object_table).unwrap_or_default();
        for entry in &object_table {
            if let object::TableEntry::Object(info) = entry {
                if 0x4c <= info.id && info.id <= (0x4c + 8) {
                    let id = info.id - 0x4c;
                    let chest = &chest_table[id as usize];

                    // Ensure it is not the medallion, crypt key or crystal ball.
                    if (chest.item_id < 0x12 || chest.item_id >= (0x12 + 8))
                        && (chest.item_id != 13)
                    {
                        chests.push(chest.clone());
                    }
                }
            }
        }
    }

    chests
}

// Takes a list of all (remaining) randomizable chests, pops them off one by one for chests it can randomize into
fn write_new_chests_for_area<W: Write + Seek>(
    n: &mut Neutopia,
    area_index: usize,
    w: &mut W,
    randomizable_chests: &mut Vec<Chest>,
) -> Result<(), Error> {
    let room_info_table = &n.room_info_tables[area_index];
    let chest_table = &n.chest_tables[&n.chest_table_pointers[area_index]];

    // find all chests that are OK to randomize and replace them with the top chest in randomizable_chests
    let mut chest_ids = Vec::new();
    let mut chest_contents = Vec::new();
    for room_id in 0u8..0x40 {
        let room = &room_info_table[&room_id];
        let object_table = parse_object_table(&room.object_table)?;
        for entry in &object_table {
            if let object::TableEntry::Object(info) = entry {
                if 0x4c <= info.id && info.id <= (0x4c + 8) {
                    let id = info.id - 0x4c;
                    let chest = &chest_table[id as usize];

                    // Ensure it is not the medallion, crypt key or crystal ball.
                    //if chest.item_id < 0x10 || chest.item_id >= (0x12 + 8) {

                    // Ensure it is not the medallion.
                    if (chest.item_id < 0x12 || chest.item_id >= (0x12 + 8))
                        && (chest.item_id != 13)
                    {
                        if let Some(randomizable_chest) = randomizable_chests.pop() {
                            chest_ids.push(id);
                            chest_contents.push(randomizable_chest);
                        } else {
                            return Err(format_err!("Terrible error, ran out of chests! T_T"));
                        }
                    }
                }
            }
        }
    }

    // Patch up a new chest table
    let mut new_chest_table = chest_table.clone();
    for (chest_id, contents) in chest_ids.iter().zip(chest_contents.iter()) {
        new_chest_table[*chest_id as usize] = contents.clone();
    }

    // Write the new table to some unused memory.
    let offset = 0x4fe00 + (0x20 * area_index as u64);
    w.seek(SeekFrom::Start(offset))?;
    for chest in &new_chest_table {
        chest.write(w)?;
    }

    // Update the area's chest table pointer.
    w.seek(SeekFrom::Start(
        neutopia::rommap::CHEST_TABLE as u64 + 3 * area_index as u64,
    ))?;
    let ptr = neutopia::util::rom_offset_to_pointer(offset as u32);
    w.write_all(&ptr)?;

    *(n.chest_tables
        .get_mut(&n.chest_table_pointers[area_index])
        .unwrap()) = new_chest_table;
    Ok(())
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

    let mut n = Neutopia::new(&buffer)?;
    let mut rom_writer = Cursor::new(buffer.clone());

    if opt.global {
        // get all chests that are allowed to be randomized from the areas and put them on one big heap
        let mut randomizable_chests = Vec::new();
        for i in 0..=0xf {
            let mut chests = get_randomizable_chests_for_area(&n, i);
            randomizable_chests.append(&mut chests);
        }

        // shuffle!
        randomizable_chests.shuffle(&mut rng);

        // Have each area pick chests from the heap one by one and pass the remaining to the next area
        for i in 0..=0xf {
            write_new_chests_for_area(&mut n, i, &mut rom_writer, &mut randomizable_chests)?;
        }
    } else {
        let mut cur_offset = n.area_pointers[4];

        for i in 4..=0xb {
            let mut chests = get_randomizable_chests_for_area(&n, i);
            let conditionals = get_conditional_items_for_area(&mut n, i)?;

            chests.shuffle(&mut rng);
            write_new_chests_for_area(&mut n, i, &mut rom_writer, &mut chests)?;

            // Now go through the shuffled rooms and add back in conditionals.
            patch_conditional_items_for_area(&mut n, i, &conditionals)?;

            let mut room_ptrs = Cursor::new(Vec::new());

            let room_ptrs_offset = cur_offset;
            let room_data_offset = cur_offset + 0x40 * 3;
            rom_writer.seek(SeekFrom::Start(room_data_offset as u64))?;

            for room_id in 0..0x40 {
                let room = &n.room_info_tables[i][&room_id];

                room_ptrs.write_all(&neutopia::util::rom_offset_to_pointer(
                    rom_writer.position() as u32,
                ))?;

                let ptrs_pos = rom_writer.position();

                // Skip over the warp, enemy, and object table pointers for now.
                rom_writer.seek(SeekFrom::Current(3 * 3))?;

                let warp_table_ptr = rom_writer.position() as u32;
                rom_writer.write_all(&room.warp_table)?;

                let enemy_table_ptr = rom_writer.position() as u32;
                rom_writer.write_all(&room.enemy_table)?;
                rom_writer.write_all(&[0xff])?;

                let object_table_ptr = rom_writer.position() as u32;
                rom_writer.write_all(&room.object_table)?;
                rom_writer.write_all(&[0xff])?;

                // Rewind and write table pointers.
                let room_end_pos = rom_writer.position();

                rom_writer.seek(SeekFrom::Start(ptrs_pos))?;
                rom_writer.write_all(&neutopia::util::rom_offset_to_pointer(warp_table_ptr))?;
                rom_writer.write_all(&neutopia::util::rom_offset_to_pointer(enemy_table_ptr))?;
                rom_writer.write_all(&neutopia::util::rom_offset_to_pointer(object_table_ptr))?;
                rom_writer.seek(SeekFrom::Start(room_end_pos))?;
            }

            // Record the end of the area.
            cur_offset = rom_writer.position() as u32;

            // Rewind and write out the points to the room data.
            rom_writer.seek(SeekFrom::Start(room_ptrs_offset as u64))?;
            rom_writer.write_all(room_ptrs.get_ref())?;

            // And finally write out new area pointer.
            rom_writer.seek(SeekFrom::Start(
                neutopia::rommap::AREA_TABLE as u64 + i as u64 * 3,
            ))?;
            rom_writer.write_all(&neutopia::util::rom_offset_to_pointer(room_ptrs_offset))?;
        }
    }
    let filename = &opt
        .out
        .unwrap_or_else(|| PathBuf::from(format!("neutopia-randomizer-{:#}.pce", radix_36(seed))));
    let mut f = File::create(filename)?;
    f.write_all(rom_writer.get_ref())?;

    println!("wrote {}", filename.display());
    Ok(())
}
