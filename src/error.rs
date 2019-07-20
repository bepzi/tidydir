// External crate imports
use snafu::Snafu;

// Standard library imports
use std::io;
use std::path::PathBuf;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Couldn't find a valid $HOME directory"))]
    InvalidHomeDir,

    #[snafu(display("Couldn't deserialize the database file from {}: {}", path.display(), source))]
    DatabaseDeserialize {
        source: ron::de::Error,
        path: PathBuf,
    },

    #[snafu(display("Couldn't serialize the database instance: {}", source))]
    DatabaseSerialize { source: ron::ser::Error },

    #[snafu(display("Couldn't open the database file at {}: {}", path.display(), source))]
    DatabaseOpen { source: io::Error, path: PathBuf },

    #[snafu(display("Couldn't save the database file to {}: {}", path.display(), source))]
    DatabaseSave { source: io::Error, path: PathBuf },

    #[snafu(display("Couldn't create data directory {}: {}", path.display(), source))]
    DataDirCreation { path: PathBuf, source: io::Error },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
