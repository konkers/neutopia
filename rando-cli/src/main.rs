use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

use failure::Error;
use structopt::StructOpt;

use rando::RandoType;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    #[structopt(long, parse(from_os_str), default_value = "Neutopia (USA).pce")]
    rom: PathBuf,

    #[structopt(long, parse(from_os_str))]
    out: Option<PathBuf>,

    #[structopt(long)]
    seed: Option<String>,

    #[structopt(long = "type", default_value = "local")]
    ty: RandoType,
}

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();

    let mut f = File::open(&opt.rom)?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)?;

    let config = rando::Config {
        seed: opt.seed,
        ty: opt.ty,
    };

    println!("{:?}", &config);
    let r = rando::randomize(&config, &buffer)?;

    let filename = &opt
        .out
        .unwrap_or_else(|| PathBuf::from(format!("neutopia-randomizer-{}.pce", r.seed)));

    let mut f = File::create(filename)?;
    f.write_all(&r.data)?;

    println!("wrote {}", filename.display());

    Ok(())
}
