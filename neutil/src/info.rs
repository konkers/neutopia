use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

use failure::Error;
use structopt::StructOpt;

use neutopia::verify;

#[derive(StructOpt, Debug)]
pub(crate) struct InfoOpt {
    #[structopt(long, parse(from_os_str), default_value = "neutopia-jp.pce")]
    rom: PathBuf,
}

pub(crate) fn command(opt: &InfoOpt) -> Result<(), Error> {
    let mut f = File::open(&opt.rom)?;
    let mut buffer = Vec::new();
    // read the whole file
    f.read_to_end(&mut buffer)?;

    let info = verify(&buffer)?;

    println!("Info for {}:", &opt.rom.display());
    println!("  Headered:    {}", info.headered);
    println!("  MD5 hash:    {}", info.md5_hash);
    println!("  Description: {}", info.desc);
    println!("  Region:      {:?}", info.region);

    Ok(())
}
