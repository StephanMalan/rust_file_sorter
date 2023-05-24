use std::io::{Read, Seek};

use chrono::{DateTime, Utc};

use crate::util;
use crate::util::io::StepableBuffReader;
use crate::validate;

pub const EXIF_SIGN_1: &[u8] = &[0xFF, 0xD8, 0xFF, 0xE0];
pub const EXIF_SIGN_2: &[u8] = &[0xFF, 0xD8, 0xFF, 0xE1];
pub const EXIF_TAG: &[u8] = &[0x45, 0x78, 0x69, 0x66]; // Exif
pub const JFIF_TAG: &[u8] = &[00, 0x10, 0x4a, 0x46, 0x49, 0x46, 00, 0x01];
const BIG_ENDIAN_TAG: &[u8] = &[0x4d, 0x4d]; // MM
const LITTLE_ENDIAN_TAG: &[u8] = &[0x49, 0x49]; // II
const HEADER: &[u8] = &[00, 0x2a];
const IFD_OFFSET: &[u8] = &[00, 00, 00, 0x08];
const IFD_POINTER: &[u8] = &[0x87, 0x69];
const DATE_TIME_TAG: &[u8] = &[0x90, 0x03];
const END_TAG: &[u8] = &[00, 00, 00, 00];

pub fn parse_datetime<R: Read + Seek>(reader: &mut StepableBuffReader<R>) -> Option<DateTime<Utc>> {
    reader.increment_by(4); // signature
    if reader.compare_bytes(JFIF_TAG.to_vec()) {
        reader.increment_by(12);
    } else {
        reader.increment_by(2);
    }
    // println!("{:?}", reader.peak(16));
    validate!(reader.compare_bytes(EXIF_TAG.to_vec()))?;
    // println!("test");
    reader.increment_by(2);

    let big_endian = match reader.read(2).as_slice() {
        BIG_ENDIAN_TAG => true,
        LITTLE_ENDIAN_TAG => false,
        _ => return None,
    };

    let start_offset = reader.total_offset;
    validate!(reader.compare_endian_bytes(HEADER.to_vec(), big_endian))?;
    validate!(reader.compare_endian_bytes(IFD_OFFSET.to_vec(), big_endian))?;
    // println!("value: {}", big_endian);
    reader.increment_by(2); // interop

    loop {
        if reader.compare_endian_bytes(DATE_TIME_TAG.to_vec(), big_endian) {
            reader.increment_by(2); // Type
            let length = reader.read_u32(big_endian);
            let mut offset = reader.read_u32(big_endian) as usize;
            if offset == 0 {
                return None;
            }
            offset += start_offset - 2;
            // println!("Datetime tag length {} and offset {}", length, offset);
            let buffer = reader.read_from(offset, length as usize);
            // println!("Datetime bytes {:?}", buffer);
            let mut datetime = std::str::from_utf8(buffer.as_slice()).unwrap().trim();
            datetime =
                datetime.trim_matches(&[char::from(0), char::from(10), char::from(13)] as &[_]);
            // println!("{}", datetime);
            return Some(util::parse_datetime(datetime)?);
        }
        if reader.compare_endian_bytes(END_TAG.to_vec(), big_endian) {
            return None;
        }
        if reader.compare_endian_bytes(IFD_POINTER.to_vec(), big_endian) {
            reader.increment_by(2); // Type
            reader.increment_by(4); // Count
            let offset = reader.read_u32(big_endian) as usize;
            // println!("Incrementing by {}", offset);
            reader.increment_by((start_offset + offset) - reader.total_offset);
            continue;
        }
        if !reader.increment_by(12) {
            break;
        }
    }

    return None;
}
