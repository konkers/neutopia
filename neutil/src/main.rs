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
        println!("{:02x}: {:05x} {:05x}", i, area_addr, room_order_addr);
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

    Ok(())
}
