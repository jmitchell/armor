#![feature(const_fn)]

extern crate armor;

use std::io::prelude::*;
use std::io;
use armor::computer::Computer;

type CommandHandler = &'static Fn(&[&str], &mut Computer) -> Result<(), String>;

struct Command {
    name: &'static str,
    doc_string: &'static str,
    handler: CommandHandler,
    subcommands: &'static [Command],
}

impl Command {
    const fn new(
        name: &'static str,
        doc_string: &'static str,
        handler: CommandHandler,
        subcommands: &'static [Command]) -> Command
    {
        Command {
            name: name,
            doc_string: doc_string,
            handler: handler,
            subcommands: subcommands,
        }
    }

    fn find(id: &[&str]) -> Option<&'static Command> {
        let mut cmd: Option<&Command> = None;
        for name in id {
            // TODO: DRY
            match cmd {
                Some(command) => {
                    for c in command.subcommands {
                        if &c.name == name {
                            cmd = Some(&c);
                            break;
                        }
                    }
                    if cmd.is_none() {
                        break;
                    }
                },
                None => {
                    for c in commands {
                        if &c.name == name {
                            cmd = Some(&c);
                            break;
                        }
                    }
                    if cmd.is_none() {
                        break;
                    }
                },
            }
        }
        cmd
    }
}

const commands: &'static [Command] = &[
    Command::new(
        "help",
        "Display documentation for supported commands",
        &handle_help,
        &[]),
    Command::new(
        "print",
        "Print current state of the emulated computer",
        &handle_print,
        print_subcommands),
];

const print_subcommands: &'static [Command] = &[
    Command::new(
        "registers",
        "Show the CPU registers",
        &handle_print_registers,
        &[]),
    Command::new(
        "code",
        "Machine code near the Program Counter",
        &handle_print_code,
        &[]),
];

fn handle_help(args: &[&str], computer: &mut Computer) -> Result<(), String> {
    match Command::find(&args[0..1]) {
        Some(cmd) => {
            println!("
Subcommands for '{}' command
----------------------------------
", cmd.name);

            for c in cmd.subcommands {
                println!("    {}: {}", c.name, c.doc_string);
            }
        },
        None => {
            println!("
Top-level commands
------------------
");
            for c in commands {
                println!("    {}: {}", c.name, c.doc_string);
            }
            println!("    exit: Exit the debugger without saving anything");
        },
    }

    println!("");
    return Ok(())
}

fn handle_print(args: &[&str], computer: &mut Computer) -> Result<(), String> {
    match Command::find(&args[0..1]) {
        Some(subcmd) => return (subcmd.handler)(&args[1..], computer),
        None => {
            println!("TODO: specialized help for print");
        }
    }
    return Ok(())
}

fn handle_print_registers(args: &[&str], computer: &mut Computer) -> Result<(), String> {
    println!("TODO: print registers");
    return Ok(())
}

fn handle_print_code(args: &[&str], computer: &mut Computer) -> Result<(), String> {
    println!("TODO: print code");
    return Ok(())
}


fn debugger_repl(computer: &mut Computer) -> Result<(), io::Error> {
    loop {
        print!("> ");
        io::stdout().flush();

        let mut input = String::new();
        try!(io::stdin().read_line(&mut input));
        match input.trim() {
            "" => continue,
            "exit" => break,
            other => {
                let words = other
                    .split(' ')
                    .filter_map(|word| {
                        if word.len() > 0 {
                            Some(word)
                        } else {
                            None
                        }
                    }).collect::<Vec<_>>();

                assert!(words.len() > 0);
                match Command::find(&words[0..1]) {
                    Some(ref cmd) => {
                        (cmd.handler)(&words[1..], computer);
                    },
                    None => println!("Unrecognized command '{}'.
Type 'help' to see supported commands.
", words[0]),
                }
            },
        }
    }

    return Ok(())
}

fn main() {
    println!("
ARMOR Debugging Interface
=========================
");

    debugger_repl(&mut Computer::new(vec![]));
}
