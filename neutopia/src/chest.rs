use failure::{format_err, Error};
use nom::{multi::many_m_n, number::complete::le_u8, IResult};

#[derive(Debug, PartialEq)]
pub struct Chest {
    pub item_id: u8,
    pub arg: u8,
    pub text: u8,
    pub unknown: u8,
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
