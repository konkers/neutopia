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
            0 => format!("Bombs x{}", self.arg),
            1 => format!("Medicine"),
            2 => format!("Fire Wand"),
            3 => format!("Sky Bell"),
            4 => format!("Wings"),
            5 => format!("Moonbeam Moss"),
            6 => format!("Magic Ring"),
            7 => format!("Placeholder"),
            8 => match self.arg {
                1 => format!("Starter Sword"),
                2 => format!("Bronze Sword"),
                3 => format!("Steel Sword"),
                4 => format!("Strongest Sword"),
                _ => format!("Unknown Sword"),
            },
            9 => match self.arg {
                1 => format!("Starter Armor"),
                2 => format!("Bronze Armor"),
                3 => format!("Steel Armor"),
                4 => format!("Strongest Armor"),
                _ => format!("Unknown Armor"),
            },
            10 => match self.arg {
                1 => format!("Starter Shield"),
                2 => format!("Bronze Shield"),
                3 => format!("Steel Shield"),
                4 => format!("Strongest Shield"),
                _ => format!("Unknown Shield"),
            },
            11 => format!("Falcon Shoes"),
            12 => format!("Rainbow Drop"),
            13 => format!("Book of Revival"),
            14 => format!("Placeholder"),
            15 => format!("Placeholder"),
            16 => format!("Crystal Ball"),
            17 => format!("Crypt Key"),
            18 => format!("Crypt 1 Medallion"),
            19 => format!("Crypt 2 Medallion"),
            20 => format!("Crypt 3 Medallion"),
            21 => format!("Crypt 4 Medallion"),
            22 => format!("Crypt 5 Medallion"),
            23 => format!("Crypt 6 Medallion"),
            24 => format!("Crypt 7 Medallion"),
            25 => format!("Crypt 8 Medallion"),
            26 => format!("Placeholder"),
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
