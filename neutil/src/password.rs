use failure::{format_err, Error};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub(crate) struct PasswordOpt {
    password: String,
}

fn decode_char(c: char) -> Result<u8, Error> {
    if !c.is_ascii() {
        return Err(format_err!("invalid character {}", c as char));
    }
    let c = c as u8;

    // 0 - 25
    if b'A' <= c && c <= b'Z' {
        return Ok(c - b'A');
    }
    // 26 - 34
    if b'1' <= c && c <= b'9' {
        return Ok(c - b'1' + 26);
    }
    // 35 - 60
    if b'a' <= c && c <= b'z' {
        return Ok(c - b'a' + 35);
    }
    if c == b'#' {
        return Ok(61);
    }
    if c == b'$' {
        return Ok(62);
    }
    if c == b'%' {
        return Ok(63);
    }

    Err(format_err!("invalid character {}", c as char))
}

fn salt_byte(i: u8) -> u8 {
    let table = [
        0x1f, 0x3a, 0x06, 0x3f, 0x21, 0x3f, 0x30, 0x37, 0x1a, 0x01, 0x20, 0x3f, 0x35, 0x03, 0x29,
        0x2b, 0x3e, 0x3f, 0x01, 0x00, 0x03, 0x2c, 0x37, 0x07, 0x3d, 0x11, 0x1e, 0x34, 0x3f, 0x19,
        0x30, 0x28, 0x37, 0x37, 0x3c, 0x0d, 0x1e, 0x31, 0x0c, 0x05, 0x35, 0x11, 0x3f, 0x24, 0x3f,
        0x3b, 0x3f, 0x26, 0x3b, 0x33, 0x3c, 0x39, 0x2e, 0x3e, 0x31, 0x08, 0x38, 0x1f, 0x00, 0x37,
        0x19, 0x24, 0x12, 0x00,
    ];
    table[(i & 0x3f) as usize]
}

fn decode_section(data: &mut [u8]) -> Result<(), Error> {
    // First de-salt the data.
    let mut salt = data[0];
    #[allow(clippy::needless_range_loop)]
    for i in 1..8 {
        data[i] ^= salt_byte(salt);
        salt = (salt + 1) & 0x3f;
    }

    // Now do a "forward xor" on the data.
    for i in (0..6).rev() {
        data[i + 1] ^= data[i];
    }

    // Now calc checksum
    let mut sum = 0;
    #[allow(clippy::needless_range_loop)]
    for i in 0..7 {
        sum += data[i] & 0x3f;
    }
    sum &= 0x3f;

    let expected_sum = data[7] & 0x3f;
    if sum != expected_sum {
        return Err(format_err!(
            "checksum {:02x} does match the expected {:02} {:x?}",
            sum,
            expected_sum,
            data
        ));
    }

    Ok(())
}

pub(crate) fn command(opt: &PasswordOpt) -> Result<(), Error> {
    if opt.password.len() != 24 {
        return Err(format_err!("Password is not 24 characters in length."));
    }

    let bytes: Result<Vec<u8>, _> = opt.password.chars().map(decode_char).collect();
    let mut bytes = bytes?;

    decode_section(&mut bytes[0..8])?;
    decode_section(&mut bytes[8..16])?;
    decode_section(&mut bytes[16..24])?;

    for (i, b) in bytes.iter().enumerate() {
        println!("{:02x}: {:02x}", i, b);
    }
    Ok(())
}
