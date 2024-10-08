// Copyright (C) 2024 Ethan Uppal. All rights reserved

use std::{
    error,
    fmt::{self, Debug, Display},
    path::{Path, PathBuf}
};

use logos::{Logos, Span};

use crate::assembly_file::{
    AssemblyFile, AssemblyItem, AssemblyMacro, AssemblySection
};

use super::Syntax;

/// Grammar for NASM syntax.
#[derive(Logos, Debug, PartialEq, Eq, Clone, Copy)]
pub enum NASMTokenType {
    #[token("bits")]
    Bits,

    #[token("section")]
    Section,

    #[token("global")]
    Global,

    #[token("extern")]
    Extern,

    #[token("qword")]
    QWord,

    #[token("dword")]
    DWord,

    #[token("%include")]
    Include,

    #[token("%define")]
    Define,

    #[token("%macro")]
    Macro,

    // TODO: finish this
    #[regex("mov|add|jmp|push|pop|call|ret|sub|mul|div|inc|dec|and|or|xor|not|shl|shr|cmp|test|db|dd|align|equ|lea|jne|je|imul|syscall|jz|jnz")]
    Mnemonic,

    #[token("%endmacro")]
    EndMacro,

    #[regex("\\$[a-zA-Z0-9_.]+")]
    MacroCall,

    #[regex("%[0-9]+")]
    MacroArg,

    #[regex("r[0-9]+")]
    Register,

    #[regex("[a-zA-Z_.][a-zA-Z0-9_.$]*")]
    Symbol,

    #[token("$")]
    CurrentPosition,

    #[regex(r"[0-9]+")]
    Number,

    #[regex(r#"("([^"\\]|\\.)*")|('([^'\\]|\\.)*')"#)]
    String,

    // Comments
    #[regex(r";[^\n]*")]
    Comment,

    #[token(":")]
    Colon,

    #[token(",")]
    Comma,

    #[token("[")]
    LeftBracket,

    #[token("]")]
    RightBracket,

    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Asterisk,

    #[token("/")]
    Slash,

    #[token("~")]
    BitNot,

    #[token("|")]
    BitOr,

    #[token("^")]
    BitXor,

    #[token("&")]
    BitAnd,

    #[token("(")]
    LeftParen,

    #[token(")")]
    RightParen,

    #[token("\n")]
    Newline,

    #[regex(r"[ \t\f]+")]
    Whitespace,

    EOF
}

#[derive(Clone, Debug)]
pub struct SourceLocation<P: AsRef<Path>> {
    pub file: P,
    pub line: usize,
    pub col: usize
}

impl<'a> From<SourceLocation<&'a Path>> for SourceLocation<PathBuf> {
    fn from(value: SourceLocation<&'a Path>) -> Self {
        SourceLocation {
            file: value.file.to_path_buf(),
            line: value.line,
            col: value.col
        }
    }
}

pub struct NASMToken<'src> {
    pub ty: NASMTokenType,
    pub value: &'src str,
    pub span: Span,
    pub loc: SourceLocation<&'src Path>
}

impl<'src> Clone for NASMToken<'src> {
    fn clone(&self) -> Self {
        NASMToken {
            ty: self.ty,
            value: self.value,
            span: self.span.clone(),
            loc: self.loc.clone()
        }
    }
}

#[derive(Debug)]
pub enum NASMParseErrorType {
    InvalidInput,
    UnexpectedEOF,
    Unexpected {
        expected: NASMTokenType,
        received: Option<(NASMTokenType, String)>
    },
    InvalidSyntax
}

impl Display for NASMParseErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput => write!(f, "Invalid input"),
            Self::UnexpectedEOF => write!(f, "Unexpected end-of-file"),
            Self::Unexpected { expected, received } => {
                write!(f, "Expected {:?}", expected)?;
                if let Some((received, value)) = received {
                    write!(f, ", but received {:?} (`{}`)", received, value)?;
                }
                Ok(())
            }
            Self::InvalidSyntax => write!(f, "Invalid syntax")
        }
    }
}

type ParserTrace = Vec<(String, SourceLocation<PathBuf>)>;

#[derive(Debug)]
pub struct NASMParseError {
    ty: NASMParseErrorType,
    trace: ParserTrace
}

impl Display for NASMParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ty)?;
        if !self.trace.is_empty() {
            write!(f, ": ")?;
        }
        for (i, (rule, loc)) in self.trace.iter().enumerate() {
            if i > 0 {
                write!(f, " > ")?;
            }
            write!(
                f,
                "{}({}:{}:{})",
                rule,
                loc.file.file_name().unwrap().to_string_lossy(),
                loc.line,
                loc.col
            )?;
        }
        Ok(())
    }
}

impl error::Error for NASMParseError {}

