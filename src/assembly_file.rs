// Copyright (C) 2024 Ethan Uppal. All rights reserved

use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Display},
    path::{Path, PathBuf}
};

use serde::Serialize;

use crate::syntax::Syntax;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Serialize)]
pub enum AssemblySection {
    Text,
    Data,
    BSS,
    ROData
}

impl Display for AssemblySection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssemblySection::Text => "text",
            AssemblySection::Data => "data",
            AssemblySection::BSS => "bss",
            AssemblySection::ROData => "read-only data"
        }
        .fmt(f)
    }
}

#[derive(Debug, Serialize)]
pub enum AssemblyItem {
    Label(String),
    Mnemonic(),
    MacroCall(String, Vec<Box<AssemblyItem>>)
}

#[derive(Debug, Serialize)]
pub struct AssemblyMacro {
    pub name: String,
    pub arg_count: usize,
    pub body: Vec<AssemblyItem>
}

/// Assembly file representation optimized for documentation generation.
#[derive(Debug, Serialize)]
pub struct AssemblyFile {
    pub bits: usize,
    pub includes: Vec<PathBuf>,
    pub globals: HashSet<String>,
    pub externs: Vec<String>,
    pub macros: Vec<AssemblyMacro>,
    pub defines: Vec<String>,
    pub sections: HashMap<AssemblySection, Vec<AssemblyItem>>
}

impl Default for AssemblyFile {
    fn default() -> Self {
        Self {
            bits: 64,
            includes: Vec::new(),
            globals: HashSet::new(),
            externs: Vec::new(),
            macros: Vec::new(),
            defines: Vec::new(),
            sections: HashMap::new()
        }
    }
}

impl AssemblyFile {
    pub fn parse<'src, S: Syntax<'src>>(
        file: &'src Path, source: &'src str
    ) -> Result<Self, S::Error> {
        S::new_parser(file, source)?.parse()
    }
}
