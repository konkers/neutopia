use std::fmt;
use std::io::prelude::*;

use byteorder::WriteBytesExt;
use failure::{format_err, Error};
use nom::{
    branch::alt,
    bytes::complete::{tag, take},
    multi::many0,
    IResult,
};

#[derive(Clone, Debug, PartialEq)]
pub struct ObjectInfo {
    pub x: u8,
    pub y: u8,
    pub id: u8,
}

impl fmt::Display for ObjectInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x{:02x} @ ({},{})", self.id, self.x, self.y)
    }
}

#[derive(Clone, Debug, PartialEq)]
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
    HiddenRoom([u8; 3]),
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

impl TableEntry {
    pub fn write(&self, w: &mut impl Write) -> Result<(), Error> {
        match self {
            Self::Object(_) => write_object(w, self)?,
            Self::OpenDoor(_) => write_open_door(w, self)?,
            Self::PushBlockGatedDoor(_) => write_push_block_gated_door(w, self)?,
            Self::EnemyGatedDoor(_) => write_enemy_gated_door(w, self)?,
            Self::BombableDoor(_) => write_bombable_door(w, self)?,
            Self::PushBlockGatedObject(_) => write_push_block_gated_object(w, self)?,
            Self::EnemyGatedObject(_) => write_enemy_gated_object(w, self)?,
            Self::BellGatedObject(_) => write_bell_gated_object(w, self)?,
            Self::DarkRoom => write_dark_room(w, self)?,
            Self::BossDoor(_) => write_boss_door(w, self)?,
            Self::Unknown0b(_) => write_unknown_0b(w, self)?,
            Self::Burnable(_) => write_burnable(w, self)?,
            Self::HiddenRoom(_) => write_hidden_room(w, self)?,
            Self::FalconBootsNeeded => write_falcon_boots_needed(w, self)?,
            Self::Npc(_) => write_npc(w, self)?,
            Self::OuchRope(_) => write_ouch_rope(w, self)?,
            Self::ArrowLauncher(_) => write_arrow_launcher(w, self)?,
            Self::Swords(_) => write_swords(w, self)?,
            Self::GhostSpawner(_) => write_ghost_spawner(w, self)?,
            Self::FireballSpawner(_) => write_fireball_spawner(w, self)?,
            Self::ShopItem(_) => write_shop_item(w, self)?,
            Self::UnknownE1(_) => write_unknown_e1(w, self)?,
            Self::UnknownF4(_) => write_unknown_f4(w, self)?,
        }
        Ok(())
    }

    pub fn chest_id(&self) -> Option<u8> {
        if let Self::Object(o) = self {
            if 0x4c <= o.id && o.id <= (0x4c + 8) {
                return Some(o.id - 0x4c);
            }
        }
        None
    }

    pub fn is_conditional(&self) -> bool {
        match self {
            Self::Unknown0b(_) => true,
            _ => false,
        }
    }