type RuleResult = Result<(), NASMParseError>;

pub struct NASM<'src> {
    pos: usize,
    tokens: Vec<NASMToken<'src>>,
    eof_token: NASMToken<'src>,
    asm: AssemblyFile,
    current_section: AssemblySection,
    rule_stack: ParserTrace
}

macro_rules! rules {
    ($($vis:vis rule $name:ident(&mut $self:ident $(, $arg:ident: $arg_ty:ty)* $(,)?) -> RuleResult
        $body:block
    )*) => {
        $(
            paste::paste! {
                $vis fn [<rule_ $name>](&mut $self $(, $arg: $arg_ty)*) -> RuleResult {
                    if $self.is_eof() {
                        return Err($self.error(NASMParseErrorType::UnexpectedEOF));
                    }
                    $self.rule_stack.push(
                        (stringify!($name).to_string(), $self.current().loc.into())
                    );
                    $body?;
                    $self.rule_stack.pop();
                    Ok(())
                }
            }
        )*
    };
}

impl<'src> NASM<'src> {
    fn is_eof(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn current(&self) -> NASMToken<'src> {
        self.tokens[self.pos].clone()
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn take(&mut self) -> NASMToken<'src> {
        // can't use current() because of the borrow checker
        let cur = self.tokens[self.pos].clone();
        self.advance();
        cur
    }

    fn peek_is(&self, ty: NASMTokenType) -> bool {
        if self.pos + 1 < self.tokens.len() {
            self.tokens[self.pos + 1].ty == ty
        } else {
            false
        }
    }

    fn skip(&mut self) {
        while !self.is_eof()
            && (self.current().ty == NASMTokenType::Newline
                || self.current().ty == NASMTokenType::Whitespace)
        {
            self.advance()
        }
    }

    fn error(&self, ty: NASMParseErrorType) -> NASMParseError {
        let mut trace = self.rule_stack.clone();
        if self.is_eof() {
            trace.push((
                "end-of-file".into(),
                self.eof_token.loc.clone().into()
            ));
        } else {
            trace.push((
                format!("{:?}", self.current().ty),
                self.current().loc.clone().into()
            ));
        }
        NASMParseError { ty, trace }
    }

    fn expect(
        &mut self, expected: NASMTokenType
    ) -> Result<NASMToken<'src>, NASMParseError> {
        if self.is_eof() {
            Err(self.error(NASMParseErrorType::Unexpected {
                expected,
                received: None
            }))
        } else {
            let token = self.take();
            if token.ty == expected {
                Ok(token)
            } else {
                Err(self.error(NASMParseErrorType::Unexpected {
                    expected,
                    received: Some((token.ty, token.value.to_string()))
                }))
            }
        }
    }

    fn expect_newline(&mut self) -> Result<NASMToken<'src>, NASMParseError> {
        self.expect(NASMTokenType::Newline)
    }

    fn current_section(&mut self) -> &mut Vec<AssemblyItem> {
        self.asm.sections.entry(self.current_section).or_default()
    }

    rules! {
        rule bits(&mut self) -> RuleResult {
            self.expect(NASMTokenType::Bits)?;
            self.asm.bits = self
                .expect(NASMTokenType::Number)?
                .value
                .parse::<usize>()
                .or(Err(self.error(NASMParseErrorType::InvalidSyntax)))?;
            Ok(())
        }

        rule section(&mut self) -> RuleResult {
            self.expect(NASMTokenType::Section)?;
            let section_name = self.expect(NASMTokenType::Symbol)?.value;
            self.current_section = match section_name.to_ascii_lowercase().as_str()
            {
                ".text" => Some(AssemblySection::Text),
                ".data" => Some(AssemblySection::Data),
                ".rodata" => Some(AssemblySection::ROData),
                ".bss" => Some(AssemblySection::BSS),
                _ => None
            }
            .ok_or(self.error(NASMParseErrorType::InvalidSyntax))?;
            self.expect_newline()?;
            Ok(())
        }

        rule label(&mut self) -> RuleResult {
            let name = self.expect(NASMTokenType::Symbol)?.value.to_string();
            self.expect(NASMTokenType::Colon)?;
            self.current_section()
                .push(AssemblyItem::Label(name));
            Ok(())
        }

        rule mnemonic(&mut self) -> RuleResult {
            self.expect(NASMTokenType::Mnemonic)?;
            while !self.is_eof() && self.current().ty != NASMTokenType::Newline {
                self.advance();
            }
            self.expect_newline()?;
            Ok(())
        }

        rule global(&mut self) -> RuleResult {
            self.expect(NASMTokenType::Global)?.value.to_string();
            let label = self.expect(NASMTokenType::Symbol)?.value.to_string();
            self.expect_newline()?;
            self.asm.globals.insert(label);
            Ok(())
        }

        rule extern(&mut self) -> RuleResult {
            self.expect(NASMTokenType::Extern)?;
            let label = self.expect(NASMTokenType::Symbol)?.value.to_string();
            self.expect_newline()?;
            self.asm.externs.push(label);
            Ok(())
        }

        rule include(&mut self) -> RuleResult {
            self.expect(NASMTokenType::Include)?;
            let path = self.expect(NASMTokenType::String)?.value.to_string();
            let path = &path[1..path.len()-1];
            self.expect_newline()?;
            self.asm.includes.push(PathBuf::from(path));
            Ok(())
        }

        rule macro_definition(&mut self) -> RuleResult {
            self.expect(NASMTokenType::Macro)?;
            let name = self.expect(NASMTokenType::MacroCall)?.value.to_string();
            let arg_count = self.expect(NASMTokenType::Number)?
                .value
                .parse::<usize>()
                .map_err(|_| self.error(NASMParseErrorType::InvalidSyntax))?;
            while !self.is_eof() && self.current().ty != NASMTokenType::EndMacro {
                self.advance();
            }
            self.expect(NASMTokenType::EndMacro)?;
            self.asm.macros.push(AssemblyMacro {
                name, arg_count, body: Vec::new()
            });
            Ok(())
        }

        rule macro_call(&mut self) -> RuleResult {
            let name = self.expect(NASMTokenType::MacroCall)?.value.to_string();
            while !self.is_eof() && self.current().ty != NASMTokenType::Newline {
                self.advance();
            }
            self.expect_newline()?;
            self.current_section().push(AssemblyItem::MacroCall(name, Vec::new()));
            Ok(())
        }

        rule define(&mut self) -> RuleResult {
            self.expect(NASMTokenType::Define)?;
            let name = self.expect(NASMTokenType::Symbol)?.value.to_string();
            while !self.is_eof() && self.current().ty != NASMTokenType::Newline {
                self.advance();
            }
            self.expect_newline()?;
            self.asm.defines.push(name);
            Ok(())
        }
    }
}

