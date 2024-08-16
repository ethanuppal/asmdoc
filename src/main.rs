// Copyright (C) 2024 Ethan Uppal. All rights reserved.

use std::{
    collections::HashMap,
    ffi, fs,
    path::{Path, PathBuf}
};

use asmdoc::{
    assembly_file::AssemblyFile, assembly_project::AssemblyProject, cli::CLI,
    docs::Markdown, syntax
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
        AssemblyFile::parse::<syntax::NASM>(path, &source)?
    );
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = CLI::parse();
    assert!(
        args.out_dir.is_dir() || !args.out_dir.exists(),
        "argument passed '-o' was not a directory"
    );

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

    let project = AssemblyProject::build_from(files);
    let docs = project.generate_docs();
    if fs::read_dir(&args.out_dir).is_err() {
        fs::create_dir(&args.out_dir)?;
    }
    let mut file_map = HashMap::new();
    for (file, _) in &docs {
        let output_relative_path =
            PathBuf::from(file.with_extension("md").file_name().unwrap());
        file_map.insert(file.clone(), output_relative_path);
    }
    for (file, docs) in &docs {
        let mut output_path = PathBuf::from(&args.out_dir);
        output_path.push(file_map.get(file).unwrap());
        fs::write(output_path, docs.to::<Markdown>(&file_map))?;
    }

    Ok(())
}
