use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

use failure::{format_err, Error};
use radix_fmt::radix_36;
use rand::{self, prelude::*};
use rand_core::SeedableRng;
use rand_pcg::Pcg32;
use structopt::StructOpt;

use randolib::randomize_rom;
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

    randomize_rom(&mut buffer, &mut rng)?;

    let filename = &opt
        .out
        .unwrap_or_else(|| PathBuf::from(format!("neutopia-randomizer-{:#}.pce", radix_36(seed))));
    let mut f = File::create(filename)?;
    f.write_all(&buffer)?;

    println!("wrote {}", filename.display());
    Ok(())
}
