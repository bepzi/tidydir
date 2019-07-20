// Local imports
use crate::error::*;

// External crate imports
use clap::crate_name;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

// Standard library imports
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use std::{fmt, fmt::Write as FmtWrite};
use std::{io, io::Write as IoWrite};

const DB_NAME: &str = "database.ron";

const SECS_PER_MIN: u64 = 60;
const SECS_PER_DAY: u64 = SECS_PER_MIN * 1440;
const STALE_THRESHOLD: Duration = Duration::from_secs(2 * SECS_PER_DAY);

/// A mapping of file/folder paths to the first time they were
/// encountered by the program.
#[derive(Serialize, Deserialize, Debug, Default)]
#[repr(transparent)]
pub struct Database {
    pub data: HashMap<PathBuf, SystemTime>,
}

impl Database {
    fn get_path() -> Result<PathBuf> {
        let project_dirs = ProjectDirs::from("", "", crate_name!()).ok_or(Error::InvalidHomeDir)?;
        let data_dir = project_dirs.data_local_dir();

        if let Err(io_err) = fs::create_dir_all(data_dir) {
            return Err(Error::DataDirCreation {
                path: data_dir.to_path_buf(),
                source: io_err,
            });
        }

        let mut path = data_dir.to_path_buf();
        path.push(DB_NAME);
        Ok(path)
    }

    /// Deserializes the database, or creates a new empty database if
    /// it doesn't already exist.
    pub fn open() -> Result<Self> {
        let path = Self::get_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }

        let maybe_db: Result<Self> = match fs::OpenOptions::new().read(true).open(&path) {
            Ok(file) => ron::de::from_reader(io::BufReader::new(file)).map_err(|de_err| {
                Error::DatabaseDeserialize {
                    source: de_err,
                    path,
                }
            }),
            Err(io_err) => Err(Error::DatabaseOpen {
                source: io_err,
                path,
            }),
        };

        // Wipe out any entries that no longer exist
        maybe_db.map(|db| Self {
            data: db
                .data
                .into_iter()
                .filter(|(path, _)| path.exists())
                .collect(),
        })
    }

    /// Serializes the database, overwriting the previous one.
    pub fn save(&self) -> Result<()> {
        let db_str = ron::ser::to_string(self)
            .map_err(|ser_err| Error::DatabaseSerialize { source: ser_err })?;

        let path = Self::get_path()?;

        let file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true) // Overwrites the old database
            .open(&path)
            .map_err(|io_err| Error::DatabaseSave {
                source: io_err,
                path: path.clone(),
            })?;

        let mut writer = io::BufWriter::new(file);
        writer
            .write_all(db_str.as_bytes())
            .map_err(|io_err| Error::DatabaseSave {
                source: io_err,
                path,
            })
    }

    pub fn get_stale_files(&self) -> StaleFiles<'_> {
        let now = SystemTime::now();
        let stale: HashMap<&'_ Path, Duration> = self
            .data
            .iter()
            .filter_map(|(path, last_seen)| {
                let since = now.duration_since(*last_seen).ok()?;
                if since >= STALE_THRESHOLD {
                    Some((path.as_ref(), since))
                } else {
                    None
                }
            })
            .collect();

        StaleFiles { data: stale }
    }
}

impl fmt::Display for Database {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: This representation might not be user-friendly enough
        write!(f, "{:#?}", self.data)
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct StaleFiles<'a> {
    data: HashMap<&'a Path, Duration>,
}

impl fmt::Display for StaleFiles<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Allocate space up front for the path strings plus the
        // formatted duration strings (assuming ~64 chars per line)
        let mut formatted = String::with_capacity(self.data.len() * 32 * 2);

        for (path, since) in &self.data {
            writeln!(
                &mut formatted,
                "\"{}\": {}",
                path.display(),
                fmt_duration(since)
            )?;
        }

        write!(f, "{}", formatted)
    }
}

fn fmt_duration(d: &Duration) -> String {
    let mut formatted = String::with_capacity(32);

    let mut seconds = d.as_secs();
    let days: u64 = seconds / SECS_PER_DAY;
    seconds -= days * SECS_PER_DAY;
    let minutes: u64 = seconds / SECS_PER_MIN;
    seconds -= minutes * SECS_PER_MIN;

    if days > 0 {
        write!(&mut formatted, "{} days, ", days).expect("write!() failed");
    }
    if minutes > 0 {
        write!(&mut formatted, "{} minutes, ", minutes).expect("write!() failed");
    }
    write!(&mut formatted, "{} seconds", seconds).expect("write!() failed");

    formatted
}
