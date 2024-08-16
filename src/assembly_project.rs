// Copyright (C) 2024 Ethan Uppal. All  rights reserved.

use std::{collections::HashMap, path::PathBuf};

use linked_hash_map::LinkedHashMap;

use crate::{
    assembly_file::{AssemblyFile, AssemblyItem, AssemblySection},
    docs::{Docs, Visibility}
};

#[derive(Default)]
pub struct AssemblyProject {
    files: HashMap<PathBuf, AssemblyFile>,
    symbols: HashMap<
        PathBuf,
        LinkedHashMap<String, (Visibility, Option<AssemblySection>)>
    >,
    /// Location of project-defined globals.
    global_sources: HashMap<String, PathBuf>,
    /// Location of project-internal externs.
    internal_externs: HashMap<String, PathBuf>,
    symbol_constituents: HashMap<String, Vec<String>>
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
                self.global_sources.insert(global.clone(), file.clone());
            }
        }
        for (file, asm) in &self.files {
            for extern_ in &asm.externs {
                if let Some(global_def_file) = self.global_sources.get(extern_)
                {
                    self.internal_externs
                        .insert(extern_.clone(), global_def_file.clone());
                }
            }

            let local_symbols = self.symbols.entry(file.clone()).or_default();

            for extern_ in &asm.externs {
                local_symbols
                    .insert(extern_.clone(), (Visibility::External, None));
            }

            let mut current_label = String::new();
            for (section, items) in &asm.sections {
                for item in items {
                    if let AssemblyItem::Label(label) = item {
                        if label.starts_with(".") {
                            self.symbol_constituents
                                .entry(current_label.clone())
                                .or_default()
                                .push(label.clone());
                        } else {
                            current_label = label.clone();
                            let visibility =
                                if asm.globals.contains(&current_label) {
                                    Visibility::Global
                                } else {
                                    Visibility::Private
                                };
                            local_symbols.insert(
                                current_label.clone(),
                                (visibility, Some(*section))
                            );
                        }
                    }
                }
            }
        }
        self
    }

    pub fn generate_docs(&self) -> Vec<(PathBuf, Docs)> {
        // what a nightmare!
        let mut docs = Vec::new();
        for (file, asm) in &self.files {
            let mut symbol_docs = Vec::new();
            for (symbol, (visibility, section)) in
                self.symbols.get(file).unwrap()
            {
                let file = if *visibility == Visibility::External {
                    self.internal_externs.get(symbol).cloned()
                } else {
                    None
                };
                let constituents = self
                    .symbol_constituents
                    .get(symbol)
                    .map(|constituents| {
                        constituents
                            .iter()
                            .map(|constituent| {
                                Box::new(Docs::InlineCode(format!(
                                    "`{}`",
                                    constituent
                                )))
                            })
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                let mut symbol_cell =
                    vec![Box::new(Docs::InlineCode(symbol.clone()))];
                for constituent in constituents {
                    symbol_cell.push(Box::new(Docs::Concat(vec![
                        Box::new(Docs::Text("- ".into())),
                        constituent,
                    ])));
                }
                symbol_docs.push(vec![
                    Box::new(Docs::Text(visibility.to_string())),
                    Box::new(Docs::CellLines(symbol_cell)),
                    Box::new(Docs::Text(
                        section.map(|s| s.to_string()).unwrap_or_default()
                    )),
                    Box::new(if let Some(file) = file {
                        Docs::ResolveFile(file)
                    } else {
                        Docs::Text("".into())
                    }),
                ]);
            }
            let defines_docs = asm
                .defines
                .iter()
                .map(|define| {
                    Box::new(Docs::Define {
                        name: define.clone()
                    })
                })
                .collect();
            let macro_docs = asm
                .macros
                .iter()
                .map(|macro_| {
                    Box::new(Docs::Macro {
                        name: macro_.name.clone(),
                        arg_count: macro_.arg_count
                    })
                })
                .collect();
            let file_docs = Docs::File {
                path: file.clone(),
                symbols: Box::new(Docs::Table {
                    header: vec![
                        Box::new(Docs::Text("Visibility".into())),
                        Box::new(Docs::Text("Label".into())),
                        Box::new(Docs::Text("Section".into())),
                        Box::new(Docs::Text("Defined in".into())),
                    ],
                    rows: symbol_docs
                }),
                defines: Box::new(Docs::List(defines_docs)),
                macros: Box::new(Docs::List(macro_docs))
            };
            docs.push((file.clone(), file_docs));
        }
        docs
    }
}
