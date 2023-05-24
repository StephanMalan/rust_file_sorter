use std::{
    cmp::min,
    io::{Read, Seek, SeekFrom},
};

const CHUNK_SIZE: usize = 4096 as usize;

pub struct StepableBuffReader<R: Read> {
    reader: R,
    buf1: [u8; CHUNK_SIZE],
    buf1_size: usize,
    buf2: [u8; CHUNK_SIZE],
    buf2_size: usize,
    pointer: usize,
    pub total_offset: usize,
}

impl<R: Read + Seek> StepableBuffReader<R> {
    pub fn new(mut reader: R) -> Self {
        let mut buf1 = [0; CHUNK_SIZE];
        let buf1_size = reader.read(&mut buf1).unwrap();
        let mut buf2 = [0; CHUNK_SIZE];
        let buf2_size = reader.read(&mut buf2).unwrap();
        StepableBuffReader {
            reader,
            buf1,
            buf1_size,
            buf2,
            buf2_size,
            pointer: 0,
            total_offset: 0,
        }
    }

    pub fn available(&self) -> usize {
        return (self.buf1_size - self.pointer) + self.buf2_size;
    }

    pub fn peak(&mut self, num_bytes: usize) -> Vec<u8> {
        if num_bytes > CHUNK_SIZE {
            panic!("Not cool 1")
        }
        if num_bytes > self.available() {
            panic!("Oh rip")
        }
        let mut buffer = Vec::new();
        let flag = min(self.buf1_size - self.pointer, num_bytes);
        buffer.extend_from_slice(&self.buf1[self.pointer..self.pointer + flag]);
        buffer.extend_from_slice(&self.buf2[0..num_bytes - flag]);
        return buffer;
    }

    pub fn increment(&mut self) -> bool {
        return self.increment_by(1);
    }

    pub fn increment_by(&mut self, num_bytes: usize) -> bool {
        if num_bytes > CHUNK_SIZE {
            self.load_from(self.total_offset + num_bytes);
            return true;
            // panic!("Not cool")
        }
        if num_bytes > self.available() {
            return false;
        }
        self.total_offset += num_bytes;
        if self.pointer + num_bytes > self.buf1_size {
            self.pointer = self.pointer + num_bytes - self.buf1_size;
            self.buf1 = self.buf2;
            self.buf1_size = self.buf2_size;
            self.buf2_size = self.reader.read(&mut self.buf2).unwrap();
        } else {
            self.pointer = self.pointer + num_bytes
        }
        return true;
    }

    fn load_from(&mut self, index: usize) {
        self.reader.seek(SeekFrom::Start(index as u64)).unwrap();
        self.buf1_size = self.reader.read(&mut self.buf1).unwrap();
        self.buf2_size = self.reader.read(&mut self.buf2).unwrap();
        self.total_offset = index;
        self.pointer = 0;
    }

    pub fn read(&mut self, num_bytes: usize) -> Vec<u8> {
        if num_bytes > self.available() {
            panic!("Reached end of source");
        }
        let data = self.peak(num_bytes);
        self.increment_by(num_bytes);
        return data;
    }

    pub fn read_u32(&mut self, big_endian: bool) -> u32 {
        let buffer = self.read(4);
        if big_endian {
            return u32::from_be_bytes(buffer.try_into().unwrap());
        }
        return u32::from_le_bytes(buffer.try_into().unwrap());
    }

    pub fn read_to_end(&mut self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&self.buf1[self.pointer..]);
        buffer.extend_from_slice(&self.buf2);
        let _ = self.reader.read_to_end(&mut buffer);
        self.buf1_size = 0;
        self.buf2_size = 0;
        return buffer;
    }

    pub fn compare_bytes(&mut self, bytes: Vec<u8>) -> bool {
        if self.peak(bytes.len()) == bytes {
            self.increment_by(bytes.len());
            return true;
        }
        return false;
    }

    pub fn compare_endian_bytes(&mut self, mut bytes: Vec<u8>, big_endian: bool) -> bool {
        if !big_endian {
            bytes = bytes.iter().rev().cloned().collect();
        }
        if self.peak(bytes.len()) == bytes {
            self.increment_by(bytes.len());
            return true;
        }
        return false;
    }

    pub fn compare_multiple_bytes(&mut self, bytes_list: Vec<Vec<u8>>) -> bool {
        for bytes in bytes_list {
            if self.compare_bytes(bytes) {
                return true;
            }
        }
        return false;
    }

    pub fn read_from(&mut self, offset: usize, length: usize) -> Vec<u8> {
        if offset + length - self.total_offset > self.available() {
            panic!("rip")
        }
        self.increment_by(&offset - &self.total_offset);
        return self.read(length);
    }
}
