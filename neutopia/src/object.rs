use std::fmt;

use failure::{format_err, Error};
use nom::{
    branch::alt,
    bytes::complete::{tag, take},
    multi::many0,
    IResult,
};

#[derive(Debug, PartialEq)]
pub struct ObjectInfo {
    x: u8,
    y: u8,
    id: u8,
}

impl fmt::Display for ObjectInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x{:02x} @ ({},{})", self.id, self.x, self.y)
    }
}

#[derive(Debug, PartialEq)]
pub enum TableEntry {
    Object(ObjectInfo),
    OpenDoor(u8),
    PushBlockGatedDoor(u8),
    EnemyGatedDoor(u8),
    BombableDoor(u8),
    PushBlockGatedObject(ObjectInfo),
    EnemyGatedObject(ObjectInfo),
    BellGatedObject(ObjectInfo),
    DarkRoom,
    BossDoor(u8),
    Unknown0b([u8; 3]),
    Burnable(ObjectInfo),
    HiddenRoom([u8; 2]),
    FalconBootsNeeded,
    Npc([u8; 5]),
    OuchRope(ObjectInfo),
    ArrowLauncher(ObjectInfo),
    Swords(ObjectInfo),
    GhostSpawner(ObjectInfo),
    FireballSpawner(ObjectInfo),
    ShopItem([u8; 7]),
    UnknownE1([u8; 9]),
    UnknownF4([u8; 5]),
}

impl fmt::Display for TableEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Object(info) => write!(f, "object {}", info),
            Self::OpenDoor(data) => write!(f, "open door 0x{:02x}", data),
            Self::PushBlockGatedDoor(data) => write!(f, "push block gated door 0x{:02x}", data),
            Self::EnemyGatedDoor(data) => write!(f, "enemy gated door 0x{:02x}", data),
            Self::BombableDoor(data) => write!(f, "bombable door 0x{:02x}", data),
            Self::PushBlockGatedObject(info) => write!(f, "push block gated object {}", info),
            Self::EnemyGatedObject(info) => write!(f, "enemy gated object {}", info),
            Self::BellGatedObject(info) => write!(f, "bell gated object {}", info),
            Self::DarkRoom => write!(f, "dark room"),
            Self::BossDoor(data) => write!(f, "boss door 0x{:02x}", data),
            Self::Unknown0b(data) => write!(f, "unknown object 0x0b {:x?}", data),
            Self::Burnable(info) => write!(f, "burnable {}", info),
            Self::HiddenRoom(data) => write!(f, "hidden room {:x?}", data),
            Self::FalconBootsNeeded => write!(f, "falcon boots needed"),
            Self::Npc(data) => write!(f, "npc {:x?}", data),
            Self::OuchRope(info) => write!(f, "ouch rope segment {}", info),
            Self::ArrowLauncher(info) => write!(f, "arrow launcher {}", info),
            Self::Swords(info) => write!(f, "swords {}", info),
            Self::GhostSpawner(info) => write!(f, "ghost spawner {}", info),
            Self::FireballSpawner(info) => write!(f, "fireball spawner {}", info),
            Self::ShopItem(data) => write!(f, "shop item {:x?}", data),
            Self::UnknownE1(data) => write!(f, "unknown object 0xe1 {:x?}", data),
            Self::UnknownF4(data) => write!(f, "unknown object 0xf4 {:x?}", data),
        }
    }
}

fn parse_object_info(i: &[u8]) -> IResult<&[u8], ObjectInfo> {
    let (i, loc) = take(1usize)(i)?;
    let (i, id) = take(1usize)(i)?;
    let x = loc[0] & 0xf;
    let y = loc[0] >> 4;

    Ok((i, ObjectInfo { x, y, id: id[0] }))
}

