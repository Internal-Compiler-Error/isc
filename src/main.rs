use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::{File, ReadDir};
use std::io::{BufReader, Read};
use rayon::prelude::*;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use clap::Parser;
use futures::stream::FuturesOrdered;
use sha2::{Digest, Sha256};
use configuration::Args;
use crate::configuration::build_tokio_runtime;

use tokio::io::AsyncReadExt;

mod file_copy;
mod join_all;
mod report;
mod configuration;
mod checksum;

use tokio::sync::Notify;
use crate::report::Report;


async fn sha256(file: tokio::fs::File) -> color_eyre::Result<[u8; 32]> {
    // many painful hours were spent trying to avoid locking, I've failed...
    // If you can make this lock free by passing the hasher and buffer back and forth, please do

    let hasher = Arc::new(Mutex::new(Sha256::new()));
    let buffer = Arc::new(Mutex::new(Box::new([0; 65536]))); // 2^16

    let mut reader = tokio::io::BufReader::new(file);
    let hash_done = Arc::new(Notify::new());

    loop {
        hash_done.notified().await;
        {
            let mut buffer = buffer.lock().unwrap();
            let len = reader.read(buffer.as_mut_slice()).await?;

            if len == 0 {
                break;
            }
        }

        let hasher_clone = hasher.clone();
        let buffer_clone = buffer.clone();
        let hash_done_clone = hash_done.clone();
        rayon::spawn(move || {
            let mut hasher = hasher_clone.lock().unwrap();
            let buffer = buffer_clone.lock().unwrap();
            hasher.update(&buffer[..]);
            hash_done_clone.notify_one();
        });
    }


    // I've suffered too much to get this compile
    let digest = Arc::try_unwrap(hasher)
        .unwrap()
        .into_inner()?
        .finalize();

    Ok(digest.into())
}

fn file_sha256(file: &mut File) -> color_eyre::Result<[u8; 32]> {
    let mut hasher = Sha256::new();
    let mut buffer = [0; 65536]; // 2^16

    let mut reader = BufReader::new(file);
    while let Ok(len) = reader.read(&mut buffer) {
        // I am dumb, turns out EOF is not an error
        if len == 0 {
            break;
        }

        hasher.update(&buffer[..len]);
    }

    Ok(hasher.finalize().into())
}


fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let source_dir = args.source;
    let destination_dir = args.destination;


    // for each file in source, calculate hash and store in hashmap
    let (source_hashes, destination_hashes) = rayon::join(
        || extract_file_to_hash(fs::read_dir(source_dir.clone()).unwrap()),
        || extract_file_hash(fs::read_dir(destination_dir.clone()).unwrap()));


    // find out what files need to be copied
    let should_copy = source_hashes
        .par_iter()
        .filter(|(_, hash)| !destination_hashes.contains(*hash));

    // for those files, that need to copy, construct a list of source and destination paths
    let copy_tasks = should_copy
        .map(|(source, _)| {
            let destination = destination_dir.join(source.file_name().unwrap());
            (source.clone(), destination)
        });

    // for each copy task, spawn a task to copy the file
    let tokio_runtime = build_tokio_runtime(args.thread_count)?;
    let runtime_handle = tokio_runtime.handle();

    let futures = Mutex::new(FuturesOrdered::new());
    copy_tasks
        .for_each(|(source, destination)| {
            futures
                .lock()
                .unwrap()
                .push_back(runtime_handle.spawn(file_copy::copy(source, destination)));
        });
    let futures = futures.into_inner().unwrap();


    // wait for all copy operations to complete
    let copy_reports = join_all::join_all_to_vec(futures, &runtime_handle);

    // print a report
    println!("{}", Report(&copy_reports));

    Ok(())
}


fn extract_file_hash(entries: ReadDir) -> HashSet<[u8; 32]> {
    let hash_sets = Mutex::new(HashSet::new());


    let file_hashes = entries
        .par_bridge()
        .filter_map(|entry| {
            let entry = entry.expect("Failed to read entry");
            let path = entry.path();
            if path.is_dir() { None } else { Some(entry.path()) }
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
        // ignore directories for now
        .filter_map(|entry| {
            let entry = entry.expect("Failed to read entry");
            let path = entry.path();
            if path.is_dir() { None } else { Some(entry.path()) }
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
