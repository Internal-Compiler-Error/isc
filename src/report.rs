use crate::file_copy::CopyResult;
use std::fmt::{Display, Formatter};
use std::fmt;

use tokio::task::JoinError;

/// Basically the Display trait, exists to get around the orphan rule. You implement this on a
/// foreign type and construct a `Report` from the foreign type. Since Report implements Display if
/// the foreign type implements OperationReport, then the foreign type can be printed.
pub trait OperationReport {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result;
}

impl<T> OperationReport for &T
    where T: OperationReport
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        OperationReport::fmt(*self, f)
    }
}

// not sure if this is actually needed
impl<T> OperationReport for &mut T
    where T: OperationReport
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        OperationReport::fmt(*self, f)
    }
}

/// Exists to get around the orphan rule
/// Usage:
/// ```
/// println!("{}", Report(&T));
/// ```
pub struct Report<T>(pub T);

impl<T> From<T> for Report<T>
    where T: OperationReport
{
    fn from(t: T) -> Self {
        Report(t)
    }
}

impl<T> Display for Report<T>
    where T: OperationReport
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        OperationReport::fmt(&self.0, f)
    }
}

impl OperationReport for CopyResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl OperationReport for Result<CopyResult, JoinError> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Ok(copy_result) => Display::fmt(copy_result, f),
            Err(join_error) => Display::fmt(join_error, f),
        }
    }
}

impl OperationReport for Vec<Result<CopyResult, JoinError>> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // count the number of failed joins
        let failed_count = self
            .iter()
            .filter(|join_result| join_result.is_err())
            .count();

        // count the number of successful joins
        let success_count = self.len() - failed_count;

        // write the number of successful and failed joins
        writeln!(f, "{success_count} files copied successfully; {failed_count} files failed to copy")?;

        // write the result of each join
        for join_result in self {
            writeln!(f, "{}", Report(join_result))?;
        }

        Ok(())
    }
}

impl OperationReport for Vec<CopyResult> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let failed_count = self
            .iter()
            .filter(|copy_result| copy_result.is_err())
            .count();

        let success_count = self.len() - failed_count;

        writeln!(f, "{success_count} files copied successfully; {failed_count} files failed to copy")?;
        for copy_result in self {
            writeln!(f, "{copy_result}")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_print_vec_of_copy_results() {
        let copy_results: Vec<Result<CopyResult, JoinError>> = vec![
            Ok(CopyResult::new("./".into(), "./".into(), Ok(0))),
            Ok(CopyResult::new("./".into(), "./".into(), Ok(0))),
        ];

        println!("{}", Report(&copy_results));

        let expected = "2 files copied successfully; 0 files failed to copy\n\
                             Copied 0 bytes from ./ to ./\n\
                             Copied 0 bytes from ./ to ./\n";

        assert_eq!(expected, format!("{}", Report(&copy_results)));
    }
}