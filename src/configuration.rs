use std::fs::File;
use std::num::NonZeroUsize;
use clap::{Parser, ValueEnum};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::runtime::Runtime;
use crate::checksum;


#[derive(Parser, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[command(name = "isc")]
#[rustfmt::skip]
#[command(about = "Selectively copy files from source to destination directory using their sha256 checksums as the equality criteria")]
#[command(long_about)]
/// Intelligently Selective Copy (isc) is a⚡blazingly fast⚡(sorry the meme had to be done) cli tool
/// that copies all the files from source to destination directory, but only those files that are
/// not present in the destination directory. The equality of files is determined by their sha256
/// checksums. The tool computes the checksums of the files in parallel, then copies the ones that
/// need to copy in parallel. Since NVMe SSDs support parallel reads and writes, this is much faster
/// than doing the operations one by one.
pub struct Args {
    /// The source directory to copy from
    pub source: PathBuf,
    /// The destination directory to copy to, if not provided, the current directory will be used
    #[clap(default_value_os_t = PathBuf::from(r#"./"#))]
    pub destination: PathBuf,
    #[clap(short, long, default_value_t = default_thread_count())]
    pub thread_count: usize,

    #[clap(short, long, value_enum, default_value_t = Algorithm::SHA2_256)]
    pub algorithm: Algorithm,
}

#[derive(Debug, PartialEq, Eq, Hash, Ord, PartialOrd, Clone, Copy, ValueEnum)]
pub enum Algorithm {
    SHA2_256,
    SHA2_512,
    SHA3_256,
    SHA3_512,
}
#[allow(unused)]
impl Algorithm {
    pub const fn digest_size(&self) -> usize {
        match self {
            Algorithm::SHA2_256 => 32,
            Algorithm::SHA2_512 => 64,
            Algorithm::SHA3_256 => 32,
            Algorithm::SHA3_512 => 64,
        }
    }
}
#[allow(unused)]
/// returns the function that implements the algorithm
pub fn checksum_impl(algorithm: &Algorithm) -> impl for<'a> Fn(&'a mut File) -> color_eyre::Result<Box<[u8]>>
{
    // removing the explicit argument type will fail to compile, I don't know why
    match algorithm {
        Algorithm::SHA2_256 => |file: &mut File| {
            let digest = checksum::file_sha2_256(file)?;
            Ok(Box::from(digest))
        },
        Algorithm::SHA2_512 => |file: &mut File| {
            let digest = checksum::file_sha2_512(file)?;
            Ok(Box::from(digest))
        },
        Algorithm::SHA3_256 => |file: &mut File| {
            let digest = checksum::file_sha3_256(file)?;
            Ok(Box::from(digest))
        },
        Algorithm::SHA3_512 => |file: &mut File| {
            let digest = checksum::file_sha3_512(file)?;
            Ok(Box::from(digest))
        },
    }
}

fn default_thread_count() -> usize {
    /// Safety: I'm a normal person who works with ZFC and the regular sets of numbers, get anything
    /// fancier than complex numbers out of here.
    const ONE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(1) };
    std::thread::available_parallelism().unwrap_or(ONE).get()
}

pub fn build_tokio_runtime(thread_count: usize) -> color_eyre::Result<Runtime> {
    use tokio::runtime::Builder;

    Builder::new_multi_thread()
        .enable_all()
        .worker_threads(thread_count)
        .thread_name_fn(|| {
            // copied from the official tokio docs
            static ATOMIC_ID: AtomicUsize = AtomicUsize::new(0);
            let id = ATOMIC_ID.fetch_add(1, Ordering::SeqCst);
            format!("isc tokio runtime {}", id)
        })
        .build()
        .map_err(|e| e.into())
}