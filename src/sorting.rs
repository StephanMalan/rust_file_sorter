use std::{
    collections::HashMap,
    fs::{self, File},
    path::{Path, PathBuf},
};

use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use jwalk::WalkDir;
use rayon::prelude::*;
use xxhash_rust::xxh3::xxh3_64;

use crate::{
    metadata_parser::datetime_parser::read_datetime,
    models::{Action, Config, FileTypes, MediaType},
    util::io::StepableBuffReader,
};

pub fn sort_files(config: Config) {
    // TODO: need to implement the command line code
    // TODO: need to implement the interim folder feature
    // all existing files should be processed
    // should ignore file name numbering for the purpose of comparing the file in order to move/copy
    // log the actions to a file
    // add a bunch of testing
    // look into multistage progress bar
    // maybe we should find a way to iterate over file size and not num files?
    // maybe actions should be split up (move, copy, delete)
    // look into faster file copy (esp for larger files)
    let exis_files = index_files(config.get_destination_folders(), &config, true);
    let mut actions = process_files(&exis_files, &config, None);

    let new_files = index_files(vec![(&config.source_dir).into()], &config, false);
    actions.append(&mut process_files(&new_files, &config, Some(&exis_files)));

    for action in &actions {
        println!("{:#?} {:#?} {:#?}", action.0, action.1, action.2)
    }

    process_actions(&actions);

    println!("Num actions: {}", actions.len());
}

fn index_files(
    source_dirs: Vec<PathBuf>,
    config: &Config,
    existing: bool,
) -> HashMap<u64, PathBuf> {
    let message_type = match existing {
        true => "existing",
        false => "new",
    };
    let bar = ProgressBar::new_spinner();
    bar.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner} [{elapsed_precise}] {wide_msg}")
            .unwrap(),
    );
    bar.enable_steady_tick(std::time::Duration::from_millis(200));
    bar.set_message(format!("Indexing {} files", message_type));

    let indexed_files: HashMap<u64, PathBuf> = source_dirs
        .iter()
        .flat_map(|dir| {
            let files: HashMap<u64, PathBuf> = WalkDir::new(&dir)
                .into_iter()
                .par_bridge()
                .filter(|e| e.is_ok())
                .filter_map(|e| {
                    let entry = e.unwrap();
                    if entry.metadata().unwrap().is_dir() {
                        bar.set_message(format!(
                            "Indexing {} files: {}",
                            message_type,
                            entry.path().display()
                        ));
                        return None;
                    }
                    let path = &entry.path();
                    let ext_result = &path.extension();
                    if ext_result.is_none()
                        || !config.file_exts.contains_key(
                            &ext_result.unwrap().to_ascii_lowercase().to_str().unwrap(),
                        )
                    {
                        // println!("Ignored: {}", &path.display());
                        return None;
                    }
                    return Some((get_file_hash(&entry.path()), entry.path()));
                })
                .collect();
            return files;
        })
        .collect();

    bar.finish();
    bar.set_message(format!("✅ Finished indexing {} files", message_type));
    return indexed_files;
}

fn get_file_hash(path: &Path) -> u64 {
    // TODO: this should call parsers to work with exif, riff and quicktime
    let mut reader = StepableBuffReader::new(File::open(&path).unwrap());

    let exif_tags = vec![vec![0xFF, 0xD8, 0xFF, 0xE1], vec![0xFF, 0xD8, 0xFF, 0xE0]];
    if reader.compare_multiple_bytes(exif_tags) {
        while reader.increment() {
            if reader.compare_bytes(vec![0xFF, 0xDA]) {
                return xxh3_64(&reader.read_to_end());
            }
        }
        panic!("File did not have FF DA");
    }

    // println!("File without EXIf: {:?} {}", reader.peak(8), path.display());
    return xxh3_64(&reader.read_to_end());
}

