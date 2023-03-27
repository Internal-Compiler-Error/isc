use std::fmt::Display;
use std::path::PathBuf;
use tokio::fs;

#[derive(Debug)]
pub struct CopyResult {
    src: PathBuf,
    dst: PathBuf,
    copy_result: std::io::Result<u64>,
}

#[allow(dead_code)]
impl CopyResult {
    pub fn is_ok(&self) -> bool {
        self.copy_result.is_ok()
    }

    pub fn is_err(&self) -> bool {
        self.copy_result.is_err()
    }

    pub fn new(src: PathBuf, dst: PathBuf, copy_result: std::io::Result<u64>) -> Self {
        CopyResult {
            src,
            dst,
            copy_result,
        }
    }
}

impl Display for CopyResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.copy_result {
            Ok(bytes) => write!(f, "Copied {bytes} bytes from {} to {}", self.src.display(), self.dst.display()),
            Err(e) => write!(f, "Failed to copy from {} to {}: {e}", self.src.display(), self.dst.display()),
        }
    }
}


/// Copy a file asynchronously, the function takes PathBufs so the entire operation can be spawned
pub async fn copy(src: PathBuf, dst: PathBuf) -> CopyResult {
    let copy_result = fs::copy(&src, &dst).await;

    CopyResult::new(src, dst, copy_result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_display() {
        let src = PathBuf::from("./");
        let dst = PathBuf::from("./");
        let copy_result = CopyResult::new(src, dst, Ok(0));

        assert_eq!(copy_result.to_string(), "Copied 0 bytes from ./ to ./");
    }
}