    pub fn loc(&self) -> Option<(u8, u8)> {
        match self {
            Self::Object(o) => Some((o.x, o.y)),
            _ => None,
        }
    }
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

impl ObjectInfo {
    fn write(&self, w: &mut impl Write) -> Result<(), Error> {
        let loc = (self.x & 0xf) | ((self.y & 0xf) << 4);
        w.write_u8(loc)?;
        w.write_u8(self.id)?;

        Ok(())
    }
}
macro_rules! gen_object_type {
    ($parse_func_name: ident, $write_func_name: ident, $tag: literal, $ty: ident) => {
        fn $parse_func_name(i: &[u8]) -> IResult<&[u8], TableEntry> {
            let (i, _) = tag([$tag])(i)?;
            let (i, info) = parse_object_info(i)?;

            Ok((i, TableEntry::$ty(info)))
        }

        fn $write_func_name(w: &mut impl Write, o: &TableEntry) -> Result<(), Error> {
            w.write_u8($tag)?;
            if let TableEntry::$ty(info) = o {
                info.write(w)?;
            } else {
                panic!("internal error");
            }

            Ok(())
        }
    };
}

macro_rules! gen_u8_type {
    ($parse_func_name: ident, $write_func_name: ident, $tag: literal, $ty: ident) => {
        fn $parse_func_name(i: &[u8]) -> IResult<&[u8], TableEntry> {
            let (i, _) = tag([$tag])(i)?;
            let (i, data) = take(1usize)(i)?;

            Ok((i, TableEntry::$ty(data[0])))
        }

        fn $write_func_name(w: &mut impl Write, o: &TableEntry) -> Result<(), Error> {
            w.write_u8($tag)?;
            if let TableEntry::$ty(data) = o {
                w.write_u8(*data)?;
            } else {
                panic!("internal error");
            }

            Ok(())
        }
    };
}

macro_rules! gen_simple_type {
    ($parse_func_name: ident, $write_func_name: ident, $tag: literal, $ty: ident) => {
        fn $parse_func_name(i: &[u8]) -> IResult<&[u8], TableEntry> {
            let (i, _) = tag([$tag])(i)?;

            Ok((i, TableEntry::$ty))
        }

        fn $write_func_name(w: &mut impl Write, _o: &TableEntry) -> Result<(), Error> {
            Ok(w.write_u8($tag)?)
        }
    };
}
macro_rules! gen_data_write {
    ($func_name: ident, $tag: literal, $ty: ident) => {
        fn $func_name(w: &mut impl Write, o: &TableEntry) -> Result<(), Error> {
            w.write_u8($tag)?;
            if let TableEntry::$ty(data) = o {
                w.write_all(data)?;
            } else {
                panic!("internal error");
            }
            Ok(())
        }
    };
}

gen_object_type!(parse_object, write_object, 0x00, Object);
gen_u8_type!(parse_open_door, write_open_door, 0x01, OpenDoor);
gen_u8_type!(
    parse_push_block_gated_door,
    write_push_block_gated_door,
    0x02,
    PushBlockGatedDoor
);
gen_u8_type!(
    parse_enemy_gated_door,
    write_enemy_gated_door,
    0x03,
    EnemyGatedDoor
);
gen_u8_type!(parse_bombable_door, write_bombable_door, 0x05, BombableDoor);
gen_object_type!(
    parse_push_block_gated_object,
    write_push_block_gated_object,
    0x06,
    PushBlockGatedObject
);
gen_object_type!(
    parse_enemy_gated_object,
    write_enemy_gated_object,
    0x07,
    EnemyGatedObject
);
gen_object_type!(
    parse_bell_gated_object,
    write_bell_gated_object,
    0x08,
    BellGatedObject
);
gen_simple_type!(parse_dark_room, write_dark_room, 0x09, DarkRoom);
gen_u8_type!(parse_boss_door, write_boss_door, 0x0a, BossDoor);

fn parse_unknown_0b(i: &[u8]) -> IResult<&[u8], TableEntry> {
    let (i, _) = tag([0x0b])(i)?;
    let (i, data) = take(3usize)(i)?;
    Ok((i, TableEntry::Unknown0b([data[0], data[1], data[2]])))
}
gen_data_write!(write_unknown_0b, 0x0b, Unknown0b);

gen_object_type!(parse_burnable, write_burnable, 0x0c, Burnable);

fn parse_hidden_room(i: &[u8]) -> IResult<&[u8], TableEntry> {
    let (i, _) = tag([0x0d])(i)?;
    let (i, data) = take(3usize)(i)?;
    Ok((i, TableEntry::HiddenRoom([data[0], data[1], data[2]])))
}
gen_data_write!(write_hidden_room, 0x0d, HiddenRoom);

gen_simple_type!(
    parse_falcon_boots_needed,
    write_falcon_boots_needed,
    0x81,
    FalconBootsNeeded
);

fn parse_npc(i: &[u8]) -> IResult<&[u8], TableEntry> {
    let (i, _) = tag([0x9a])(i)?;
    let (i, data) = take(5usize)(i)?;
    Ok((
        i,
        TableEntry::Npc([data[0], data[1], data[2], data[3], data[4]]),
    ))
}
gen_data_write!(write_npc, 0x9a, Npc);

gen_object_type!(parse_ouch_rope, write_ouch_rope, 0xbd, OuchRope);
gen_object_type!(
    parse_arrow_launcher,
    write_arrow_launcher,
    0xbf,
    ArrowLauncher
);
gen_object_type!(parse_swords, write_swords, 0xc0, Swords);
gen_object_type!(parse_ghost_spawner, write_ghost_spawner, 0xc1, GhostSpawner);
gen_object_type!(
    parse_fireball_spawner,
    write_fireball_spawner,
    0xc6,
    FireballSpawner
);

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
gen_data_write!(write_shop_item, 0xda, ShopItem);

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
gen_data_write!(write_unknown_e1, 0xe1, UnknownE1);

fn parse_unknown_f4(i: &[u8]) -> IResult<&[u8], TableEntry> {
    let (i, _) = tag([0xf4])(i)?;
    let (i, data) = take(5usize)(i)?;
    Ok((
        i,
        TableEntry::UnknownF4([data[0], data[1], data[2], data[3], data[4]]),
    ))
}
gen_data_write!(write_unknown_f4, 0xf4, UnknownF4);

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

    if !i.is_empty() && i[0] != 0xff {
        return Err(format_err!("unparsed input: {:x?}", i));
    }

    Ok(data.len() - i.len())
}

