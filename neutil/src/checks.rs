use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

use failure::Error;
use structopt::StructOpt;

use neutopia::Neutopia;
use rando::Check;

#[derive(StructOpt, Debug)]
pub(crate) struct ChecksOpt {
    #[structopt(long, parse(from_os_str), default_value = "Neutopia (USA).pce")]
    rom: PathBuf,

    #[structopt(long, parse(from_os_str), default_value = "out")]
    out: PathBuf,
}

pub(crate) fn command(opt: &ChecksOpt) -> Result<(), Error> {
    let mut f = File::open(&opt.rom)?;
    let mut data = Vec::new();
    // read the whole file
    f.read_to_end(&mut data)?;

    let n = Neutopia::new(&data)?;

    let chests = n.filter_chests(|chest| {
        // All areas that are non the end game area.
        (chest.area < 0x10)
                // Chest does not contain medallion
                && (chest.info.item_id < 0x12 || chest.info.item_id >= (0x12 + 8))
    });

    let mut checks = Vec::new();
    for chest in &chests {
        let name = format!(
            "{} - {}",
            neutopia::area_name(chest.area),
            chest.info.get_item_name(),
        );
        let check = Check {
            name,
            area: chest.area,
            room: chest.room,
            index: chest.index,
            gates: Vec::new(),
        };
        checks.push(check);
    }

    let f = File::create(&opt.out)?;
    serde_json::to_writer_pretty(f, &checks)?;
    Ok(())
}
