use std::fs::File;
use std::io::{prelude::*, Cursor, SeekFrom};
use std::path::PathBuf;

use failure::{format_err, Error};
use radix_fmt::radix_36;
use rand::{self, prelude::*};
use rand_core::SeedableRng;
use rand_pcg::Pcg32;
use structopt::StructOpt;

use neutopia::{object, object::parse_object_table, Neutopia, Chest};
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    #[structopt(long, parse(from_os_str), default_value = "neutopia-jp.pce")]
    rom: PathBuf,

    #[structopt(long, parse(from_os_str))]
    out: Option<PathBuf>,

    #[structopt(long)]
    seed: Option<String>,
}

/*
    Returns a list of chests that are allowed to be randomized (so no crypt keys, crystal balls or medallions are returned here)
    All these chests are piled on one big heap, randomized and passed into write_new_chests_for_area.
 */
fn get_randomizable_chests_for_area(n: &Neutopia, area_index: usize) -> Vec<Chest> {
    let room_info_table = &n.room_info_tables[area_index];
    let chest_table = &n.chest_tables[&n.chest_table_pointers[area_index]];

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
                    if chest.item_id < 0x10 || chest.item_id >= (0x12 + 8) {
                        chests.push(chest.clone());
                    }
                }
            }
        }
    }

    return chests;
}

/*
    Takes a list of all (remaining) randomizable chests, pops them off one by one for chests it can randomize into
 */
fn write_new_chests_for_area(
    rng: &mut impl Rng,
    n: &Neutopia,
    area_index: usize,
    data: &mut [u8],
    randomizable_chests: &mut Vec<Chest>
) -> Result<(), Error> {
    let room_info_table = &n.room_info_tables[area_index];
    let chest_table = &n.chest_tables[&n.chest_table_pointers[area_index]];

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

                    // Ensure it is not the medallion.
                    if chest.item_id < 0x10 || chest.item_id >= (0x12 + 8) {
                        if let Some(randomizable_chest) = randomizable_chests.pop() {
                            chest_ids.push(id);
                            chest_contents.push(randomizable_chest);
                        } else {
                            return Err(format_err!("Terrible error, ran out of chests! T_T"))
                        }
                    }
                }
            }
        }
    }

    chest_contents.shuffle(rng);

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

    let n = Neutopia::new(&buffer)?;


    let mut chest_pairs = Vec::new();
    for i in 0..=0xf {
        let mut chests = get_randomizable_chests_for_area(&n, i);
        chest_pairs.append(&mut chests);
    }
    chest_pairs.shuffle(&mut rng);

    for i in 0..=0xf {
        write_new_chests_for_area(&mut rng, &n, i, &mut buffer, &mut chest_pairs)?;
    }

    let filename = &opt
        .out
        .unwrap_or(PathBuf::from(format!("neutopia-NR-{:#}.pce", radix_36(seed))));
    let mut f = File::create(filename)?;
    f.write_all(&buffer)?;

    println!("wrote {}", filename.display());
    Ok(())
}
