use std::collections::{BTreeMap, BTreeSet};

use failure::{format_err, Error};
use neutopia::{self, rom, Neutopia};
use serde::{Deserialize, Serialize};

static CHECKS_DATA: &[u8] = include_bytes!("checks.json");

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
    pub(crate) fn loc(&self) -> LocationId {
        LocationId {
            area: self.area,
            room: self.room,
            index: self.index,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct LocationId {
    pub area: u8,
    pub room: u8,
    pub index: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Item {
    pub info: rom::Chest,
    pub area_lock: Option<u8>,
}

pub(crate) struct State {
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

    pub fn is_complete(&self) -> bool {
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

        let check = self
            .unassigned_checks
            .remove(loc)
            .ok_or_else(|| format_err!("can't place item at unknown location {:?}", loc))?;
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
