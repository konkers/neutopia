use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

use failure::Error;
use structopt::StructOpt;

use neutopia::{object::parse_object_table, Neutopia};

#[derive(StructOpt, Debug)]
pub(crate) struct DocOpt {
    #[structopt(long, parse(from_os_str), default_value = "neutopia-jp.pce")]
    rom: PathBuf,

    #[structopt(long, parse(from_os_str), default_value = "out")]
    outdir: PathBuf,
}

fn write_byte_array(f: &mut File, data: &[u8]) -> Result<(), Error> {
    write!(f, "```\n[")?;
    for (i, val) in data.iter().enumerate() {
        if i != 0 {
            write!(f, ", ")?;
        }
        write!(f, "{:02x}", val)?;
    }
    writeln!(f, "]\n```\n")?;
    Ok(())
}

fn write_area_markdown(opt: &DocOpt, n: &Neutopia, area_index: usize) -> Result<(), Error> {
    let mut path: PathBuf = opt.outdir.clone();
    path.push(format!("area_{:02x}.md", area_index));
    let mut f = File::create(path)?;

    writeln!(f, "# Area {:01X}\n", area_index)?;

    writeln!(f, "## Overview\n")?;
    writeln!(f, "| | |")?;
    writeln!(f, "|-|-|")?;
    writeln!(
        f,
        "| Area data table pointer | {:05x} |",
        n.area_pointers[area_index]
    )?;
    writeln!(
        f,
        "| Room order table pointer | {:05x} |",
        n.room_order_pointers[area_index]
    )?;
    if area_index < n.chest_table_pointers.len() {
        writeln!(
            f,
            "| Chest table pointer | {:05x} |",
            n.chest_table_pointers[area_index]
        )?;
    }
    writeln!(f)?;

    let addr = n.room_order_pointers[area_index];
    let table = &n.room_order_tables[&addr];
    writeln!(f, "## Room Map/Order\n")?;
    writeln!(f, "|   | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 |")?;
    write!(f, "  |---|---|---|---|---|---|---|---|---|")?;
    for (i, room_id) in table.iter().enumerate() {
        if i % 8 == 0x00 {
            write!(f, "\n| {} |", &i / 8)?;
        }

        write!(f, " [{:02x}](#room-{:02x}) |", room_id, i)?;
    }

    writeln!(f)?;

    if area_index < n.chest_table_pointers.len() {
        writeln!(f, "## Chests\n")?;
        writeln!(f, "| index | item id | arg | text | ?? | item name |")?;
        writeln!(f, "|-------|---------|-----|------|----|-----------|")?;
        let chest_table = &n.chest_tables[&n.chest_table_pointers[area_index]];
        for (i, chest) in chest_table.iter().enumerate() {
            writeln!(
                f,
                "| {} | {:02x} | {:02x} | {:02x} | {:02x} | {} |",
                i,
                &chest.item_id,
                &chest.arg,
                &chest.text,
                &chest.unknown,
                chest.get_item_name()
            )?;
        }
    }

    writeln!(f, "## Rooms\n")?;
    let rooms = &n.room_info_tables[area_index];
    let mut room_ids: Vec<u8> = rooms.keys().copied().collect();
    room_ids.sort();

    for room_id in room_ids {
        let room = &rooms[&room_id];
        writeln!(
            f,
            "### Room {:02X}\n\n ***({}, {}) @ {:05x}***\n",
            room_id,
            room_id / 8,
            room_id % 8,
            room.base_addr,
        )?;
        writeln!(f, "| | |")?;
        writeln!(f, "|-|-|")?;
        writeln!(f, "| warp table ptr | {:05x} |", room.warp_table_pointer)?;
        writeln!(f, "| enemy table ptr | {:05x} |", room.enemy_table_pointer)?;
        writeln!(
            f,
            "| object table ptr | {:05x} |",
            room.object_table_pointer
        )?;
        writeln!(f)?;

        writeln!(f, "#### Warp Table\n")?;
        write_byte_array(&mut f, &room.warp_table)?;

        writeln!(f, "#### Enemy Table\n")?;
        write_byte_array(&mut f, &room.enemy_table)?;

        writeln!(f, "#### Object Table\n")?;
        write_byte_array(&mut f, &room.object_table)?;
        match parse_object_table(&room.object_table) {
            Ok(table) => {
                for entry in &table {
                    writeln!(f, "- {}", entry)?;
                }
                writeln!(f)?;
            }
            Err(e) => println!(
                "Can't parse object table area {:02x} room {:02x}: {}",
                area_index, room_id, e
            ),
        }
    }

    Ok(())
}

pub(crate) fn command(opt: &DocOpt) -> Result<(), Error> {
    let mut f = File::open(&opt.rom)?;
    let mut buffer = Vec::new();
    // read the whole file
    f.read_to_end(&mut buffer)?;

    let n = Neutopia::new(&buffer)?;

    for area_index in 0..n.area_pointers.len() {
        write_area_markdown(opt, &n, area_index)?;
    }
    Ok(())
}
