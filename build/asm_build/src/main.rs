use std::num::ParseIntError;
use std::path::PathBuf;

use failure::Error;
use parse_int::parse;
use structopt::StructOpt;

fn parse_num(src: &str) -> Result<u32, ParseIntError> {
    parse::<u32>(src)
}

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(long, parse(from_os_str), default_value = "patch.ips")]
    out: PathBuf,

    #[structopt(long, parse(try_from_str = parse_num))]
    rom_size: u32,

    #[structopt(long, parse(from_os_str), default_value = "bass")]
    bass: PathBuf,

    #[structopt(long, parse(from_os_str), default_value = ".")]
    tmp_dir: PathBuf,

    #[structopt(parse(from_os_str))]
    src_files: Vec<PathBuf>,
}

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();

    asm_build::build(
        &opt.bass,
        opt.rom_size,
        &opt.tmp_dir,
        &opt.src_files,
        &opt.out,
    )?;

    Ok(())
}
