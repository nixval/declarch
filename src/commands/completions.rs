use crate::cli::args::Cli;
use crate::error::Result;
use clap::CommandFactory;
use clap_complete::{Shell, generate};
use std::io;

pub fn run(shell: Shell) -> Result<()> {
    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();

    generate(shell, &mut cmd, bin_name, &mut io::stdout());

    Ok(())
}
