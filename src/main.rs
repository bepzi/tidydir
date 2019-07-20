#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(unused)]
#![warn(unused_qualifications)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(single_use_lifetimes)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(nonstandard_style)]
#![warn(elided_lifetimes_in_paths)]
#![deny(rust_2018_idioms)]
#![deny(unsafe_code)]
// #![warn(missing_docs)]

// Local modules
mod database;
mod error;

// Local imports
use database::*;
use error::*;

// External crate imports
use clap::{
    app_from_crate, crate_authors, crate_description, crate_name, crate_version, Arg, SubCommand,
};

// Standard library imports
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

const SUBCMD_TRACK: &str = "track";
const SUBCMD_UNTRACK: &str = "untrack";
const SUBCMD_LIST: &str = "list";

const ARG_SECS: &str = "secs";
const ARG_PATHS: &str = "paths";
const ARG_ALL: &str = "all";

const DEFAULT_STALE_THRESHOLD: &str = "172800"; // 2 days

fn run() -> Result<()> {
    let matches = app_from_crate!()
        .arg(
            Arg::with_name(ARG_SECS)
                .short(ARG_SECS.get(0..1).unwrap())
                .help("Seconds until a file is considered \"stale\"")
                .takes_value(true)
                .value_name(&ARG_SECS.to_ascii_uppercase())
                .default_value(&DEFAULT_STALE_THRESHOLD),
        )
        .subcommand(
            SubCommand::with_name(SUBCMD_TRACK)
                .about("Registers files and folders")
                .arg(
                    Arg::with_name(ARG_PATHS)
                        .help("List of files and folders to track")
                        .takes_value(true)
                        .value_name(&ARG_PATHS.to_ascii_uppercase())
                        .required(true)
                        .multiple(true),
                ),
        )
        .subcommand(
            SubCommand::with_name(SUBCMD_UNTRACK)
                .about("Unregisters files and folders")
                .arg(
                    Arg::with_name(ARG_ALL)
                        .long(ARG_ALL)
                        .help("Unregisters all files and folders")
                        .takes_value(false),
                )
                .arg(
                    Arg::with_name(ARG_PATHS)
                        .help("List of files and folders to stop tracking")
                        .takes_value(true)
                        .value_name(&ARG_PATHS.to_ascii_uppercase())
                        .required_unless(ARG_ALL)
                        .multiple(true),
                ),
        )
        .subcommand(
            SubCommand::with_name(SUBCMD_LIST)
                .about("Lists all currently tracked files and folders"),
        )
        .get_matches();

    let mut db = Database::open()?;

    if let Some(subcmd_matches) = matches.subcommand_matches(SUBCMD_TRACK) {
        let paths = subcmd_matches
            .values_of_os(ARG_PATHS)
            .unwrap() // Should be a safe unwrap
            .map(PathBuf::from);

        // Attempt to canonicalize the given paths, but also keep the
        // originals so we can tell the user when something goes wrong
        let (valid, invalid): (
            Vec<(std::io::Result<PathBuf>, PathBuf)>,
            Vec<(std::io::Result<PathBuf>, PathBuf)>,
        ) = paths
            .map(|path| (path.canonicalize(), path))
            .partition(|(result, _)| result.is_ok());

        if !invalid.is_empty() {
            println!("Ignoring invalid paths:");
            for (path, _) in invalid {
                println!(" - {:?}", path);
            }
        }

        let paths = valid.into_iter().filter_map(|(result, _)| result.ok());

        let sys_time = SystemTime::now();
        for path in paths {
            db.data.entry(path).or_insert(sys_time);
        }
    } else if let Some(subcmd_matches) = matches.subcommand_matches(SUBCMD_UNTRACK) {
        if subcmd_matches.is_present(ARG_ALL) {
            db = Database::default();
        } else {
            // --all flag was not present, remove only the given paths

            let paths = subcmd_matches
                .values_of_os(ARG_PATHS)
                .unwrap() // Should be a safe unwrap
                .map(PathBuf::from);

            // Attempt to canonicalize the given paths, but also keep the
            // originals so we can tell the user when something goes wrong
            let (valid, invalid): (
                Vec<(std::io::Result<PathBuf>, PathBuf)>,
                Vec<(std::io::Result<PathBuf>, PathBuf)>,
            ) = paths
                .map(|path| (path.canonicalize(), path))
                .partition(|(result, _)| result.is_ok());

            if !invalid.is_empty() {
                println!("Ignoring invalid paths:");
                for (path, _) in invalid {
                    println!(" - {:?}", path);
                }
            }

            let paths = valid.into_iter().filter_map(|(result, _)| result.ok());
            for path in paths {
                db.data.remove(&path);
            }
        }
    } else if matches.subcommand_matches(SUBCMD_LIST).is_some() {
        println!("{}", db);
    } else {
        // No subcommand; default action is to print any stale files
        match matches.value_of(ARG_SECS).unwrap().parse::<u64>() {
            Ok(secs) => print!("{}", db.get_stale_files(&Duration::from_secs(secs))),
            Err(parse_err) => {
                return Err(Error::U64Parse {
                    source: parse_err,
                    arg: matches.value_of(ARG_SECS).unwrap().to_string(),
                })
            }
        }
    }

    db.save()?;
    Ok(())
}

fn main() -> Result<()> {
    ::std::process::exit(match run() {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("{}", e);
            1
        }
    })
}