macro_rules! gen_object_type {
    ($func_name: ident, $tag: literal, $ty: ident) => {
        fn $func_name(i: &[u8]) -> IResult<&[u8], TableEntry> {
            let (i, _) = tag([$tag])(i)?;
            let (i, info) = parse_object_info(i)?;

            Ok((i, TableEntry::$ty(info)))
        }
    };
}

macro_rules! gen_u8_type {
    ($func_name: ident, $tag: literal, $ty: ident) => {
        fn $func_name(i: &[u8]) -> IResult<&[u8], TableEntry> {
            let (i, _) = tag([$tag])(i)?;
            let (i, data) = take(1usize)(i)?;

            Ok((i, TableEntry::$ty(data[0])))
        }
    };
}

macro_rules! gen_simple_type {
    ($func_name: ident, $tag: literal, $ty: ident) => {
        fn $func_name(i: &[u8]) -> IResult<&[u8], TableEntry> {
            let (i, _) = tag([$tag])(i)?;

            Ok((i, TableEntry::$ty))
        }
    };
}

gen_object_type!(parse_object, 0x00, Object);
gen_u8_type!(parse_open_door, 0x01, OpenDoor);
gen_u8_type!(parse_push_block_gated_door, 0x02, PushBlockGatedDoor);
gen_u8_type!(parse_enemy_gated_door, 0x03, EnemyGatedDoor);
gen_u8_type!(parse_bombable_door, 0x05, BombableDoor);
gen_object_type!(parse_push_block_gated_object, 0x06, PushBlockGatedObject);
gen_object_type!(parse_enemy_gated_object, 0x07, EnemyGatedObject);
gen_object_type!(parse_bell_gated_object, 0x08, BellGatedObject);
gen_simple_type!(parse_dark_room, 0x09, DarkRoom);
gen_u8_type!(parse_boss_door, 0x0a, BossDoor);

fn parse_unknown_0b(i: &[u8]) -> IResult<&[u8], TableEntry> {
    let (i, _) = tag([0x0b])(i)?;
    let (i, data) = take(3usize)(i)?;
    Ok((i, TableEntry::Unknown0b([data[0], data[1], data[2]])))
}

gen_object_type!(parse_burnable, 0x0c, Burnable);

fn parse_hidden_room(i: &[u8]) -> IResult<&[u8], TableEntry> {
    let (i, _) = tag([0x0d])(i)?;
    let (i, data) = take(2usize)(i)?;
    Ok((i, TableEntry::HiddenRoom([data[0], data[1]])))
}

gen_simple_type!(parse_falcon_boots_needed, 0x81, FalconBootsNeeded);

fn parse_npc(i: &[u8]) -> IResult<&[u8], TableEntry> {
    let (i, _) = tag([0x9a])(i)?;
    let (i, data) = take(5usize)(i)?;
    Ok((
        i,
        TableEntry::Npc([data[0], data[1], data[2], data[3], data[4]]),
    ))
}

gen_object_type!(parse_ouch_rope, 0xbd, OuchRope);
gen_object_type!(parse_arrow_launcher, 0xbf, ArrowLauncher);
gen_object_type!(parse_swords, 0xc0, Swords);
gen_object_type!(parse_ghost_spawner, 0xc1, GhostSpawner);
gen_object_type!(parse_fireball_spawner, 0xc6, FireballSpawner);

fn parse_shop_item(i: &[u8]) -> IResult<&[u8], TableEntry> {
    let (i, _) = tag([0xda])(i)?;
    let (i, data) = take(7usize)(i)?;
    Ok((
        i,
        TableEntry::ShopItem([
            data[0], data[1], data[2], data[3], data[4], data[5], data[6],
        ]),
    ))
}

fn parse_unknown_e1(i: &[u8]) -> IResult<&[u8], TableEntry> {
    let (i, _) = tag([0xe1])(i)?;
    let (i, data) = take(9usize)(i)?;
    Ok((
        i,
        TableEntry::UnknownE1([
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7], data[8],
        ]),
    ))
}

