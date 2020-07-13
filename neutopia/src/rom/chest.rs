use std::io::prelude::*;

use byteorder::WriteBytesExt;
use failure::{format_err, Error};
use nom::{multi::many_m_n, number::complete::le_u8, IResult};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Chest {
    pub item_id: u8,
    pub arg: u8,
    pub text: u8,
    pub unknown: u8,
}

impl Chest {
    pub fn write(&self, w: &mut impl Write) -> Result<(), Error> {
        w.write_u8(self.item_id)?;
        w.write_u8(self.arg)?;
        w.write_u8(self.text)?;
        w.write_u8(self.unknown)?;

        Ok(())
    }

    #[allow(clippy::useless_format)]
    pub fn get_item_name(&self) -> String {
        match self.item_id {
            0x00 => format!("Bombs x{}", self.arg),
            0x01 => format!("Medicine"),
            0x02 => format!("Fire Wand"),
            0x03 => format!("Sky Bell"),
            0x04 => format!("Wings"),
            0x05 => format!("Moonbeam Moss"),
            0x06 => format!("Magic Ring"),
            0x07 => format!("Placeholder"),
            0x08 => match self.arg {
                1 => format!("Starter Sword"),
                2 => format!("Bronze Sword"),
                3 => format!("Steel Sword"),
                4 => format!("Strongest Sword"),
                _ => format!("Unknown Sword"),
            },
            0x09 => match self.arg {
                1 => format!("Starter Armor"),
                2 => format!("Bronze Armor"),
                3 => format!("Steel Armor"),
                4 => format!("Strongest Armor"),
                _ => format!("Unknown Armor"),
            },
            0x0a => match self.arg {
                1 => format!("Starter Shield"),
                2 => format!("Bronze Shield"),
                3 => format!("Steel Shield"),
                4 => format!("Strongest Shield"),
                _ => format!("Unknown Shield"),
            },
            0x0b => format!("Falcon Shoes"),
            0x0c => format!("Rainbow Drop"),
            0x0d => format!("Book of Revival"),
            0x0e => format!("Placeholder"),
            0x0f => format!("Placeholder"),
            0x10 => format!("Crystal Ball"),
            0x11 => format!("Crypt Key"),
            0x12 => format!("Crypt 1 Medallion"),
            0x13 => format!("Crypt 2 Medallion"),
            0x14 => format!("Crypt 3 Medallion"),
            0x15 => format!("Crypt 4 Medallion"),
            0x16 => format!("Crypt 5 Medallion"),
            0x17 => format!("Crypt 6 Medallion"),
            0x18 => format!("Crypt 7 Medallion"),
            0x19 => format!("Crypt 8 Medallion"),
            0x20 => format!("Placeholder"),
            _ => format!("Unknown"),
        }
    }
}

fn parse_chest(i: &[u8]) -> IResult<&[u8], Chest> {
    let (i, item_id) = le_u8(i)?;
    let (i, arg) = le_u8(i)?;
    let (i, text) = le_u8(i)?;
    let (i, unknown) = le_u8(i)?;

    Ok((
        i,
        Chest {
            item_id,
            arg,
            text,
            unknown,
        },
    ))
}

pub fn parse_chest_table(i: &[u8]) -> Result<Vec<Chest>, Error> {
    let (_, table) =
        many_m_n(8, 8, parse_chest)(i).map_err(|e| format_err!("parse error: {}", e))?;

    Ok(table)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_chest() {
        assert_eq!(
            parse_chest(&[0x11, 0x01, 0x85, 0x41]),
            Ok((
                &[][..],
                Chest {
                    item_id: 0x11,
                    arg: 0x01,
                    text: 0x85,
                    unknown: 0x41,
                }
            ))
        );
    }
}
