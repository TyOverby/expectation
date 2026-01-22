extern crate structopt;
extern crate expectation_shared;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate crossbeam;
extern crate colored;

use std::io::Result as IoResult;
use structopt::StructOpt;
mod command;
mod output;
mod promote;

#[derive(StructOpt, Debug)]
pub struct Specifier {
    /// Specifies which tests to run or promote
    #[structopt(name = "filter")]
    filter: Option<String>,

    /// Filetypes is a filter for which kinds of files are considered
    /// when running tests and promoting results.
    #[structopt(short = "f", long = "filetypes")]
    filetypes: Vec<String>,

    #[structopt(short = "v", long = "verbose")]
    verbose: bool,

    #[structopt(long = "release")]
    release: bool,
}

#[derive(StructOpt, Debug)]
#[structopt(
    about = r#"EXAMPLES:
    cargo expect run                     # runs all tests
    cargo expect run -f svg              # runs all tests but only diffs svg files
    cargo expect run my_test_name        # uses "my_test_name" as a filter for running tests
    cargo expect run my_test_name -f svg # uses "my_test_name" as a filter for running tests but only diffs svg files

    cargo expect promote                      # promotes all tests with all files
    cargo expect promote -f svg               # promotes all tests but only promotes svg files produced by those tests
    cargo expect promote my_test_name         # promotes all files in tests that match "my_test_name"
    cargo expect promote my_test_name -f svg  # promotes only svg files for tests that match "my_test_name"
"#
)]
pub enum Command {
    /// Runs expectation tests in this crate
    #[structopt(name = "run")]
    Run(Specifier),

    /// Promotes the "actual" files to "expected" files
    #[structopt(name = "promote")]
    Promote(Specifier),

    /// Cleans up the expectation-tests directory by removing the "diff" and "actual" folders.
    #[structopt(name = "clean")]
    Clean,
}

fn main() -> IoResult<()> {
    let mut args: Vec<_> = ::std::env::args_os().collect();
    if args.len() >= 2 && args[1] == "expect" {
        args.remove(1);
    }

    let c = Command::from_iter(args);
    match c {
        Command::Promote(spec) => {
            let good = command::perform_promote(spec)?;
            if !good {
                ::std::process::exit(1);
            }
        }
        Command::Run(spec) => {
            let good = command::perform_run(spec)?;
            if !good {
                ::std::process::exit(1);
            }
        }
        _ => panic!(),
    }
    Ok(())
}