fn process_files(
    files: &HashMap<u64, PathBuf>,
    config: &Config,
    existing_hashes: Option<&HashMap<u64, PathBuf>>,
) -> Vec<(Action, PathBuf, PathBuf)> {
    let new_files = existing_hashes.is_some();
    let file_type_msg = match new_files {
        true => "new",
        false => "existing",
    };
    let bar = ProgressBar::new_spinner();
    bar.set_style(
        ProgressStyle::default_spinner()
            .template(
                "{spinner} [{elapsed_precise}] {wide_msg} [{bar:20}] ({pos}/{len}, ETA {eta})",
            )
            .unwrap(),
    );
    bar.set_message(format!("  Processing {} files", file_type_msg));
    bar.set_length(files.len() as u64);

    let file_lookup: HashMap<PathBuf, u64> =
        files.iter().map(|(h, p)| (p.clone(), h.clone())).collect();
    let actions: Vec<(Action, PathBuf, PathBuf)> = files
        .par_iter()
        .progress_with(bar.clone())
        .filter_map(|(h, p)| {
            if new_files && existing_hashes.unwrap().contains_key(h) {
                return None;
            }
            let ext_option = p.extension().unwrap().to_ascii_lowercase();
            let ext = ext_option.to_str().unwrap();
            let result = match config.file_exts.get(ext).unwrap() {
                FileTypes::IMAGE => {
                    process_media(&config, MediaType::IMAGE, &p, ext, &file_lookup, new_files)
                }
                FileTypes::VIDEO => {
                    process_media(&config, MediaType::VIDEO, &p, ext, &file_lookup, new_files)
                }
                FileTypes::DOCUMENT => process_document(&config, &p, &file_lookup, new_files),
            };
            return result;
        })
        .collect();

    bar.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner} [{elapsed_precise}] {wide_msg}")
            .unwrap(),
    );
    bar.finish();
    bar.set_message(format!("✅ Finished processing {} files", file_type_msg));
    return actions;
}

fn process_media(
    config: &Config,
    media_type: MediaType,
    path: &PathBuf,
    ext: &str,
    file_lookup: &HashMap<PathBuf, u64>,
    new_files: bool,
) -> Option<(Action, PathBuf, PathBuf)> {
    if file_lookup.get(path).is_none() {
        if !new_files {
            return Some((Action::Delete, path.clone(), path.clone()));
        }
        return None;
    }
    let dt = read_datetime(&path);
    if dt.is_none() {
        // println!("OOPS! No datetime for {}", path.display()) // TODO: temporary
    }
    let file_prefix = match media_type {
        MediaType::IMAGE => "IMG",
        MediaType::VIDEO => "VID",
    };
    let file_name = match dt {
        Some(dt) => PathBuf::from(format!(
            "{}_{}.{}",
            file_prefix,
            dt.format("%Y%m%d_%H%M%S"),
            ext
        )),
        None => PathBuf::from(path.file_name().unwrap()),
    };
    let dest_dir = match media_type {
        MediaType::IMAGE if dt.is_some() => PathBuf::from(config.image_dir.clone()).join(file_name),
        MediaType::IMAGE => PathBuf::from(config.image_dir.clone() + "_temp").join(file_name),
        MediaType::VIDEO if dt.is_some() => PathBuf::from(config.video_dir.clone()).join(file_name),
        MediaType::VIDEO => PathBuf::from(config.video_dir.clone() + "_temp").join(file_name),
    };
    return match &dest_dir != path {
        true if new_files => Some((Action::Copy, path.clone(), dest_dir)),
        true => Some((Action::Move, path.clone(), dest_dir)),
        false => None,
    };
}

fn process_document(
    config: &Config,
    path: &PathBuf,
    file_lookup: &HashMap<PathBuf, u64>,
    new_files: bool,
) -> Option<(Action, PathBuf, PathBuf)> {
    if !new_files {
        return None;
    }
    let dest_path = PathBuf::from(config.doc_dir.clone()).join(path.file_name()?);
    return Some((Action::Copy, path.clone(), dest_path));
}

fn process_actions(actions: &Vec<(Action, PathBuf, PathBuf)>) {
    let bar = ProgressBar::new_spinner();
    bar.set_style(
        ProgressStyle::default_spinner()
            .template(
                "{spinner} [{elapsed_precise}] {wide_msg} [{bar:20}] ({pos}/{len}, ETA {eta})",
            )
            .unwrap(),
    );
    bar.set_message("Processing file changes");
    bar.set_length(actions.len() as u64);
    actions
        .par_iter()
        .progress_with(bar.clone())
        .for_each(|(act, src, dest)| match act {
            Action::Copy => copy_file(src, dest, true),
            Action::Move => copy_file(src, dest, false),
            Action::Delete => delete_file(src),
        });

    bar.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner} [{elapsed_precise}] {wide_msg}")
            .unwrap(),
    );
    bar.finish();
    bar.set_message("✅ Finished processing file changes");
}

fn copy_file(src: &PathBuf, dest: &PathBuf, copy: bool) {
    let mut new_path = dest.clone();
    let mut count = 1;
    while new_path.exists() {
        let new_file_name = format!("{}({})", dest.file_stem().unwrap().to_str().unwrap(), count);
        let new_file_name_with_ext = format!(
            "{}.{}",
            new_file_name,
            dest.extension().unwrap().to_str().unwrap()
        );
        new_path.set_file_name(new_file_name_with_ext);
        count += 1;
    }
    if copy {
        fs::copy(src, new_path).unwrap();
    } else {
        fs::rename(src, new_path).unwrap();
    }
}

fn delete_file(src: &PathBuf) {
    todo!()
}
