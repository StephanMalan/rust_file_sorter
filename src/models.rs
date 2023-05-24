use std::{collections::HashMap, fs, path::PathBuf};

pub enum FileTypes {
    DOCUMENT,
    IMAGE,
    VIDEO,
}

pub enum MediaType {
    IMAGE,
    VIDEO,
}

#[derive(Debug)]
pub enum Action {
    Copy,
    Delete,
    Move,
}

pub struct Config<'a> {
    pub source_dir: String,
    pub doc_dir: String,
    pub image_dir: String,
    pub video_dir: String,
    pub file_exts: HashMap<&'a str, &'a FileTypes>,
}

impl Config<'_> {
    pub fn create_folders(&self) {
        let _ = fs::create_dir_all(&self.doc_dir);
        let _ = fs::create_dir_all(&self.image_dir);
        let _ = fs::create_dir_all(format!("{}_temp", &self.image_dir));
        let _ = fs::create_dir_all(&self.video_dir);
        let _ = fs::create_dir_all(format!("{}_temp", &self.video_dir));
    }

    pub fn get_destination_folders(&self) -> Vec<PathBuf> {
        return vec![
            self.doc_dir.clone().into(),
            self.image_dir.clone().into(),
            (self.image_dir.clone() + "_temp").into(),
            self.video_dir.clone().into(),
            (self.video_dir.clone() + "_temp").into(),
        ];
    }
}
