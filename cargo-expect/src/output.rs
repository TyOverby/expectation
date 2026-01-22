use colored::*;
use expectation_shared::{Result as EResult, ResultKind};
use std::io::Result as IoResult;

pub fn print_promotion(name: &str, results: Vec<(EResult, IoResult<String>)>, verbose: bool) -> (bool, usize) {
    let passed = results
        .iter()
        .all(|(_, r)|  r.is_ok());
    let nothing_done = results
        .iter()
        .all(|(r, _)|  match r.kind {
            ResultKind::Ok => true,
            _ => false,
        });
    let change_count = results
        .iter()
        .filter(|(r, _)| match r.kind {
            ResultKind::Ok => false,
            ResultKind::IoError(_) => false,
            _ => true,
        })
        .count();
    if nothing_done {
        return (passed, change_count);
    }

    if passed {
        println!("︎{} {}", "✔".green(), name);
    } else {
        println!("{} {}", "✘".red(), name);
    }

    for (EResult{file_name, ..}, io_result) in results {
        match io_result {
            Ok(detail) => {
                println!(
                    "  {} {}",
                    "✔".green(),
                    file_name.to_string_lossy()
                );
                if verbose {
                   println!("    ► {}", detail);
                }
            }
            Err(ioe) => {
                println!(
                    "  {} {} ❯ Error occurred during ",
                    "✘".red(),
                    file_name.to_string_lossy()
                );
                println!("    ► {}", ioe);
            }
        }
    }

    (passed, change_count)
}

pub fn print_results(name: &str, results: &[EResult], verbose: bool) {
    let passed = results.iter().all(|r| match r.kind {
        ResultKind::Ok => true,
        _ => false,
    });
    if passed {
        println!("︎{} {}", "✔".green(), name);
    } else {
        println!("{} {}", "✘".red(), name);
    }

    if passed && !verbose {
        return;
    }

    for result in results {
        match result {
            EResult {
                file_name,
                kind: ResultKind::Ok,
                ..
            } => {
                println!(
                    "  {}︎ {} ❯ Ok",
                    "✔".green(),
                    file_name.to_string_lossy()
                );
            }
            EResult {
                file_name,
                kind: ResultKind::ExpectedNotFound(double),
                ..
            } => {
                println!(
                    "  {} {} ❯ Expected Not Found",
                    "✘".red(),
                    file_name.to_string_lossy()
                );
                println!("    ► Actual: {}", double.actual.to_string_lossy());
                println!(
                    "    {} Expected: {}",
                    "☛".yellow(),
                    double.expected.to_string_lossy()
                );
            }
            EResult {
                file_name,
                kind: ResultKind::ActualNotFound(double),
                ..
            } => {
                println!(
                    "  {} {} ❯ Actual Not Found",
                    "✘".red(),
                    file_name.to_string_lossy()
                );
                println!(
                    "    {} Actual: {}",
                    "☛".yellow(),
                    double.actual.to_string_lossy()
                );
                println!("    ► Expected: {}", double.expected.to_string_lossy());
            }
            EResult {
                file_name,
                kind: ResultKind::Difference(tripple),
                ..
            } => {
                println!(
                    "  {} {} ❯ Difference",
                    "✘".red(),
                    file_name.to_string_lossy()
                );
                println!("    ► Actual: {}", tripple.actual.to_string_lossy());
                println!("    ► Expected: {}", tripple.expected.to_string_lossy());
                match tripple.diffs.len() {
                    0 => {}
                    1 => {
                        println!("    ► Diff: {}", tripple.diffs[0].to_string_lossy());
                    }
                    _ => {
                        println!("    ► Diffs:");
                        for diff in &tripple.diffs {
                            println!("      • {}", diff.to_string_lossy());
                        }
                    }
                }
            }
            EResult {
                file_name,
                kind: ResultKind::IoError(error),
                ..
            } => {
                println!(
                    "  {} Io Error for file {}: {}",
                    "✘".red(),
                    file_name.to_string_lossy(),
                    error
                );
            }
        }
    }
}