pub fn parse_object_table(data: &[u8]) -> Result<Vec<TableEntry>, Error> {
    let (i, table) =
        many0(parse_object_table_entry)(data).map_err(|e| format_err!("parse error: {}", e))?;

    if !i.is_empty() {
        return Err(format_err!("unparsed input: {:x?}", i));
    }

    Ok(table)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn run_parse_test(data: &[u8], entry: TableEntry) {
        let mut c = Cursor::new(Vec::new());
        entry.write(&mut c).unwrap();
        let enc_data = c.into_inner();
        assert_eq!(&data[..], &enc_data[..]);

        assert_eq!(parse_object_table_entry(data), Ok((&[][..], entry)));
    }

    #[test]
    fn test_parse_entries() {
        run_parse_test(
            &[0x00, 0x52, 0xa5],
            TableEntry::Object(ObjectInfo {
                x: 2,
                y: 5,
                id: 0xa5,
            }),
        );

        run_parse_test(&[0x01, 0x02], TableEntry::OpenDoor(0x02));

        run_parse_test(&[0x02, 0x01], TableEntry::PushBlockGatedDoor(0x01));

        run_parse_test(&[0x03, 0x08], TableEntry::EnemyGatedDoor(0x08));

        run_parse_test(&[0x05, 0x0a], TableEntry::BombableDoor(0x0a));

        run_parse_test(
            &[0x06, 0x25, 0x5a],
            TableEntry::PushBlockGatedObject(ObjectInfo {
                x: 5,
                y: 2,
                id: 0x5a,
            }),
        );

        run_parse_test(
            &[0x07, 0x25, 0x5a],
            TableEntry::EnemyGatedObject(ObjectInfo {
                x: 5,
                y: 2,
                id: 0x5a,
            }),
        );

        run_parse_test(
            &[0x08, 0x25, 0x5a],
            TableEntry::BellGatedObject(ObjectInfo {
                x: 5,
                y: 2,
                id: 0x5a,
            }),
        );

        run_parse_test(&[0x09], TableEntry::DarkRoom);

        run_parse_test(&[0x0a, 0x50], TableEntry::BossDoor(0x50));

        run_parse_test(
            &[0x0b, 0x46, 0x2a, 0x04],
            TableEntry::Unknown0b([0x46, 0x2a, 0x04]),
        );

        run_parse_test(
            &[0x0c, 0x52, 0xa5],
            TableEntry::Burnable(ObjectInfo {
                x: 2,
                y: 5,
                id: 0xa5,
            }),
        );

        run_parse_test(
            &[0x0d, 0x14, 0x14, 0x33],
            TableEntry::HiddenRoom([0x14, 0x14, 0x33]),
        );

        run_parse_test(&[0x81], TableEntry::FalconBootsNeeded);

        run_parse_test(
            &[0x9a, 0x48, 0x02, 0x03, 0x00, 0x40],
            TableEntry::Npc([0x48, 0x02, 0x03, 0x00, 0x40]),
        );

        run_parse_test(
            &[0xbd, 0x25, 0x5a],
            TableEntry::OuchRope(ObjectInfo {
                x: 5,
                y: 2,
                id: 0x5a,
            }),
        );

        run_parse_test(
            &[0xbf, 0x25, 0x5a],
            TableEntry::ArrowLauncher(ObjectInfo {
                x: 5,
                y: 2,
                id: 0x5a,
            }),
        );

        run_parse_test(
            &[0xc0, 0x25, 0x5a],
            TableEntry::Swords(ObjectInfo {
                x: 5,
                y: 2,
                id: 0x5a,
            }),
        );

        run_parse_test(
            &[0xc1, 0x25, 0x5a],
            TableEntry::GhostSpawner(ObjectInfo {
                x: 5,
                y: 2,
                id: 0x5a,
            }),
        );

        run_parse_test(
            &[0xc6, 0x25, 0x5a],
            TableEntry::FireballSpawner(ObjectInfo {
                x: 5,
                y: 2,
                id: 0x5a,
            }),
        );

        run_parse_test(
            &[0xda, 0x46, 0x00, 0x00, 0x02, 0x00, 0x01, 0x01],
            TableEntry::ShopItem([0x46, 0x00, 0x00, 0x02, 0x00, 0x01, 0x01]),
        );

        run_parse_test(
            &[0xe1, 0x48, 0x02, 0x00, 0x7d, 0x41, 0x56, 0x2e, 0x81, 0x01],
            TableEntry::UnknownE1([0x48, 0x02, 0x00, 0x7d, 0x41, 0x56, 0x2e, 0x81, 0x01]),
        );

        run_parse_test(
            &[0xf4, 0xa7, 0x02, 0x03, 0x40, 0x43],
            TableEntry::UnknownF4([0xa7, 0x02, 0x03, 0x40, 0x43]),
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
