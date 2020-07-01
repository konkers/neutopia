use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

use failure::Error;
use structopt::StructOpt;

use neutopia::Neutopia;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    #[structopt(long, parse(from_os_str), default_value = "neutopia-jp.pce")]
    rom: PathBuf,
}

fn write_byte_array(f: &mut File, data: &Vec<u8>) -> Result<(), Error> {
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

fn write_area_markdown(n: &Neutopia, area_index: usize) -> Result<(), Error> {
    let path: PathBuf = ["out", &format!("area_{:02x}.md", area_index)]
        .iter()
        .collect();
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
    write!(f, "\n")?;

    let addr = n.room_order_pointers[area_index];
    let table = &n.room_order_tables[&addr];
    writeln!(f, "## Room Map/Order\n")?;
    writeln!(f, "```")?;
    writeln!(f, "     0  1  2  3  4  5  6  7")?;
    write!(f, "  +------------------------")?;
    for (i, room_id) in table.iter().enumerate() {
        if i % 8 == 0x00 {
            write!(f, "\n{} |", &i / 8)?;
        }

        write!(f, " {:02x}", room_id)?;
    }
    writeln!(f, "\n```")?;

    writeln!(f, "## Rooms\n")?;
    let rooms = &n.room_info_tables[area_index];
    let mut room_ids: Vec<u8> = rooms.keys().map(|x| *x).collect();
    room_ids.sort();

    for room_id in room_ids {
        let room = &rooms[&room_id];
        writeln!(
            f,
            "### Room {:02X} ({}, {}) @ {:05x}\n",
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
        write!(f, "\n")?;

        writeln!(f, "#### Warp Table\n")?;
        write_byte_array(&mut f, &room.warp_table)?;

        writeln!(f, "#### Enemy Table\n")?;
        write_byte_array(&mut f, &room.enemy_table)?;

        writeln!(f, "#### Object Table\n")?;
        write_byte_array(&mut f, &room.object_table)?;
    }

    Ok(())
}

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();
    let mut f = File::open(opt.rom)?;
    let mut buffer = Vec::new();
    // read the whole file
    f.read_to_end(&mut buffer)?;

    let n = Neutopia::new(&buffer)?;

    println!("Area data:");
    for (i, (area_addr, room_order_addr)) in n
        .area_pointers
        .iter()
        .zip(n.room_order_pointers.iter())
        .enumerate()
    {
        println!("{:02x}: {} {}", i, area_addr, room_order_addr);
    }

    println!("\nRoom order tables:");
    for (ptr, table) in &n.room_order_tables {
        print!("{:05x}:", ptr);
        for (i, room_id) in table.iter().enumerate() {
            if i > 0 && i % 0x10 == 0x00 {
                print!("\n      ");
            }
            print!(" {:02x}", room_id);
        }
        print!("\n");
    }

    for area_index in 0..n.area_pointers.len() {
        write_area_markdown(&n, area_index)?;
    }
    Ok(())
}
