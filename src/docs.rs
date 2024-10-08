// Copyright (C) 2024 Ethan Uppal. All  rights reserved.

use inform::fmt::IndentFormatter;
use std::{
    collections::HashMap,
    fmt::{self, Display, Write},
    marker::PhantomData,
    path::PathBuf
};

const INDENT: usize = 2;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Global,
    Private,
    External
}

impl Display for Visibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Visibility::Global => "global",
            Visibility::Private => "private",
            Visibility::External => "external"
        }
        .fmt(f)
    }
}

pub enum Docs {
    File {
        path: PathBuf,
        symbols: Box<Docs>,
        defines: Box<Docs>,
        macros: Box<Docs>
    },
    Paragraphs(Vec<Box<Docs>>),
    List(Vec<Box<Docs>>),
    Table {
        header: Vec<Box<Docs>>,
        rows: Vec<Vec<Box<Docs>>>
    },
    Macro {
        name: String,
        arg_count: usize
    },
    Define {
        name: String
    },
    InlineCode(String),
    Text(String),
    CellLines(Vec<Box<Docs>>),
    ResolveFile(PathBuf),
    Concat(Vec<Box<Docs>>)
}

pub trait Backend {
    fn fmt(
        docs: &Docs, f: &mut IndentFormatter,
        file_map: &HashMap<PathBuf, PathBuf>
    ) -> fmt::Result;
}

struct IndentDisplay<'docs, B: Backend>(
    PhantomData<B>,
    &'docs Docs,
    &'docs HashMap<PathBuf, PathBuf>
);

impl<'docs, B: Backend> Display for IndentDisplay<'docs, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = IndentFormatter::new(f, INDENT);
        let Self(_, docs, file_map) = self;
        B::fmt(docs, &mut f, file_map)
    }
}

impl Docs {
    /// `file_map` must contain, for each file referenced in this documentation,
    /// a file path to the intended location of the documentation for that
    /// file. For example, if a file references `foo.nasm`, then you must supply
    /// the path (e.g., `foo.md`) where the documentation for `foo.nasm`
    /// will be supplied.
    pub fn to<B: Backend>(
        &self, file_map: &HashMap<PathBuf, PathBuf>
    ) -> String {
        IndentDisplay::<B>(PhantomData, self, file_map).to_string()
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::File { .. } => false,
            Self::Paragraphs(items) => items.is_empty(),
            Self::List(list) => list.is_empty(),
            Self::Table { rows, .. } => rows.is_empty(),
            Self::Macro { .. } => false,
            Self::Define { .. } => false,
            Self::InlineCode(..) => false,
            Self::Text(..) => false,
            Self::CellLines(lines) => lines.is_empty(),
            Self::ResolveFile(..) => false,
            Self::Concat(items) => items.is_empty()
        }
    }
}

pub struct Markdown;

impl Backend for Markdown {
    fn fmt(
        docs: &Docs, f: &mut IndentFormatter,
        file_map: &HashMap<PathBuf, PathBuf>
    ) -> fmt::Result {
        match docs {
            Docs::File {
                path,
                symbols,
                defines,
                macros
            } => {
                writeln!(f, "<!-- This file was generated by asmdoc <https://github.com/ethanuppal/asmdoc>. -->")?;
                writeln!(
                    f,
                    "# {}\n",
                    path.file_name().unwrap().to_string_lossy()
                )?;

                if !symbols.is_empty() {
                    writeln!(f, "## Symbols")?;
                    Self::fmt(symbols, f, file_map)?;
                    writeln!(f)?;
                }

                if !defines.is_empty() {
                    writeln!(f, "## Defines")?;
                    Self::fmt(defines, f, file_map)?;
                    writeln!(f)?;
                }

                if !macros.is_empty() {
                    writeln!(f, "## Macros")?;
                    Self::fmt(macros, f, file_map)?;
                    writeln!(f)?;
                }

                Ok(())
            }
            Docs::Paragraphs(items) => items.iter().try_for_each(|item| {
                write!(f, "- ")
                    .and_then(|_| Self::fmt(item, f, file_map))
                    .and_then(|_| write!(f, "\n\n"))
            }),
            Docs::List(items) => items.iter().try_for_each(|item| {
                write!(f, "- ")
                    .and_then(|_| Self::fmt(item, f, file_map))
                    .and_then(|_| writeln!(f))
            }),
            Docs::Table { header, rows } => {
                write!(f, "\n| ")?;
                for col in header {
                    Self::fmt(col, f, file_map)?;
                    write!(f, " |")?;
                }
                writeln!(f)?;

                write!(f, "| ")?;
                for _ in header {
                    write!(f, "--- |")?;
                }
                writeln!(f)?;

                for row in rows {
                    write!(f, "| ")?;
                    for col in row {
                        Self::fmt(col, f, file_map)?;
                        write!(f, " |")?;
                    }
                    writeln!(f)?;
                }

                Ok(())
            }
            Docs::Macro { name, arg_count } => {
                write!(
                    f,
                    "`{}` ({} argument{})",
                    name,
                    arg_count,
                    if *arg_count == 1 { "" } else { "s" }
                )
            }
            Docs::Define { name } => write!(f, "`{}`", name),
            Docs::InlineCode(code) => write!(f, "`{}`", code),
            Docs::Text(text) => write!(f, "{}", text),
            Docs::CellLines(lines) => {
                for (i, line) in lines.iter().enumerate() {
                    if i > 0 {
                        write!(f, "<br>")?;
                    }
                    Self::fmt(line, f, file_map)?;
                }
                Ok(())
            }
            Docs::ResolveFile(file) => {
                write!(
                    f,
                    "[{}]({})",
                    file.file_name().unwrap().to_string_lossy(),
                    file_map.get(file).unwrap().to_string_lossy()
                )
            }
            Docs::Concat(items) => items
                .iter()
                .try_for_each(|item| Self::fmt(item, f, file_map))
        }
    }
}
