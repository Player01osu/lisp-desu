#![allow(dead_code)]
use ast::{Pythonify, TranspileError};
use parser::{ParseError, StringReader, TokenKind};
use std::env::{args, Args};
use std::fs;
use std::path::Path;

#[derive(Debug)]
enum CliError {
    ArgsError(ArgsError),
    ParseError(ParseError),
    TranspileError(TranspileError),
}

impl From<TranspileError> for CliError {
    fn from(value: TranspileError) -> Self {
        Self::TranspileError(value)
    }
}

impl From<std::io::Error> for CliError {
    fn from(value: std::io::Error) -> Self {
        todo!()
    }
}

impl From<ParseError> for CliError {
    fn from(value: ParseError) -> Self {
        Self::ParseError(value)
    }
}

#[derive(Debug)]
enum ArgsError {
    NotEnoughArgs,
    MissingInput,
}

fn main() -> Result<(), CliError> {
    let args = args().collect::<Vec<String>>();
    let program = args.first().expect("Program name should exist");
    if args.len() < 2 {
        // TODO
        todo!("REPL mode");
        //return Err(CliError::ArgsError(ArgsError::NotEnoughArgs));
        //eprintln!("{program}: Missing file path");
        //exit(1);
    }

    let mut outpath = None;
    let mut inpath = None;
    let mut change_outpath = false;

    for arg in &args[1..] {
        match arg.as_str() {
            "-o" => change_outpath = true,
            s => {
                if change_outpath {
                    outpath = Some(s);
                    change_outpath = false;
                } else {
                    inpath = Some(s);
                }
            }
        }
    }

    let file_path = inpath.ok_or_else(|| CliError::ArgsError(ArgsError::MissingInput))?;
    let outpath = outpath.map(|v| v.to_owned()).unwrap_or_else(|| {
        format!(
            "{}.py",
            Path::new(file_path).file_stem().unwrap().to_string_lossy()
        )
    });
    let src = fs::read_to_string(file_path)?;

    let transpiler = Pythonify::new(&src);
    transpiler.output(outpath)?;

    Ok(())
}
