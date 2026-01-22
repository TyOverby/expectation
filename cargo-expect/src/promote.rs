use expectation_shared::ResultKind;
use expectation_shared::filesystem::FileSystem;
use std::io::Result as IoResult;

pub fn promote(result: &ResultKind, filesystem: Box<dyn FileSystem>) -> IoResult<String> {
    match result {
        ResultKind::IoError(_) |
        ResultKind::Ok => Ok("Nothing to do".into()),
        ResultKind::ExpectedNotFound(double) => {
            filesystem.copy(&double.actual, &double.expected)?;
            Ok(format!("moved {} -> {}", double.actual.to_string_lossy(),
                                         double.expected.to_string_lossy()))
        }
        ResultKind::ActualNotFound(double) => {
            filesystem.remove(&double.expected)?;
            Ok(format!("removed {}", double.expected.to_string_lossy()))
        }
        ResultKind::Difference(triple) => {
            filesystem.copy(&triple.actual, &triple.expected)?;
            Ok(format!("moved {} -> {}", triple.actual.to_string_lossy(),
                                         triple.expected.to_string_lossy()))
        }

    }
}