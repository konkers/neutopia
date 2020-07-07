use std::collections::HashMap;

use failure::{format_err, Error};
use lazy_static::lazy_static;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Region {
    NA,
    JP,
    Unknown,
}

pub struct RomInfo {
    pub headered: bool,
    pub md5_hash: String,
    pub known: bool,
    pub desc: String,
    pub region: Region,
}

#[derive(Clone, Copy, Debug)]
struct DbEntry {
    desc: &'static str,
    region: Region,
}

impl Default for DbEntry {
    fn default() -> Self {
        Self {
            desc: "Unrecognized ROM",
            region: Region::Unknown,
        }
    }
}

lazy_static! {
    static ref KNOWN_ROMS: HashMap<String, DbEntry> = {
        let mut roms = HashMap::new();
        roms.insert(
            "eb0789088fc70be42b2f994c1b66be21".to_string(),
            DbEntry {
                desc: "Neutopia (U)",
                region: Region::NA,
            },
        );
        roms.insert(
            "08ae173878d8a3783fa35e80c99a5dc4".to_string(),
            DbEntry {
                desc: "Neutopia (J)",
                region: Region::JP,
            },
        );

        roms
    };
}

pub fn verify(data: &[u8]) -> Result<RomInfo, Error> {
    let expected_size = 384 * 1024;
    let header_size = 0x200;

    let (headered, buffer) = if data.len() == expected_size {
        (false, &data as &[u8])
    } else if data.len() == expected_size + header_size {
        (true, &data[header_size..])
    } else {
        return Err(format_err!(
            "Rom size ({}) is neither the expected size of the headered({}) nor the un-headered({}) rom",
            data.len(), expected_size + header_size, expected_size));
    };

    let digest = md5::compute(buffer);
    let md5_hash = format!("{:x}", digest);

    let db_entry = KNOWN_ROMS.get(&md5_hash).map_or(Default::default(), |o| *o);
    let known = KNOWN_ROMS.contains_key(&md5_hash);

    Ok(RomInfo {
        headered,
        md5_hash,
        known,
        desc: db_entry.desc.into(),
        region: db_entry.region,
    })
}
