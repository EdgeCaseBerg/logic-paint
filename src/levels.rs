use crate::netbpm::{LoadPbmErr, Pbm};
use crate::netppm::{LoadPpmErr, Ppm};

use std::fs::{read_dir, read_to_string, write};
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Level {
    pub info: Pbm,
    pub image: Ppm,
    pub completed: bool,
    pub path: PathBuf,
}

impl Level {
    pub fn mark_completed(&mut self) -> std::io::Result<()> {
        self.completed = true;
        write(&self.path, "1")
    }
}

#[derive(Debug)]
pub enum LevelLoadError {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    ParsePbm {
        path: PathBuf,
        source: LoadPbmErr,
    },
    ParsePpm {
        path: PathBuf,
        source: LoadPpmErr,
    },
    InvalidDirectory(PathBuf),
}

pub type LevelsLoadResult<T> = Result<T, LevelLoadError>;

impl std::fmt::Display for LevelLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LevelLoadError::Io { path, source } => {
                write!(f, "could not read {:?}: {}", path, source)
            }
            LevelLoadError::ParsePbm { path, source } => {
                write!(f, "could not parse PBM {:?}: {}", path, source)
            }
            LevelLoadError::ParsePpm { path, source } => {
                write!(f, "could not parse PPM {:?}: {}", path, source)
            }
            LevelLoadError::InvalidDirectory(path) => {
                write!(f, "{:?} is not a directory", path)
            }
        }
    }
}

impl std::error::Error for LevelLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LevelLoadError::Io { source, .. } => Some(source),
            LevelLoadError::ParsePbm { source, .. } => Some(source),
            LevelLoadError::ParsePpm { source, .. } => Some(source),
            LevelLoadError::InvalidDirectory(_) => None,
        }
    }
}

pub fn load_levels_from_dir(dir: &Path) -> LevelsLoadResult<Vec<Level>> {
    if !dir.is_dir() {
        return Err(LevelLoadError::InvalidDirectory(dir.to_path_buf()));
    }

    let mut level_files: Vec<_> = read_dir(dir)
        .map_err(|e| LevelLoadError::Io {
            path: dir.to_path_buf(),
            source: e,
        })?
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            path.extension()
                .is_some_and(|ext| ext == "level")
                .then_some(path)
        })
        .collect();

    level_files.sort();

    level_files
        .into_iter()
        .map(|level_file| -> LevelsLoadResult<Level> {
            let contents = read_to_string(&level_file).map_err(|e| LevelLoadError::Io {
                path: level_file.clone(),
                source: e,
            })?;

            let completed = contents.trim() == "1";

            let pbm_path = level_file.with_extension("pbm");
            let ppm_path = level_file.with_extension("ppm");

            let pbm: Pbm = read_to_string(&pbm_path)
                .map_err(|e| LevelLoadError::Io {
                    path: pbm_path.clone(),
                    source: e,
                })?
                .parse()
                .map_err(|e| LevelLoadError::ParsePbm {
                    path: pbm_path.clone(),
                    source: e,
                })?;

            let ppm: Ppm = read_to_string(&ppm_path)
                .map_err(|e| LevelLoadError::Io {
                    path: ppm_path.clone(),
                    source: e,
                })?
                .parse()
                .map_err(|e| LevelLoadError::ParsePpm {
                    path: ppm_path.clone(),
                    source: e,
                })?;

            Ok(Level {
                info: pbm,
                image: ppm,
                completed,
                path: level_file.clone(),
            })
        })
        .collect()
}
