mod metadata_parser;
mod models;
mod sorting;
mod util;

use models::{Config, FileTypes};
use std::{collections::HashMap, str, time::Instant};

fn main() {
    let now = Instant::now();

    let file_extensions: HashMap<&str, &FileTypes> = [
        ("doc", FileTypes::DOCUMENT),
        ("docx", FileTypes::DOCUMENT),
        ("pdf", FileTypes::DOCUMENT),
        ("ppt", FileTypes::DOCUMENT),
        ("pptx", FileTypes::DOCUMENT),
        ("xls", FileTypes::DOCUMENT),
        ("xlsx", FileTypes::DOCUMENT),
        ("jpeg", FileTypes::IMAGE),
        ("jpg", FileTypes::IMAGE),
        ("png", FileTypes::IMAGE),
        ("avi", FileTypes::VIDEO),
        ("mov", FileTypes::VIDEO),
        ("mp4", FileTypes::VIDEO),
    ]
    .iter()
    .map(|(e, f)| return (e.clone(), f.clone()))
    .collect();

    let config = Config {
        // source_dir: "/mnt/c/source/LOUISE/VIDEO SPEEL KLAVIER".to_owned(),
        source_dir: "/mnt/c/source/my pictures".to_owned(),
        doc_dir: "/mnt/c/dest/doc".to_owned(),
        image_dir: "/mnt/c/dest/image".to_owned(),
        video_dir: "/mnt/c/dest/video".to_owned(),
        file_exts: file_extensions,
    };
    config.create_folders();

    // what happens if two files are identified. both hashes are the same, but one has exif data
    // if dest folder has files, move files to interim place

    sorting::sort_files(config);
    // let dt = metadata_parser::datetime_parser::read_datetime(&std::path::PathBuf::from(
    //     "/mnt/c/source/Whatsapp Images/IMG-20141009-WA0000.jpg",
    // ));
    // println!("{}", dt.unwrap().to_rfc2822());

    println!("Seconds elapased: {}s", now.elapsed().as_secs_f32());
}
