#![allow(unused, dead_code)]
/// This entire file is full of hacks, some Rust language lawyer should probably look at it
use std::fs::File;
use std::io::{BufReader, Read};
use sha2::{Digest};
use sha2::Sha512 as sha2_sha512;
use sha2::Sha256 as sha2_sha256;
use sha3::Sha3_512 as sha3_sha512;
use sha3::Sha3_256 as sha3_sha256;

// somehow this is enough for sha3 also, probably because it's re-exported or something
use sha2::digest::OutputSizeUser;
use sha2::digest::typenum::Unsigned;


/// Generate a function that calculates the hash of a file, yes this is kinda stupid
macro_rules! generate_hash_fn {
    ($fn_name:ident, $hasher:ident) => {
        pub fn $fn_name(file: &mut File) -> color_eyre::Result<[u8; <$hasher as OutputSizeUser>::OutputSize::USIZE]> {
            let mut hasher = $hasher::new();
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
    };
}

generate_hash_fn!(file_sha2_256, sha2_sha256);
generate_hash_fn!(file_sha2_512, sha2_sha512);
generate_hash_fn!(file_sha3_256, sha3_sha256);
generate_hash_fn!(file_sha3_512, sha3_sha512);
