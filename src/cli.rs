// Copyright (C) 2024 Ethan Uppal. All rights reserved.

use argh::FromArgs;
use std::path::PathBuf;

/// Extracts smart documentation from an assembly project.
#[derive(FromArgs)]
pub struct CLI {
    /// A file or directory containing assembly code.
    #[argh(positional)]
    pub paths: Vec<PathBuf>
}

impl CLI {
    pub fn parse() -> Self {
        argh::from_env()
    }
}
