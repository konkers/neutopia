use std::fmt;

use failure::{format_err, Error};
use nom::{
    branch::alt,
    bytes::complete::{tag, take},
    multi::many0,
    number::complete::le_u8,
    IResult,
};

#[derive(Debug, PartialEq)]
pub struct Chest {
    item_id: u8,
    arg: u8,
    text: u8,
    unknown: u8,
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
