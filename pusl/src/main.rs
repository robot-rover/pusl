use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use clap::{App, Arg, SubCommand};
use pusl_lang::backend::{
    execute,
    linearize::{linearize_file, ByteCodeFile},
    startup, ExecContext, debug,
};
use pusl_lang::lexer::lex;
use pusl_lang::parser::parse;
use std::io;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::PathBuf;
use std::{fs::File, io::Seek};

const MAJOR_VERSION: u16 = 1; // Bytecode to run must match
const MINOR_VERSION: u16 = 1; // Ok to run bytecode where bytecode minor version < interpreter minor version

const MAGIC_NUMBER: &[u8] = "pusl".as_bytes();

fn open_code_or_source(path: &PathBuf, verbosity: u64) -> io::Result<(impl io::BufRead, [u8; 4])> {
    if verbosity >= 1 {
        println!("Using input file: {}", path.display());
    }
    let input_file = File::open(&path)?;
    let mut reader = BufReader::new(input_file);
    let mut magic_number = [0u8; 4];
    reader.read_exact(&mut magic_number)?;
    reader.seek(io::SeekFrom::Start(0))?;
    Ok((reader, magic_number))
}

fn compile_from_source(
    path: &PathBuf,
    reader: impl BufRead,
    verbosity: u64,
) -> io::Result<ByteCodeFile> {
    if verbosity >= 1 {
        println!("Using input file: {}", path.display());
    }
    let lines = reader.lines().collect::<Result<Vec<String>, _>>()?;
    let tokens = lex(lines.iter().map(|str| str.as_str()));
    let ast = parse(tokens);
    let base_func = linearize_file(ast);
    if verbosity >= 2 {
        println!("{:?}", &base_func);
    }
    Ok(base_func)
}

fn write_to_code_path(path: &PathBuf, base_func: &ByteCodeFile, verbosity: u64) -> io::Result<()> {
    if verbosity >= 1 {
        println!("Using output file: {}", path.display());
    }
    if verbosity >= 2 {
        println!("{:?}", base_func);
    }
    let output_file = File::create(path)?;
    let mut writer = BufWriter::new(output_file);

    writer.write_all(MAGIC_NUMBER)?;
    writer.write_u16::<LittleEndian>(MAJOR_VERSION)?; // Bytecode Major Version
    writer.write_u16::<LittleEndian>(MINOR_VERSION)?; // Bytecode Minor Version
    bincode::serialize_into(writer, base_func).expect("Unable to write bytecode");
    Ok(())
}

fn load_code_from_path(
    path: &PathBuf,
    mut reader: impl BufRead,
    verbosity: u64,
) -> io::Result<ByteCodeFile> {
    if verbosity >= 1 {
        println!("Using input file: {}", path.display());
    }

    let mut magic_number = [0u8; 4];
    reader.read_exact(&mut magic_number)?;
    assert_eq!(magic_number, MAGIC_NUMBER, "Bytecode is corrupt");
    let bytcode_major = reader.read_u16::<LittleEndian>()?;
    assert_eq!(
        bytcode_major, MAJOR_VERSION,
        "Bytecode version is incompatible"
    );
    let bytcode_minor = reader.read_u16::<LittleEndian>()?;
    assert!(
        bytcode_minor <= MINOR_VERSION,
        "Bytecode version is incompatible"
    );
    let function: ByteCodeFile = bincode::deserialize_from(reader).expect("Bytecode is corrupt");
    if verbosity >= 2 {
        println!("{:?}", &function);
    }
    Ok(function)
}

fn main() -> io::Result<()> {
    let matches = App::new("pusl")
        .version("0.1.0")
        .author("robot_rover <sam.obrien00@gmail.com>")
        .about("pusl language | compiler | interpreter | virtual machine")
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets verbose output"),
        )
        .subcommand(
            SubCommand::with_name("compile")
                .about("compile \".pusl\" source files to \".puslc\" bytecode files")
                .arg(
                    Arg::with_name("SOURCE")
                        .help("path to the source file")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("analyze")
                        .short("a")
                        .long("analyze")
                        .help("print compiled bytecode rather than writing to disk"),
                ),
        )
        .subcommand(
            SubCommand::with_name("run")
                .about("execute a \".pusl\" source file or a compiled \".puslc\" bytecode file")
                .arg(
                    Arg::with_name("PATH")
                        .help("path to the file")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("analyze")
                        .short("a")
                        .long("analyze")
                        .help("print compiled bytecode rather running it"),
                ),
        )
        .subcommand(
            SubCommand::with_name("debug")
                .about("execute a \".pusl\" source file or a compiled \".puslc.\" bytecode file with the pusl debugger")
                .arg(
                    Arg::with_name("PATH")
                        .help("path to the file")
                        .required(true)
                        .index(1),
                    ),
                )
        .get_matches();

    let verbosity = matches.occurrences_of("v");

    match matches.subcommand() {
        ("compile", Some(matches)) => {
            let mut path = PathBuf::from(matches.value_of("SOURCE").unwrap());

            let (reader, _magic) = open_code_or_source(&path, verbosity)?;
            let bcf = compile_from_source(&path, reader, verbosity)?;
            if matches.is_present("analyze") {
                println!("{:#?}", bcf.base_func);
            } else {
                path.set_extension("puslc");
                write_to_code_path(&path, &bcf, verbosity)?;
            }
        }
        ("run", Some(matches)) => {
            let path = PathBuf::from(matches.value_of("PATH").unwrap());

            let (reader, magic) = open_code_or_source(&path, verbosity)?;
            let bcf = if magic == MAGIC_NUMBER {
                load_code_from_path(&path, reader, verbosity)?
            } else {
                compile_from_source(&path, reader, verbosity)?
            };
            if matches.is_present("analyze") {
                println!("{:#?}", bcf.base_func);
            } else {
                let ctx = ExecContext::default();
                let mut state = startup(bcf, path, ctx);
                execute(&mut state);
            }
        }
        ("debug", Some(matches)) => {
            let path = PathBuf::from(matches.value_of("PATH").unwrap());

            let (reader, magic) = open_code_or_source(&path, verbosity)?;
            let bcf = if magic == MAGIC_NUMBER {
                load_code_from_path(&path, reader, verbosity)?
            } else {
                compile_from_source(&path, reader, verbosity)?
            };
            if matches.is_present("analyze") {
                println!("{:#?}", bcf.base_func);
            } else {
                let mut ctx = ExecContext::default();
                let mut interrupt = debug::make_interrupt();
                ctx.interrupt = Some(&mut interrupt);
                let mut state = startup(bcf, path, ctx);
                execute(&mut state);
            }
        }
        _ => println!("{}", matches.usage()),
    }

    Ok(())
}
