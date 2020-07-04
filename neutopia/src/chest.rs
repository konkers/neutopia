use std::io::prelude::*;

use byteorder::WriteBytesExt;
use failure::{format_err, Error};
use nom::{multi::many_m_n, number::complete::le_u8, IResult};

#[derive(Clone, Debug, PartialEq)]
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

    pub fn get_item_name(&self) -> String {
        return match self.item_id {
            0 => format!("{} bombs", self.arg),
            1 => format!("Medicine"),
            2 => format!("Firewand"),
            3 => format!("Sky Bell"),
            4 => format!("Wings"),
            5 => format!("Moonbeam Moss level {}", self.arg),
            6 => format!("Magic Ring"),
            7 => format!("Placeholder"),
            8 => match self.arg {
                1 => format!("Starter Sword"),
                2 => format!("Blue Sword"),
                3 => format!("Purple Sword"),
                4 => format!("Red Sword"),
                _ => format!("Unknown Sword"),
            },
            9 => match self.arg {
                1 => format!("Starter Armor"),
                2 => format!("Blue Armor"),
                3 => format!("Purple Armor"),
                4 => format!("Red Armor"),
                _ => format!("Unknown Armor"),
            },
            10 => match self.arg {
                1 => format!("Wood Shield"),
                2 => format!("Steel Shield"),
                3 => format!("Fire Shield"),
                4 => format!("Blue Shield"),
                _ => format!("Unknown Shield"),
            },
            11 => format!("Falcon Shoes"),
            12 => format!("Rainbow Drop"),
            13 => format!("Book of Revival"),
            14 => format!("Placeholder"),
            15 => format!("Placeholder"),
            16 => format!("Crystal Ball"),
            17 => format!("Crypt Key"),
            18 => format!("Medallion"),
            19 => format!("Medallion"),
            20 => format!("Medallion"),
            21 => format!("Medallion"),
            22 => format!("Medallion"),
            23 => format!("Medallion"),
            24 => format!("Medallion"),
            25 => format!("Medallion"),
            26 => format!("Medallion"),
            _ => format!("Unknown"),
        };
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
