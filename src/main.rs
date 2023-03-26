use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::{File, ReadDir};
use std::io::Read;
use rayon::prelude::*;
use std::path::{PathBuf};
use std::sync::Mutex;
use clap::Parser;
use sha2::{Sha256, Digest};


#[derive(Parser, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[command(author, version, about, long_about = None)]
struct Args {
    source: PathBuf,
    destination: Option<PathBuf>,
}

fn file_sha256(file: &mut File) -> color_eyre::Result<[u8; 32]> {
    let mut hasher = Sha256::new();
    let mut buffer = [0; 65536]; // 2^16

    while let Ok(len) = file.read(&mut buffer) {
        hasher.update(&buffer[..len]);
    }

    Ok(hasher.finalize().into())
}


fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let args = Args::parse();
    println!("{:?}", args);


    let source_dir = args.source;
    // if destination is not provided, use current directory
    let destination_dir = args.destination.unwrap_or(PathBuf::from(r#"./"#));


    // for each file in source, calculate hash and store in hashmap
    let (source_hashes, destination_hashes) = rayon::join(
        || extract_file_to_hash(fs::read_dir(source_dir.clone()).unwrap()),
        || extract_file_hash(fs::read_dir(destination_dir.clone()).unwrap()));

    // for each file in source, check if it exists in destination, only copy if it doesn't
    source_hashes
        .par_iter()
        // if the hash of the file in source is in the destination, then it exists, so don't copy
        .filter(|(_, hash)| {
            !destination_hashes.contains(*hash)
        })
        .for_each(|(file, _)| {
            fs::copy(file, destination_dir.join(file.file_name().unwrap())).unwrap();
        });


    Ok(())
}


fn extract_file_hash(entries: ReadDir) -> HashSet<[u8; 32]> {
    let hash_sets = Mutex::new(HashSet::new());


    let file_hashes = entries
        .par_bridge()
        // map each entry to PathBuf, panic if it's a directory
        .map(|entry| {
            let entry = entry.expect("Failed to read entry");
            let path = entry.path();
            if path.is_dir() {
                panic!("ISC only supports if the directory only contains files");
            }
            entry.path()
        })
        .map(|path| {
            let mut file = File::open(path.clone()).unwrap();
            file_sha256(&mut file).unwrap()
        });

    file_hashes.for_each(|hash| {
        hash_sets
            .lock()
            .unwrap() //  I'm been told by a Tokio developer that mutex poisoning is dumb anyway
            .insert(hash);
    });


    hash_sets.into_inner().unwrap()
}

fn extract_file_to_hash(entries: ReadDir) -> HashMap<PathBuf, [u8; 32]> {
    let filename_to_hash = Mutex::new(HashMap::new());

    // map each entry to PathBuf, panic if it's a directory
    let file_hashes = entries
        .par_bridge()
        .map(|entry| {
            let entry = entry.expect("Failed to read entry");
            let path = entry.path();
            if path.is_dir() {
                panic!("ISC only supports if the directory only contains files");
            }
            entry.path()
        })
        .map(|path| {
            let mut file = File::open(path.clone()).unwrap();
            let hash = file_sha256(&mut file).unwrap();
            (path, hash)
        });

    file_hashes.for_each(|(path, hash)| {
        filename_to_hash
            .lock()
            .unwrap() //  I'm been told by a Tokio developer that mutex poisoning is dumb anyway
            .insert(path, hash);
    });


    filename_to_hash.into_inner().unwrap()
}
