use std::{
    io::{Read, Seek},
    str::from_utf8,
};

use chrono::{DateTime, Utc};

use crate::util::{self, io::StepableBuffReader};

pub const RIFF_SIGN: &[u8] = "RIFF".as_bytes();

#[rustfmt::skip]
mod constants {
    #[derive(PartialEq, Clone)]
    pub struct ChunkType<'a> {
        pub identifier: &'a str,
        pub container: bool,
    }

    pub const IDIT: ChunkType = ChunkType {container: false, identifier: "IDIT"};
    pub const LIST_HDRL: ChunkType = ChunkType {container: true, identifier: "hdrl"};
    pub const MOVI: ChunkType = ChunkType {container: true, identifier: "movi"};
}

struct Chunk {
    container: bool,
    id: String,
    start_index: usize,
    size: usize,
}

impl PartialEq<constants::ChunkType<'_>> for Chunk {
    fn eq(&self, other: &constants::ChunkType) -> bool {
        self.id == other.identifier && self.container == other.container
    }
}

pub fn parse_datetime<R: Read + Seek>(reader: &mut StepableBuffReader<R>) -> Option<DateTime<Utc>> {
    let riff_length = reader.read_u32(false) as usize;
    // reader.increment_by(4); // Riff length
    reader.increment_by(4); // Riff type
    let chunk = find_chunk(
        reader,
        vec![constants::LIST_HDRL, constants::IDIT],
        riff_length,
    );
    match chunk {
        Ok(chunk) if chunk.is_some() => {
            let buffer = reader.read(chunk?.size);
            let mut dt = from_utf8(buffer.as_slice()).unwrap_or("").trim();
            dt = dt.trim_matches(&[char::from(0), char::from(10), char::from(13)] as &[_]);
            return Some(util::parse_datetime(dt)?);
        }
        _ => None,
    }
}

fn find_chunk<'a, R: Read + Seek>(
    reader: &'a mut StepableBuffReader<R>,
    chunk_tags: Vec<constants::ChunkType<'a>>,
    chunk_length: usize,
) -> Result<Option<Chunk>, ()> {
    let mut offset = 0;
    while offset < chunk_length {
        let chunk = get_chunk(reader);
        // println!("Chunk ({}, {})", chunk.id, chunk.container);
        if &chunk == chunk_tags.get(0).unwrap() {
            if chunk_tags.len() == 1 {
                return Ok(Some(chunk));
            }
            let mut new_chunk_tags = chunk_tags.clone();
            new_chunk_tags.remove(0);
            // println!("going in!");
            let result = find_chunk(reader, new_chunk_tags, chunk.size);
            match result {
                Ok(result) if result.is_some() => return Ok(result),
                Err(_) => return Err(()),
                _ => (),
            }
        } else {
            if chunk == constants::MOVI {
                return Err(());
            }
            // println!("Incrementing by: {}", chunk.size);
            offset += chunk.size;
            reader.increment_by(chunk.size);
        }
    }
    // println!("done");
    Ok(None)
}

fn get_chunk<R: Read + Seek>(reader: &mut StepableBuffReader<R>) -> Chunk {
    let chunk_tag = String::from_utf8(reader.read(4).to_vec()).unwrap();
    let size = reader.read_u32(false) as usize;
    if chunk_tag == "RIFF" || chunk_tag == "LIST" {
        let container_type = String::from_utf8(reader.read(4).to_vec()).unwrap();
        return Chunk {
            container: true,
            id: container_type,
            start_index: reader.total_offset,
            size: size - 4,
        };
    } else {
        return Chunk {
            container: false,
            id: chunk_tag,
            start_index: reader.total_offset,
            size,
        };
    }
}
