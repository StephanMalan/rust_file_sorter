use std::{fs::File, os::unix::prelude::FileExt, path::PathBuf};

use chrono::{DateTime, Duration, TimeZone, Utc};

pub const QUICKTIME_SIGN_1: [u8; 4] = [0x66, 0x74, 0x79, 0x70]; // ftyp
pub const QUICKTIME_SIGN_2: [u8; 4] = [0x6d, 0x64, 0x61, 0x74]; // mdat

struct Atom {
    name: String,
    start_index: u64,
    size: u64,
}

pub fn parse_datetime(path: &PathBuf) -> Option<DateTime<Utc>> {
    let reader = File::open(path).ok()?;
    let atom = find_atom_recursively(
        &reader,
        vec!["moov", "trak", "mdia", "mdhd"],
        0,
        reader.metadata().ok()?.len(),
    )?;

    let mut buffer: [u8; 8] = [0; 8];
    reader
        .read_exact_at(&mut buffer, atom.start_index + 8)
        .ok()?;
    // println!("{:?}", buffer);
    let seconds = u64::from_be_bytes(buffer);
    if seconds == 0 {
        return None;
    }
    let datetime = Utc
        .with_ymd_and_hms(1904, 1, 1, 0, 0, 0)
        .unwrap()
        .checked_add_signed(Duration::seconds(seconds as i64))?;
    // println!("{} {} {}", atom.name, atom.start_index, atom.size);
    // println!("{}", datetime.to_rfc3339());
    return Some(datetime);
}

fn find_atom_recursively(
    reader: &File,
    atom_names: Vec<&str>,
    start_index: u64,
    end_index: u64,
) -> Option<Atom> {
    // println!("{} {} {}", atom_names.get(0)?, start_index, end_index);
    let mut index = start_index;
    while index < end_index {
        let atom = get_atom(reader, index)?;
        if atom.name == atom_names.get(0)?.to_owned() {
            if atom_names.len() == 1 {
                return Some(Atom {
                    name: atom.name,
                    start_index: index,
                    size: atom.size,
                });
            }
            let mut new_atom_names = atom_names.clone();
            new_atom_names.remove(0);
            let value = find_atom_recursively(reader, new_atom_names, index + 8, index + atom.size);
            if value.is_some() {
                return Some(value?);
            }
        }
        index += atom.size;
    }
    return None;
}

fn get_atom(reader: &File, index: u64) -> Option<Atom> {
    let mut buffer: [u8; 4] = [0; 4];
    reader.read_exact_at(&mut buffer, index).ok()?;
    let size = u32::from_be_bytes(buffer) as u64;
    reader.read_exact_at(&mut buffer, index + 4).ok()?;
    let name = String::from_utf8(buffer.to_vec()).ok()?;
    return Some(Atom {
        name,
        start_index: index,
        size,
    });
}
