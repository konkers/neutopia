use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;

use byteorder::{BigEndian, WriteBytesExt};
use failure::{format_err, Error};

fn make_target_file<P: AsRef<Path>>(path: P, size: usize, value: u8) -> Result<(), Error> {
    let data: Vec<u8> = vec![value; size];
    let mut f = File::create(path)?;
    f.write_all(&data)?;

    Ok(())
}

fn run_bass(bass: &PathBuf, source: &PathBuf, out: &PathBuf) -> Result<(), Error> {
    let _out = Command::new(bass)
        .args(&["-o", &out.to_string_lossy(), &source.to_string_lossy()])
        .output()?;
    Ok(())
}

#[derive(Debug)]
struct Hunk {
    offset: u32,
    data: Vec<u8>,
}

fn diff_files(zeros_path: &PathBuf, effs_path: &PathBuf) -> Result<Vec<Hunk>, Error> {
    let mut f = File::open(zeros_path)?;
    let mut zeros = Vec::new();
    f.read_to_end(&mut zeros)?;

    let mut f = File::open(effs_path)?;
    let mut effs = Vec::new();
    f.read_to_end(&mut effs)?;

    if zeros.len() != effs.len() {
        return Err(format_err!(
            "Can't diff.  File lengths differ {} != {}",
            zeros.len(),
            effs.len()
        ));
    }

    let mut last_match = 0;
    let mut hunks: Vec<Hunk> = Vec::new();

    for (offset, (a, b)) in zeros.iter().zip(effs.iter()).enumerate() {
        if a == b {
            if (last_match + 1) == offset {
                let mut hunk = hunks.pop().unwrap();
                hunk.data.push(*a);
                hunks.push(hunk);
            } else {
                hunks.push(Hunk {
                    offset: offset as u32,
                    data: vec![*a],
                });
            }
            last_match = offset;
        }
    }

    Ok(hunks)
}

fn write_ips(w: &mut impl Write, hunks: &[Hunk]) -> Result<(), Error> {
    w.write_all("PATCH".as_bytes())?;
    for hunk in hunks {
        w.write_u24::<BigEndian>(hunk.offset)?;
        w.write_u16::<BigEndian>(hunk.data.len() as u16)?;
        w.write(&hunk.data)?;
    }
    w.write_all("EOF".as_bytes())?;
    Ok(())
}

pub fn build(
    bass: &PathBuf,
    rom_size: u32,
    tmp_dir: &PathBuf,
    src_files: &[PathBuf],
    out: &PathBuf,
) -> Result<(), Error> {
    let zeros_path = tmp_dir.join("00.bin");
    let effs_path = tmp_dir.join("ff.bin");

    let mut hunks: Vec<Hunk> = Vec::new();
    for file in src_files {
        make_target_file(&zeros_path, rom_size as usize, 0x00)?;
        make_target_file(&effs_path, rom_size as usize, 0xff)?;
        run_bass(bass, &file, &zeros_path)?;
        run_bass(bass, &file, &effs_path)?;
        let mut new_hunks = diff_files(&zeros_path, &effs_path)?;
        hunks.append(&mut new_hunks);
    }

    let mut f = File::create(out)?;
    write_ips(&mut f, &hunks)?;

    Ok(())
}
