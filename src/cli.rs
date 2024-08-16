// Copyright (C) 2024 Ethan Uppal. All rights reserved.

use argh::FromArgs;
use std::path::PathBuf;

/// Extracts smart documentation from an assembly project, given as a series of
/// files and folders.
#[derive(FromArgs)]
pub struct CLI {
    /// output directory for generated documentation
    #[argh(
        option,
        short = 'o',
        long = "output",
        default = "PathBuf::from(\"docs\")"
    )]
    pub out_dir: PathBuf,

    /// files or directories containing assembly code.
    #[argh(positional)]
    pub paths: Vec<PathBuf>
}

impl CLI {
    pub fn parse() -> Self {
        argh::from_env()
    }
}
