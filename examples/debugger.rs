#![feature(const_fn)]

extern crate armor;

use std::io::prelude::*;
use std::io;
use armor::computer::Computer;

type CommandHandler = &'static Fn(&[&str], &mut Computer);

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

    fn find<'a>(id: &'a [&'a str]) ->
        Option<(&'static Command, &'a [&'a str])>
    {
        let mut cmd: Option<&Command> = None;
        for i in 0..id.len() {
            match cmd {
                Some(command) => {
                    let mut updated = false;
                    for c in command.subcommands {
                        if c.name == id[i] {
                            cmd = Some(&c);
                            updated = true;
                            break;
                        }
                    }
                    if !updated {
                        return Some((cmd.unwrap(), &id[i..]));
                    }
                },
                None => {
                    for c in COMMANDS {
                        if c.name == id[i] {
                            cmd = Some(&c);
                            break;
                        }
                    }
                    if cmd.is_none() {
                        return None
                    }
                },
            }
        }

        match cmd {
            Some(c) => Some((c, &[])),
            None => None,
        }
    }
}

const COMMANDS: &'static [Command] = &[
    Command::new(
        "help",
        "Display documentation for supported commands",
        &handle_help,
        &[]),
    Command::new(
        "print",
        "Print current state of the emulated computer",
        &handle_print,
        PRINT_SUBCOMMANDS),
];

const PRINT_SUBCOMMANDS: &'static [Command] = &[
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

fn handle_help(args: &[&str], computer: &mut Computer) {
    match Command::find(args) {
        Some((cmd, _)) => {
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
            for c in COMMANDS {
                println!("    {}: {}", c.name, c.doc_string);
            }
            println!("    exit: Exit the debugger without saving anything");
        },
    }
}

fn handle_print(args: &[&str], computer: &mut Computer) {
    println!("TODO: specialized help for print");
}

fn handle_print_registers(args: &[&str], computer: &mut Computer) {
    println!("TODO: print registers");
}

fn handle_print_code(args: &[&str], computer: &mut Computer) {
    println!("TODO: print code");
}


fn debugger_repl(computer: &mut Computer) -> Result<(), io::Error> {
    loop {
        print!("> ");
        try!(io::stdout().flush());

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
                match Command::find(&words) {
                    Some((cmd, args)) => {
                        (cmd.handler)(args, computer);
                        println!("");
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

    let boot_code = vec![];
    let computer = &mut Computer::new(boot_code);
    debugger_repl(computer);
}
