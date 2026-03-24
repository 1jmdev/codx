use std::path::PathBuf;

use clap::Parser;

use crate::app::{App, AppError};

#[derive(Debug, Parser)]
#[command(name = "codx", about = "Codx terminal editor MVP")]
struct Cli {
    path: Option<PathBuf>,
}

pub fn run() -> Result<(), AppError> {
    let cli = Cli::parse();
    let mut app = App::open(cli.path)?;
    app.run()
}
