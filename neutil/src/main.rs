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

    for (i, addr) in n.area_pointers.iter().enumerate() {
        println!("{}: 0x{:06x}", i, addr);
    }

    Ok(())
}
