//! CLI entrypoint for godot-dead-code-finder.

use clap::{CommandFactory, Parser};
use gdcf::cli::{run, Args};

fn main() {
    if std::env::args().len() == 1 {
        let mut cmd = Args::command();
        let _ = cmd.print_help();
        std::process::exit(0);
    }
    let exit_code = run(Args::parse());
    std::process::exit(exit_code);
}
