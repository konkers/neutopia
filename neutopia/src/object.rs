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

#[derive(Debug, PartialEq)]
pub enum TableEntry {
    Object(ObjectInfo),
    OpenDoor(u8),
    PushBlockGatedDoor(u8),
    EnemyGatedDoor(u8),
    BombableDoor(u8),
    PushBlockGatedObject(ObjectInfo),
    EnemyGatedObject(ObjectInfo),
    DarkRoom,
    BossDoor(u8),
    Unknown0b([u8; 3]),
    Swords(ObjectInfo),
    GhostSpawner(ObjectInfo),
    FireballSpawner(ObjectInfo),
    UnknownE1([u8; 9]),
}

fn parse_object_info(i: &[u8]) -> IResult<&[u8], ObjectInfo> {
    let (i, loc) = take(1usize)(i)?;
    let (i, id) = take(1usize)(i)?;
    let x = loc[0] & 0xf;
    let y = loc[0] >> 4;

    Ok((i, ObjectInfo { x, y, id: id[0] }))
}

fn parse_object(i: &[u8]) -> IResult<&[u8], TableEntry> {
    let (i, _) = tag([0x00])(i)?;
    let (i, info) = parse_object_info(i)?;

    Ok((i, TableEntry::Object(info)))
}

fn parse_open_door(i: &[u8]) -> IResult<&[u8], TableEntry> {
    let (i, _) = tag([0x01])(i)?;
    let (i, data) = take(1usize)(i)?;

    Ok((i, TableEntry::OpenDoor(data[0])))
}

fn parse_push_block_gated_door(i: &[u8]) -> IResult<&[u8], TableEntry> {
    let (i, _) = tag([0x02])(i)?;
    let (i, data) = take(1usize)(i)?;

    Ok((i, TableEntry::PushBlockGatedDoor(data[0])))
}

fn parse_enemy_gated_door(i: &[u8]) -> IResult<&[u8], TableEntry> {
    let (i, _) = tag([0x03])(i)?;
    let (i, data) = take(1usize)(i)?;

    Ok((i, TableEntry::EnemyGatedDoor(data[0])))
}

fn parse_bombable_door(i: &[u8]) -> IResult<&[u8], TableEntry> {
    let (i, _) = tag([0x05])(i)?;
    let (i, data) = take(1usize)(i)?;

    Ok((i, TableEntry::BombableDoor(data[0])))
}

fn parse_push_block_gated_object(i: &[u8]) -> IResult<&[u8], TableEntry> {
    let (i, _) = tag([0x06])(i)?;
    let (i, info) = parse_object_info(i)?;

    Ok((i, TableEntry::PushBlockGatedObject(info)))
}

fn parse_enemy_gated_object(i: &[u8]) -> IResult<&[u8], TableEntry> {
    let (i, _) = tag([0x07])(i)?;
    let (i, info) = parse_object_info(i)?;

    Ok((i, TableEntry::EnemyGatedObject(info)))
}

fn parse_dark_room(i: &[u8]) -> IResult<&[u8], TableEntry> {
    let (i, _) = tag([0x09])(i)?;
    Ok((i, TableEntry::DarkRoom))
}

fn parse_unknown_0b(i: &[u8]) -> IResult<&[u8], TableEntry> {
    let (i, _) = tag([0x0b])(i)?;
    let (i, data) = take(3usize)(i)?;
    Ok((i, TableEntry::Unknown0b([data[0], data[1], data[2]])))
}

fn parse_boss_door(i: &[u8]) -> IResult<&[u8], TableEntry> {
    let (i, _) = tag([0x0a])(i)?;
    let (i, data) = take(1usize)(i)?;

    Ok((i, TableEntry::BossDoor(data[0])))
}

fn parse_swords(i: &[u8]) -> IResult<&[u8], TableEntry> {
    let (i, _) = tag([0xc0])(i)?;
    let (i, info) = parse_object_info(i)?;

    Ok((i, TableEntry::Swords(info)))
}

fn parse_ghost_spawner(i: &[u8]) -> IResult<&[u8], TableEntry> {
    let (i, _) = tag([0xc1])(i)?;
    let (i, info) = parse_object_info(i)?;

    Ok((i, TableEntry::GhostSpawner(info)))
}

fn parse_fireball_spawner(i: &[u8]) -> IResult<&[u8], TableEntry> {
    let (i, _) = tag([0xc6])(i)?;
    let (i, info) = parse_object_info(i)?;

    Ok((i, TableEntry::FireballSpawner(info)))
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

fn parse_object_table_entry(i: &[u8]) -> IResult<&[u8], TableEntry> {
    alt((
        parse_object,
        parse_open_door,
        parse_push_block_gated_door,
        parse_enemy_gated_door,
        parse_bombable_door,
        parse_push_block_gated_object,
        parse_enemy_gated_object,
        parse_dark_room,
        parse_unknown_0b,
        parse_boss_door,
        parse_swords,
        parse_ghost_spawner,
        parse_fireball_spawner,
        parse_unknown_e1,
    ))(i)
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
            parse_object_table_entry(&[0xe1, 0x48, 0x02, 0x00, 0x7d, 0x41, 0x56, 0x2e, 0x81, 0x01]),
            Ok((
                &[][..],
                TableEntry::UnknownE1([0x48, 0x02, 0x00, 0x7d, 0x41, 0x56, 0x2e, 0x81, 0x01])
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
