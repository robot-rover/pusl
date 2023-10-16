#![allow(unused_imports)]
extern crate pusl_lang;
extern crate shrust;
extern crate simplelog;

use pusl_lang::backend::debug::{DebugCommand, DebugResponse};
use pusl_lang::backend::linearize::{linearize_file, ByteCodeFile};
use pusl_lang::backend::{startup, ExecContext};
use pusl_lang::lexer::lex;
use pusl_lang::parser::parse;
use shrust::{ExecError, Shell, ShellIO};
use simplelog::{Config, ConfigBuilder, LevelFilter, TermLogger, TerminalMode};
use std::path::PathBuf;
use std::process::exit;
use std::sync::mpsc;
use std::sync::mpsc::RecvError;
use std::thread;
use std::{num::ParseIntError, unimplemented};

const SMALL_SOURCE: &'static str = include_str!("../resources/simple_program.pusl");
const SECOND_SOURCE: &'static str = include_str!("../resources/secondary_source.pusl");

fn test_resolve(path: Vec<String>) -> Option<ByteCodeFile> {
    assert_eq!(path.join("/"), "secondary_source");
    let lines = SECOND_SOURCE.lines();
    let roots = lex(lines);
    let ast = parse(roots);
    let code = linearize_file(ast);
    Some(code)
}

#[allow(unreachable_code, unused_variables)]
fn main() {
    let mut config = ConfigBuilder::new();
    config
        .set_time_level(LevelFilter::Off)
        .set_location_level(LevelFilter::Off)
        .set_thread_level(LevelFilter::Off);
    TermLogger::init(LevelFilter::Debug, config.build(), TerminalMode::Mixed).unwrap();
    let lines = SMALL_SOURCE.lines();
    let roots = lex(lines);
    let ast = parse(roots);
    let code = linearize_file(ast);
    let ctx = ExecContext {
        resolve: test_resolve,
        stream: None,
    };
    let (command_channel_send, command_channel_recv) = mpsc::channel::<DebugCommand>();
    let (response_channel_send, response_channel_recv) = mpsc::channel::<DebugResponse>();
    let cli_channels = (command_channel_send, response_channel_recv);
    let debug_channels = (command_channel_recv, response_channel_send);

    unimplemented!();
    // thread::spawn(move || startup(code, ctx, Some(debug_channels)));

    let line = if let DebugResponse::Paused(line) = cli_channels.1.recv().unwrap() {
        line
    } else {
        panic!()
    };

    let mut shell = Shell::new((cli_channels.0, cli_channels.1, line));
    shell.new_command_noargs("run", "run the program to the end", |io, data| {
        data.0.send(DebugCommand::Run).unwrap();
        let result = data.1.recv().unwrap();
        match result {
            DebugResponse::Paused(line) => {
                data.2 = line;
            }
            DebugResponse::Done => {
                exit(0);
            }
        }
        Ok(())
    });

    shell.new_command_noargs("next", "run the next line of bytecode", |io, data| {
        data.0.send(DebugCommand::RunToIndex(data.2 + 1)).unwrap();
        let result = data.1.recv().unwrap();
        match result {
            DebugResponse::Paused(line) => {
                data.2 = line;
            }
            DebugResponse::Done => {
                exit(0);
            }
        }
        Ok(())
    });

    shell.new_command(
        "line",
        "run to the specified bytecode index",
        1,
        |io, data, args| {
            let target = match args[0].parse::<usize>() {
                Ok(line) => line,
                Err(err) => return Err(ExecError::Other(Box::new(err))),
            };
            data.0.send(DebugCommand::RunToIndex(target)).unwrap();
            let result = data.1.recv().unwrap();
            match result {
                DebugResponse::Paused(line) => data.2 = line,
                DebugResponse::Done => {
                    exit(0);
                }
            }
            Ok(())
        },
    );

    shell.run_loop(&mut ShellIO::default());
}