fn parse_unknown_f4(i: &[u8]) -> IResult<&[u8], TableEntry> {
    let (i, _) = tag([0xf4])(i)?;
    let (i, data) = take(5usize)(i)?;
    Ok((
        i,
        TableEntry::UnknownF4([data[0], data[1], data[2], data[3], data[4]]),
    ))
}

fn parse_object_table_entry(i: &[u8]) -> IResult<&[u8], TableEntry> {
    // There seems to be a limit on the size of tuples in for alt so we
    // split it.
    alt((
        alt((
            parse_object,
            parse_open_door,
            parse_push_block_gated_door,
            parse_enemy_gated_door,
            parse_bombable_door,
            parse_push_block_gated_object,
            parse_enemy_gated_object,
            parse_bell_gated_object,
            parse_dark_room,
            parse_unknown_0b,
            parse_burnable,
            parse_hidden_room,
            parse_falcon_boots_needed,
            parse_npc,
            parse_boss_door,
        )),
        alt((
            parse_ouch_rope,
            parse_arrow_launcher,
            parse_swords,
            parse_ghost_spawner,
            parse_fireball_spawner,
            parse_shop_item,
            parse_unknown_e1,
            parse_unknown_f4,
        )),
    ))(i)
}

pub fn object_table_len(data: &[u8]) -> Result<usize, Error> {
    let (i, _) =
        many0(parse_object_table_entry)(data).map_err(|e| format_err!("parse error: {}", e))?;

    if i.len() > 0 && i[0] != 0xff {
        return Err(format_err!("unparsed input: {:x?}", i));
    }

    Ok(data.len() - i.len())
}

