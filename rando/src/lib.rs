use std::io::{prelude::*, Cursor, SeekFrom};
use std::str::FromStr;

use failure::{format_err, Error};
use ips::Patch;
use neutopia::{self, rom, verify::Region, Neutopia};
use radix_fmt::radix_36;
use rand::{self, prelude::*};
use rand_core::SeedableRng;
use rand_pcg::Pcg32;

mod patches;
mod state;

pub use state::Check;
use state::State;

#[derive(Debug)]
pub enum RandoType {
    Local,
    Global,
    None,
}

impl FromStr for RandoType {
    type Err = Error;
    fn from_str(day: &str) -> Result<Self, Self::Err> {
        match day {
            "local" => Ok(RandoType::Local),
            "global" => Ok(RandoType::Global),
            "none" => Ok(RandoType::None),
            _ => Err(format_err!("Could not parse rando type")),
        }
    }
}

#[derive(Debug)]
pub struct Config {
    pub ty: RandoType,
    pub seed: Option<String>,
}

pub struct RandomizedGame {
    pub seed: String,
    pub data: Vec<u8>,
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
    let n = Neutopia::new(rom_data)?;

    let mut state = State::new(n)?;
    let book = state.get_item_by_id(0xd)?;
    let moss = state.get_item_by_id(0x5)?;

    state.place_item(book, 0xc, 0x9, 0x0)?;
    state.place_item(moss, 0xc, 0x11, 0x1)?;

    // Place area locked items first.
    for area in 0x4..=0xf {
        let items = state.filter_items(|item| match item.area_lock {
            Some(a) => a == area,
            None => false,
        });

        for item in items {
            // Query checks each iteration so that we pick up changes we make.
            // Also, ignore key item gating as we know the area locked items
            // are not affected by gating.
            let checks = state.filter_checks_gateless(|check| check.area == area);
            let check = checks.choose(rng).unwrap();
            state.place_item_by_loc(item, &check.loc())?;
        }
    }

    // Next place the fire wand, bell, shoes, and drop in logic
    let mut items = state.filter_items(|item| {
        item.info.item_id == 0x2
            || item.info.item_id == 0x3
            || item.info.item_id == 0xb
            || item.info.item_id == 0xc
    });
    items.shuffle(rng);
    while !items.is_empty() {
        // Get all open checks and chose one
        let checks = state.filter_checks(|_| true);
        let check = checks.choose(rng).unwrap();
        let item = items.pop().unwrap();
        state.place_item_by_loc(item, &check.loc())?;
    }

    //
    // Now assign the rest of the items considering gating.
    //

    // Get all the items and shuffle them.
    let mut items = state.filter_items(|_| true);
    items.shuffle(rng);
    while !state.is_complete() {
        // Get all open checks and chose one
        let checks = state.filter_checks(|_| true);
        let check = checks.choose(rng).unwrap();
        let item = items.pop().unwrap();
        state.place_item_by_loc(item, &check.loc())?;
    }
    let n = state.finalize()?;
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

pub fn randomize(config: &Config, data: &[u8]) -> Result<RandomizedGame, Error> {
    // Let the user specify a seed in base36.  Otherwise randomly generate one.
    let seed = match &config.seed {
        Some(s) => u64::from_str_radix(s, 36)
            .map_err(|e| format_err!("Seed name must be a valid base36 64 bit number: {}", e))?,
        None => rand::thread_rng().gen(),
    };

    let mut rng = Pcg32::seed_from_u64(seed);

    let mut buffer = verify_rom(data.to_vec())?;

    apply_patches(&mut buffer)?;

    let new_data = match config.ty {
        RandoType::Local => crypt_rando(&mut rng, &buffer)?,
        RandoType::Global => global_rando(&mut rng, &buffer)?,
        _ => buffer,
    };

    Ok(RandomizedGame {
        seed: format!("{:#}", radix_36(seed)),
        data: new_data,
    })
}
