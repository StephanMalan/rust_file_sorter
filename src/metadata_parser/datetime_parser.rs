use std::panic;
use std::{fs::File, path::PathBuf};

use crate::metadata_parser::exif_parser;
use crate::metadata_parser::quicktime_parser;
use crate::metadata_parser::riff_parser;
use crate::util::io::StepableBuffReader;
use chrono::{DateTime, Utc};

// const MEDIA_TAG: &[u8] = "mdhd".as_bytes();
// const EXIF_TAG: &[u8] = "Exif".as_bytes();
// const EXIF_HEADER_NULL_TAG: &[u8] = &[0, 0];
// const EXIF_II_TAG: &[u8] = "II".as_bytes();
// const EXIF_HEADER_TAG: &[u8] = &[42, 00];
// const EXIF_0_OFFSET_TAG: &[u8] = &[08, 00, 00, 00];
// const EXIF_NUM_OPP_TAG: &[u8] = &[11, 00];
// const EXIF_DATETIME_TAG: &[u8] = &[50, 01];
// const EXIF_END_TAG: &[u8] = &[00, 00, 00, 00];
// const CHUNK_SIZE: usize = 1024 as usize;

// struct FileData {
//     start_index: usize,
//     index: usize,
//     data: [u8; CHUNK_SIZE],
// }

// impl FileData {
//     fn new(data: [u8; CHUNK_SIZE]) -> Self {
//         FileData {
//             index: 0,
//             start_index: 0,
//             data,
//         }
//     }

//     fn matches_tag_incr(&mut self, compared_to: &[u8]) -> Result<bool, ()> {
//         self.matches_tag(compared_to)?;
//         self.increment_by(compared_to.len());
//         Ok(true)
//     }

//     fn matches_tag(&self, compared_to: &[u8]) -> Result<bool, ()> {
//         let data = &self.data[self.index..(self.index + compared_to.len())];
//         if !data.iter().zip(compared_to).all(|(a, b)| a == b) {
//             return Err(());
//         }
//         Ok(true)
//     }

//     fn get_data(&self, range: Range<usize>) -> &[u8] {
//         return &self.data[range];
//     }

//     fn next_byte(&mut self) -> u8 {
//         let data = self.data[self.index];
//         self.increment();
//         return data;
//     }

//     fn next_u64(&mut self) -> u64 {
//         ((self.next_byte() as u64) << 24)
//             + ((self.next_byte() as u64) << 16)
//             + ((self.next_byte() as u64) << 8)
//             + ((self.next_byte() as u64) << 0)
//     }

//     fn save_index(&mut self) {
//         self.start_index = self.index;
//     }

//     fn increment(&mut self) {
//         self.index += 1;
//     }

//     fn increment_by(&mut self, value: usize) {
//         self.index += value;
//     }

//     fn increment_by_tag(&mut self, tag: &[u8]) {
//         self.increment_by(tag.len());
//     }
// }

pub(crate) fn read_datetime(path: &PathBuf) -> Option<DateTime<Utc>> {
    let test = panic::catch_unwind(|| {
        let mut reader = StepableBuffReader::new(File::open(path).unwrap());
        // if file starts with FF D8 FF E1 or FF D8 FF E0 -> read exif
        // if file starts with 49 49 2A 00 or 4D 4D 00 2A -> read tiff
        // if file is video, check for mdhd
        if reader.peak(4).eq(&exif_parser::EXIF_SIGN_1)
            || reader.peak(4).eq(&exif_parser::EXIF_SIGN_2)
        {
            // println!("EXIF");
            return exif_parser::parse_datetime(&mut reader);
        }
        if reader.compare_bytes(riff_parser::RIFF_SIGN.to_vec()) {
            // println!("RIFF");
            return riff_parser::parse_datetime(&mut reader);
        }
        if reader
            .peak(8)
            .ends_with(&quicktime_parser::QUICKTIME_SIGN_1)
            || reader
                .peak(8)
                .ends_with(&quicktime_parser::QUICKTIME_SIGN_2)
        {
            // println!("Quicktime");
            return quicktime_parser::parse_datetime(path);
        }
        None
    });
    match test {
        Ok(dt) => return Some(dt?),
        Err(_) => {
            println!("Oh fuck {}", path.display());
            return None;
        }
    }

    return None;
}

// fn read_file_chunk(path: &str, start: bool) -> Result<[u8; CHUNK_SIZE], Error> {
//     let mut buffer: [u8; CHUNK_SIZE] = [0; CHUNK_SIZE];
//     let mut file = File::open(path)?;
//     let seek_from = match start {
//         true => SeekFrom::Start(0),
//         false => SeekFrom::End(-(CHUNK_SIZE as i64)),
//     };
//     file.seek(seek_from)?;
//     file.read_exact(&mut buffer[..CHUNK_SIZE])?;
//     return Ok(buffer);
// }

// fn parse_datetime(buffer: [u8; CHUNK_SIZE]) -> Result<String, ()> {
//     println!("parse_datetime");
//     let mut file_data = FileData::new(buffer);
//     for i in 0..(buffer.len() - 4) {
//         if file_data.matches_tag(MEDIA_TAG).is_ok() {
//             return parse_mdhd_datetime(file_data);
//         } else if file_data.matches_tag(EXIF_TAG).is_ok() {
//             return parse_exif_datetime(file_data);
//         }
//         file_data.increment();
//     }
//     Err(())
// }

// fn parse_mdhd_datetime(mut file_data: FileData) -> Result<String, ()> {
//     file_data.increment_by_tag(MEDIA_TAG);
//     file_data.increment_by(4); // Version and flags
//     let seconds = file_data.next_u64();
//     return Ok(Local
//         .with_ymd_and_hms(1904, 1, 1, 0, 0, 0)
//         .unwrap()
//         .checked_add_signed(Duration::seconds(seconds as i64))
//         .unwrap()
//         .format("%Y/%m/%d %H:%M:%S")
//         .to_string());
// }

// fn parse_exif_datetime(mut file_data: FileData) -> Result<String, ()> {
//     println!("parse_exif_datetime");
//     file_data.increment_by_tag(EXIF_TAG);
//     file_data.save_index();

//     file_data.matches_tag_incr(EXIF_HEADER_NULL_TAG)?;
//     file_data.matches_tag_incr(EXIF_II_TAG)?;
//     file_data.matches_tag_incr(EXIF_HEADER_TAG)?;
//     file_data.matches_tag_incr(EXIF_0_OFFSET_TAG)?;
//     file_data.matches_tag_incr(EXIF_NUM_OPP_TAG)?;
//     loop {
//         if file_data.matches_tag(EXIF_DATETIME_TAG).is_ok() {
//             file_data.increment_by_tag(EXIF_DATETIME_TAG);
//             file_data.increment_by(2); // Type
//             let length = file_data.next_byte() as usize;
//             file_data.increment_by(3); // Length filler
//             let offset = (file_data.next_byte() as usize) + file_data.start_index + 2;
//             let datetime_bytes = file_data.get_data(offset..offset + length);
//             let datetime = str::from_utf8(datetime_bytes).unwrap().trim();
//             return Ok(datetime.to_owned());
//         } else if file_data.matches_tag(EXIF_END_TAG).is_ok() {
//             return Err(());
//         }
//         file_data.increment_by(12); // Size of each EXIF tag description
//     }
// }