impl<'src> Syntax<'src> for NASM<'src> {
    type Error = NASMParseError;

    fn new_parser(
        file: &'src Path, source: &'src str
    ) -> Result<Self, Self::Error> {
        let mut lexer = NASMTokenType::lexer(source);
        let mut tokens = Vec::new();
        let mut line = 1;
        let mut col = 1;
        while let Some(ty) = lexer.next() {
            let ty = ty.map_err(|_| Self::Error {
                ty: NASMParseErrorType::InvalidInput,
                trace: vec![(
                    "lex".into(),
                    SourceLocation {
                        file: file.to_path_buf(),
                        line,
                        col
                    }
                )]
            })?;

            if ty != NASMTokenType::Whitespace {
                tokens.push(NASMToken {
                    ty,
                    value: lexer.slice(),
                    span: lexer.span(),
                    loc: SourceLocation { file, line, col }
                });
            }

            if ty == NASMTokenType::Newline {
                line += 1;
                col = 1;
            } else {
                col += lexer.slice().len();
            }
        }
        let eof_token = NASMToken {
            ty: NASMTokenType::EOF,
            value: "",
            span: Span {
                start: source.len(),
                end: source.len()
            },
            loc: SourceLocation { file, line, col }
        };

        Ok(Self {
            pos: 0,
            tokens,
            eof_token,
            asm: AssemblyFile::default(),
            current_section: AssemblySection::Text,
            rule_stack: ParserTrace::new()
        })
    }

    fn parse(mut self) -> Result<AssemblyFile, Self::Error> {
        if !self.is_eof() {
            self.rule_stack
                .push(("parse".to_string(), self.current().loc.clone().into()));
        }
        self.skip();
        while !self.is_eof() {
            match self.current().ty {
                NASMTokenType::Bits => self.rule_bits(),
                NASMTokenType::Section => self.rule_section(),
                NASMTokenType::Symbol if self.peek_is(NASMTokenType::Colon) => {
                    self.rule_label()
                }
                NASMTokenType::Mnemonic => self.rule_mnemonic(),
                NASMTokenType::Global => self.rule_global(),
                NASMTokenType::Extern => self.rule_extern(),
                NASMTokenType::Macro => self.rule_macro_definition(),
                NASMTokenType::MacroCall => self.rule_macro_call(),
                NASMTokenType::Comment => {
                    // TODO:
                    self.advance();
                    Ok(())
                }
                NASMTokenType::Include => self.rule_include(),
                NASMTokenType::Define => self.rule_define(),
                _ => Err(self.error(NASMParseErrorType::InvalidSyntax))
            }?;
            self.skip();
        }

        Ok(self.asm)
    }
}
