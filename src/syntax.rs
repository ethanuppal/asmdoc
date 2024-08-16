// Copyright (C) 2024 Ethan Uppal. All rights reserved

use std::{
    error,
    fmt::{Debug, Display}
};

use crate::assembly_file::AssemblyFile;

pub mod nasm;
pub use nasm::NASM;

pub trait Syntax<'src>
where
    Self: Sized {
    type Error: Display + Debug + error::Error;

    fn new_parser(source: &'src str) -> Result<Self, Self::Error>;

    fn parse(self) -> Result<AssemblyFile, Self::Error>;
}
