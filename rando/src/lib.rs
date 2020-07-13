use std::collections::{BTreeMap, BTreeSet};
use std::io::{prelude::*, Cursor, SeekFrom};
use std::str::FromStr;

use failure::{format_err, Error};
use ips::Patch;
use neutopia::{self, rom, verify::Region, Neutopia};
use radix_fmt::radix_36;
use rand::{self, prelude::*};
use rand_core::SeedableRng;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};

mod patches;

static CHECKS_DATA: &[u8] = include_bytes!("checks.json");

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

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum Gate {
    RainbowDrop,
    FalconShoes,
    FireWand,
    Bell,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Check {
    pub name: String,
    pub area: u8,
    pub room: u8,
    #[serde(default)]
    pub index: u8,
    pub gates: Vec<Gate>,
}

impl Check {
    pub fn loc(&self) -> LocationId {
        LocationId {
            area: self.area,
            room: self.room,
            index: self.index,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LocationId {
    pub area: u8,
    pub room: u8,
    pub index: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Item {
    info: rom::Chest,
    area_lock: Option<u8>,
}

struct State {
    // We're using BTree data structures to give us deterministic traversal
    // ordering.
    unassigned_checks: BTreeMap<LocationId, Check>,
    unplaced_items: BTreeSet<Item>,
    cleared_gates: BTreeSet<Gate>,

    assigned_chests: Vec<neutopia::Chest>,

    n: Neutopia,
}

impl State {
    pub fn new(n: Neutopia) -> Result<Self, Error> {
        let mut unplaced_items = BTreeSet::new();

        // Filter out end game area and medallions
        let chests = n.filter_chests(|chest| (chest.area < 0x10) && (chest.info.item_id < 0x12));

        for chest in chests {
            // Lock crystal balls and crypt keys to their area
            let area_lock = match chest.info.item_id {
                0x10 | 0x11 => Some(chest.area),
                _ => None,
            };

            unplaced_items.insert(Item {
                info: chest.info,
                area_lock,
            });
        }

        Ok(Self {
            unassigned_checks: get_checks()?,
            unplaced_items,
            cleared_gates: BTreeSet::new(),
            assigned_chests: Vec::new(),
            n,
        })
    }

    fn is_complete(&self) -> bool {
        assert_eq!(self.unassigned_checks.len(), self.unplaced_items.len());
        self.unassigned_checks.is_empty()
    }

    fn gate_for_item(item: &Item) -> Option<Gate> {
        match item.info.item_id {
            0x02 => Some(Gate::FireWand),
            0x03 => Some(Gate::Bell),
            0x0b => Some(Gate::FalconShoes),
            0x0c => Some(Gate::RainbowDrop),
            _ => None,
        }
    }

    pub fn place_item(&mut self, item: Item, area: u8, room: u8, index: u8) -> Result<(), Error> {
        self.place_item_by_loc(item, &LocationId { area, room, index })
    }

    pub fn place_item_by_loc(&mut self, item: Item, loc: &LocationId) -> Result<(), Error> {
        if let Some(area) = &item.area_lock {
            if *area != loc.area {
                return Err(format_err!(
                    "attempting to place area locked item {:?} in area {}",
                    &item,
                    loc.area
                ));
            }
        }

        let check = self.unassigned_checks.remove(loc).ok_or(format_err!(
            "can't place item at unknown location {:?}",
            loc
        ))?;
        if !self.unplaced_items.remove(&item) {
            return Err(format_err!("can't place unknown item {:?}", item));
        }

        if let Some(gate) = Self::gate_for_item(&item) {
            self.cleared_gates.insert(gate);
        }

        let chest = neutopia::Chest {
            info: item.info,
            area: check.area,
            room: check.room,
            index: check.index,
        };

        self.assigned_chests.push(chest);

        assert_eq!(self.unassigned_checks.len(), self.unplaced_items.len());

        Ok(())
    }

    pub fn filter_items(&self, filter: impl Fn(&Item) -> bool) -> Vec<Item> {
        let mut items = Vec::new();
        for item in &self.unplaced_items {
            if filter(item) {
                items.push(item.clone());
            }
        }

        items
    }

    pub fn get_item_by_id(&self, id: u8) -> Result<Item, Error> {
        let items = self.filter_items(|item| item.info.item_id == id);
        if items.len() > 1 {
            Err(format_err!("Found {} items with id {:02}", items.len(), id))
        } else if items.is_empty() {
            Err(format_err!("Found no items with id {:02}", id))
        } else {
            Ok(items[0].clone())
        }
    }

    pub fn filter_checks(&self, filter: impl Fn(&Check) -> bool) -> Vec<Check> {
        let mut checks = Vec::new();
        'check: for check in self.unassigned_checks.values() {
            // Filter out gated checks first.
            for gate in &check.gates {
                if !self.cleared_gates.contains(gate) {
                    continue 'check;
                }
            }
            if filter(check) {
                checks.push(check.clone());
            }
        }

        checks
    }

    pub fn filter_checks_gateless(&self, filter: impl Fn(&Check) -> bool) -> Vec<Check> {
        let mut checks = Vec::new();
        for check in self.unassigned_checks.values() {
            if filter(check) {
                checks.push(check.clone());
            }
        }

        checks
    }

    pub fn finalize(mut self) -> Result<Neutopia, Error> {
        self.n.update_chests(&self.assigned_chests)?;
        Ok(self.n)
    }
}

fn get_checks() -> Result<BTreeMap<LocationId, Check>, Error> {
    let checks_vec: Vec<Check> = serde_json::from_slice(&CHECKS_DATA)
        .map_err(|e| format_err!("failed to parse checks JSON: {}", e))?;

    let mut checks = BTreeMap::new();
    for check in checks_vec {
        let loc = LocationId {
            area: check.area,
            room: check.room,
            index: check.index,
        };
        if checks.contains_key(&loc) {
            return Err(format_err!(
                "duplicate location {:?} for check {}",
                &loc,
                &check.name
            ));
        }
        checks.insert(loc, check);
    }

    Ok(checks)
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
