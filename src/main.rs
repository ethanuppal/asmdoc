// Copyright (C) 2024 Ethan Uppal. All rights reserved.

use std::{
    collections::HashMap,
    ffi, fs,
    path::{Path, PathBuf}
};

use asmdoc::{
    assembly_file::AssemblyFile, assembly_project::AssemblyProject, cli::CLI,
    syntax
};
use walkdir::WalkDir;

fn can_parse(path: &Path) -> bool {
    path.is_file()
        && ["nasm", "asm"].contains(
            &path.extension().and_then(ffi::OsStr::to_str).unwrap_or("")
        )
}

fn parse_file(
    store: &mut HashMap<PathBuf, AssemblyFile>, path: &Path
) -> anyhow::Result<()> {
    let source = fs::read(path)?;
    let source = String::from_utf8(source)?; // and_then won't work
    store.insert(
        path.to_owned(),
        AssemblyFile::parse::<syntax::NASM>(&source)?
    );
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = CLI::parse();

    let mut files = HashMap::new();
    for path in &args.paths {
        if can_parse(path) {
            parse_file(&mut files, path)?;
        } else if path.is_dir() {
            for file in WalkDir::new(path).into_iter().flatten() {
                if can_parse(file.path()) {
                    parse_file(&mut files, file.path())?;
                }
            }
        }
    }

    // let mut output_toml = toml::Table::new();
    // for (file, asm) in store {
    //     output_toml.insert(
    //         file.to_string_lossy().to_string(),
    //         toml::Value::try_from(&asm).unwrap()
    //     );
    // }
    // println!("{}", toml::to_string_pretty(&output_toml).unwrap());

    let project = AssemblyProject::from(files);

    Ok(())
}
