// Copyright (C) 2024 Ethan Uppal. All  rights reserved.

use std::{collections::HashMap, path::PathBuf};

use crate::assembly_file::AssemblyFile;

#[derive(Default)]
pub struct AssemblyProject {
    files: HashMap<PathBuf, AssemblyFile>,
    globals: HashMap<String, PathBuf>,
    internal_externs: HashMap<String, PathBuf>
}

impl AssemblyProject {
    pub fn build_from(files: HashMap<PathBuf, AssemblyFile>) -> Self {
        Self {
            files,
            ..Default::default()
        }
        .resolve()
    }

    fn resolve(mut self) -> Self {
        for (file, asm) in &self.files {
            for global in &asm.globals {
                self.globals.insert(global.clone(), file.clone());
            }
        }
        for asm in self.files.values() {
            for extern_ in &asm.externs {
                if let Some(global_def_file) = self.globals.get(extern_) {
                    self.internal_externs
                        .insert(extern_.clone(), global_def_file.clone());
                }
            }
        }
        self
    }
}
