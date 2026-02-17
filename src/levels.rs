use crate::netbpm;
use crate::netppm;

use crate::netbpm::{LoadPbmErr, Pbm};
use crate::netppm::{LoadPpmErr, Ppm};

use std::ffi::OsStr;
use std::fs::{read_dir, read_to_string};
use std::path::Path;

#[derive(Debug)]
pub struct Level {
    pub info: Pbm,
    pub image: Ppm,
    pub completed: bool,
}

#[derive(Debug)]
pub enum LevelLoadError {
    Io(std::io::Error),
    Pbm(LoadPbmErr),
    Ppm(LoadPpmErr),
    InvalidDirectory,
}

impl From<std::io::Error> for LevelLoadError {
    fn from(e: std::io::Error) -> Self {
        LevelLoadError::Io(e)
    }
}

impl From<LoadPbmErr> for LevelLoadError {
    fn from(e: LoadPbmErr) -> Self {
        LevelLoadError::Pbm(e)
    }
}

impl From<LoadPpmErr> for LevelLoadError {
    fn from(e: LoadPpmErr) -> Self {
        LevelLoadError::Ppm(e)
    }
}

pub fn load_levels_from_dir(dir: &Path) -> Vec<Level> {
    if !dir.is_dir() {
        return vec![];
    }

    let dir_iter = read_dir(dir);
    if let Err(problem) = dir_iter {
        panic!("could not load levels {}", problem);
    }
    let dir_iter = dir_iter.unwrap();
    let mut level_files = Vec::new();
    for entry in dir_iter {
        if let Ok(entry) = entry {
            let path = entry.path();
            match path.extension() {
                Some(os_string) => {
                    if os_string.to_string_lossy() == "level" {
                        level_files.push(path);
                    }
                }
                _ => {}
            };
        }
    }
    level_files.sort();

    let mut levels = Vec::new();
    for level_file in level_files {
        match read_to_string(&level_file) {
            Err(problem) => {
                panic!("could not load level {:?} {:?}", &level_file, problem);
            }
            Ok(contents) => {
                // contents are completed 0 or 1, expected to be paired with pbm and ppm file of same name
                let ppm_file_path = level_file.with_extension("ppm");
                let pbm_file_path = level_file.with_extension("pbm");

                let completed = contents.trim() == "1";

                let pbm = read_to_string(&pbm_file_path)
                    .expect(&format!("could not read level file {:?}", &pbm_file_path));
                let pbm: Pbm = pbm
                    .parse()
                    .expect(&format!("could not parse level file {:?}", &pbm_file_path));

                let ppm = read_to_string(&ppm_file_path)
                    .expect(&format!("could not read level file {:?}", &ppm_file_path));
                let ppm: Ppm = ppm
                    .parse()
                    .expect(&format!("could not parse level file {:?}", &ppm_file_path));

                levels.push(Level {
                    info: pbm,
                    image: ppm,
                    completed: completed,
                });
            }
        }
    }

    levels
}
