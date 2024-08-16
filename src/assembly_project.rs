// Copyright (C) 2024 Ethan Uppal. All  rights reserved.

use std::{collections::HashMap, path::PathBuf};

use crate::assembly_file::AssemblyFile;

pub struct AssemblyProject {
    files: HashMap<PathBuf, AssemblyFile>
}

impl AssemblyProject {
    pub fn from(files: HashMap<PathBuf, AssemblyFile>) -> Self {
        Self { files }
    }
}
