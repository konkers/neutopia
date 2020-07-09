use std::fs::File;
use std::io::{prelude::*, Cursor, SeekFrom};
use std::path::PathBuf;

use failure::{format_err, Error};
use ips::Patch;
use radix_fmt::radix_36;
use rand::{self, prelude::*};
use rand_core::SeedableRng;
use rand_pcg::Pcg32;
use structopt::{clap::arg_enum, StructOpt};

use neutopia::{self, rom, verify::Region, Neutopia};

mod patches;

arg_enum! {
    #[derive(Debug)]
    enum RandoType {
        Local,
        Global,
        None,
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    #[structopt(long, parse(from_os_str), default_value = "Neutopia (USA).pce")]
    rom: PathBuf,

    #[structopt(long, parse(from_os_str))]
    out: Option<PathBuf>,

    #[structopt(long)]
    seed: Option<String>,

    #[structopt(long = "type", possible_values = &RandoType::variants(), case_insensitive = true, default_value = "local")]
    ty: RandoType,
}

// Shuffle all items within each crypt.  Does not touch overworld items.
fn crypt_rando(rng: &mut impl Rng, rom_data: &[u8]) -> Result<Vec<u8>, Error> {
    let mut n = Neutopia::new(rom_data)?;

    for area_idx in 0x4..=0xb {
        // Find all the chest we want to randomize.
        let mut chests = n.filter_chests(|chest| {
            // Chest is in current area
            (chest.area == area_idx)
                // Chest does not contain medallion
                && (chest.info.item_id < 0x12 || chest.info.item_id >= (0x12 + 8))
        });

        // Shuffle the chests.
        let mut randomized_chests: Vec<rom::Chest> =
            chests.iter().map(|chest| chest.info.clone()).collect();
        randomized_chests.shuffle(rng);

        // Update the chests' info
        for (i, chest) in chests.iter_mut().enumerate() {
            chest.info = randomized_chests[i].clone();
        }

        n.update_chests(&chests)?;
    }

    n.write()
}

// Shuffle all items across crypts and overworld.  Does not contain logic
// to make sure seed is completable.
fn global_rando(rng: &mut impl Rng, rom_data: &[u8]) -> Result<Vec<u8>, Error> {
    let mut n = Neutopia::new(rom_data)?;

    let mut chests = n.filter_chests(|chest| {
        // Chest is in current area
        (chest.area < 0x10)
                // Chest does not contain medallion, crystal ball, or key
                && (chest.info.item_id < 0x10 || chest.info.item_id >= (0x12 + 8))
    });

    // Shuffle the chests.
    let mut randomized_chests: Vec<rom::Chest> =
        chests.iter().map(|chest| chest.info.clone()).collect();
    randomized_chests.shuffle(rng);

    // Update the chests' info
    for (i, chest) in chests.iter_mut().enumerate() {
        chest.info = randomized_chests[i].clone();
    }

    n.update_chests(&chests)?;
    n.write()
}

fn verify_rom(data: Vec<u8>) -> Result<Vec<u8>, Error> {
    // Verify
    let info = neutopia::verify(&data)?;
    if !info.known {
        return Err(format_err!(
            "Rom with MD5 hash {} is unrecognized.",
            &info.md5_hash
        ));
    }
    if info.region != Region::NA {
        return Err(format_err!(
            "Region {:?} rom not supported.  Please use NA rom.",
            &info.region
        ));
    }

    if info.headered {
        Ok(data[0x200..].to_vec())
    } else {
        Ok(data)
    }
}

fn apply_patch<W: Write + Seek>(w: &mut W, patch_data: &[u8]) -> Result<(), Error> {
    let patch = Patch::parse(patch_data)?;

    for hunk in patch.hunks() {
        w.seek(SeekFrom::Start(hunk.offset() as u64))?;
        w.write_all(hunk.payload())?;
    }

    Ok(())
}

fn apply_patches(data: &mut [u8]) -> Result<(), Error> {
    let mut c = Cursor::new(data);
    for patch in patches::PATCHES.iter() {
        apply_patch(&mut c, patch)?;
    }
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

    let mut buffer = verify_rom(buffer)?;

    apply_patches(&mut buffer)?;

    let new_data = match opt.ty {
        RandoType::Local => crypt_rando(&mut rng, &buffer)?,
        RandoType::Global => global_rando(&mut rng, &buffer)?,
        _ => buffer,
    };

    let filename = &opt
        .out
        .unwrap_or_else(|| PathBuf::from(format!("neutopia-randomizer-{:#}.pce", radix_36(seed))));
    let mut f = File::create(filename)?;
    f.write_all(&new_data)?;

    println!("wrote {}", filename.display());
    Ok(())
}