pub fn parse_object_table(data: &[u8]) -> Result<Vec<TableEntry>, Error> {
    let (i, table) =
        many0(parse_object_table_entry)(data).map_err(|e| format_err!("parse error: {}", e))?;

    if i.len() > 0 {
        return Err(format_err!("unparsed input: {:x?}", i));
    }

    Ok(table)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_entries() {
        assert_eq!(
            parse_object_table_entry(&[0x00, 0x52, 0xa5]),
            Ok((
                &[][..],
                TableEntry::Object(ObjectInfo {
                    x: 2,
                    y: 5,
                    id: 0xa5
                })
            ))
        );

        assert_eq!(
            parse_object_table_entry(&[0x01, 0x02]),
            Ok((&[][..], TableEntry::OpenDoor(0x02)))
        );

        assert_eq!(
            parse_object_table_entry(&[0x02, 0x01]),
            Ok((&[][..], TableEntry::PushBlockGatedDoor(0x01)))
        );

        assert_eq!(
            parse_object_table_entry(&[0x03, 0x08]),
            Ok((&[][..], TableEntry::EnemyGatedDoor(0x08)))
        );

        assert_eq!(
            parse_object_table_entry(&[0x05, 0x0a]),
            Ok((&[][..], TableEntry::BombableDoor(0x0a)))
        );

        assert_eq!(
            parse_object_table_entry(&[0x06, 0x25, 0x5a]),
            Ok((
                &[][..],
                TableEntry::PushBlockGatedObject(ObjectInfo {
                    x: 5,
                    y: 2,
                    id: 0x5a
                })
            ))
        );

        assert_eq!(
            parse_object_table_entry(&[0x07, 0x25, 0x5a]),
            Ok((
                &[][..],
                TableEntry::EnemyGatedObject(ObjectInfo {
                    x: 5,
                    y: 2,
                    id: 0x5a
                })
            ))
        );

        assert_eq!(
            parse_object_table_entry(&[0x08, 0x25, 0x5a]),
            Ok((
                &[][..],
                TableEntry::BellGatedObject(ObjectInfo {
                    x: 5,
                    y: 2,
                    id: 0x5a
                })
            ))
        );

        assert_eq!(
            parse_object_table_entry(&[0x09]),
            Ok((&[][..], TableEntry::DarkRoom))
        );

        assert_eq!(
            parse_object_table_entry(&[0x0a, 0x50]),
            Ok((&[][..], TableEntry::BossDoor(0x50)))
        );

        assert_eq!(
            parse_object_table_entry(&[0x0b, 0x46, 0x2a, 0x04]),
            Ok((&[][..], TableEntry::Unknown0b([0x46, 0x2a, 0x04])))
        );

        assert_eq!(
            parse_object_table_entry(&[0x0c, 0x52, 0xa5]),
            Ok((
                &[][..],
                TableEntry::Burnable(ObjectInfo {
                    x: 2,
                    y: 5,
                    id: 0xa5
                })
            ))
        );

        assert_eq!(
            parse_object_table_entry(&[0x0d, 0x14, 0x14]),
            Ok((&[][..], TableEntry::HiddenRoom([0x14, 0x14])))
        );

        assert_eq!(
            parse_object_table_entry(&[0x81]),
            Ok((&[][..], TableEntry::FalconBootsNeeded))
        );
        assert_eq!(
            parse_object_table_entry(&[0x9a, 0x48, 0x02, 0x03, 0x00, 0x40]),
            Ok((&[][..], TableEntry::Npc([0x48, 0x02, 0x03, 0x00, 0x40])))
        );

        assert_eq!(
            parse_object_table_entry(&[0xbd, 0x25, 0x5a]),
            Ok((
                &[][..],
                TableEntry::OuchRope(ObjectInfo {
                    x: 5,
                    y: 2,
                    id: 0x5a
                })
            ))
        );

        assert_eq!(
            parse_object_table_entry(&[0xbf, 0x25, 0x5a]),
            Ok((
                &[][..],
                TableEntry::ArrowLauncher(ObjectInfo {
                    x: 5,
                    y: 2,
                    id: 0x5a
                })
            ))
        );

        assert_eq!(
            parse_object_table_entry(&[0xc0, 0x25, 0x5a]),
            Ok((
                &[][..],
                TableEntry::Swords(ObjectInfo {
                    x: 5,
                    y: 2,
                    id: 0x5a
                })
            ))
        );

        assert_eq!(
            parse_object_table_entry(&[0xc1, 0x25, 0x5a]),
            Ok((
                &[][..],
                TableEntry::GhostSpawner(ObjectInfo {
                    x: 5,
                    y: 2,
                    id: 0x5a
                })
            ))
        );

        assert_eq!(
            parse_object_table_entry(&[0xc6, 0x25, 0x5a]),
            Ok((
                &[][..],
                TableEntry::FireballSpawner(ObjectInfo {
                    x: 5,
                    y: 2,
                    id: 0x5a
                })
            ))
        );

        assert_eq!(
            parse_object_table_entry(&[0xda, 0x46, 0x00, 0x00, 0x02, 0x00, 0x01, 0x01]),
            Ok((
                &[][..],
                TableEntry::ShopItem([0x46, 0x00, 0x00, 0x02, 0x00, 0x01, 0x01]),
            ))
        );

        assert_eq!(
            parse_object_table_entry(&[0xe1, 0x48, 0x02, 0x00, 0x7d, 0x41, 0x56, 0x2e, 0x81, 0x01]),
            Ok((
                &[][..],
                TableEntry::UnknownE1([0x48, 0x02, 0x00, 0x7d, 0x41, 0x56, 0x2e, 0x81, 0x01])
            ))
        );

        assert_eq!(
            parse_object_table_entry(&[0xf4, 0xa7, 0x02, 0x03, 0x40, 0x43]),
            Ok((
                &[][..],
                TableEntry::UnknownF4([0xa7, 0x02, 0x03, 0x40, 0x43]),
            ))
        );

        assert_eq!(
            parse_object_table(&[0x01, 0x02, 0x02, 0x01]).unwrap(),
            vec![
                TableEntry::OpenDoor(0x02),
                TableEntry::PushBlockGatedDoor(0x01)
            ]
        );
    }
}
