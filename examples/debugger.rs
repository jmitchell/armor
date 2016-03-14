extern crate armor;

use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io;
use armor::address;
use armor::address::{
    Region
};
use armor::computer::Computer;
use armor::registers::{
    ProgramStatusRegister,
    RegisterBank,
};

type CommandHandler = &'static Fn(&[&str], &mut Computer);

struct Command(&'static str, &'static str, CommandHandler, &'static [Command]);

impl Command {
    fn find<'a>(id: &'a [&'a str]) ->
        Option<(&'static Command, &'a [&'a str])>
    {
        let mut cmd: Option<&Command> = None;
        for i in 0..id.len() {
            match cmd {
                Some(&Command(_, _, _, subcommands)) => {
                    let mut updated = false;
                    for c in subcommands {
                        let &Command(name, _, _, _) = c;
                        if name == id[i] {
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
                        let &Command(name, _, _, _) = c;
                        if name == id[i] {
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
    Command(
        "print",
        "Print current state of the emulated computer",
        &handle_print,
        PRINT_SUBCOMMANDS),
    Command(
        "step",
        "Execute next instruction",
        &handle_step,
        &[]),
    Command(
        "run",
        "Continually execute instructions",
        &handle_run,
        &[]),
    Command(
        "help",
        "Display documentation for supported commands",
        &handle_help,
        &[]),
];

const PRINT_SUBCOMMANDS: &'static [Command] = &[
    Command(
        "registers",
        "Show the CPU registers",
        &handle_print_registers,
        &[]),
    Command(
        "code",
        "Machine code near the Program Counter",
        &handle_print_code,
        &[]),
];

fn handle_help(args: &[&str], _computer: &mut Computer) {
    match Command::find(args) {
        Some((&Command(name, _, _, subcommands), _)) => {
            println!("
Subcommands for '{}' command
----------------------------------
", name);

            for &Command(name, doc_string, _, _) in subcommands {
                println!("    {}: {}", name, doc_string);
            }
        },
        None => {
            println!("
Top-level commands
------------------
");
            for &Command(name, doc_string, _, _) in COMMANDS {
                println!("    {}: {}", name, doc_string);
            }
            println!("    exit: Exit the debugger without saving anything");
        },
    }
}

fn handle_step(_args: &[&str], computer: &mut Computer) {
    computer.execute_next_instruction();
}

fn handle_run(_args: &[&str], computer: &mut Computer) {
    println!("");
    let mut prev_addr: Option<u32> = None;
    loop {
        // TODO: refactor code copied from handle_print_code.
        let pc_addr = computer.cpu.register_file.lookup(RegisterBank::R15).unwrap().bits;
        assert!(pc_addr % 4 == 0);
        if prev_addr.is_some() && pc_addr != prev_addr.unwrap() + 4 {
            println!("");
        }
        print!("\t0x{:08x}: ", pc_addr);
        match computer.instruction_at(pc_addr as u64) {
            Err(s) => println!("{}", s),
            Ok(instr) => println!("{}", instr.as_str()),
        }
        computer.execute_next_instruction();
        prev_addr = Some(pc_addr);
    }
}

fn handle_print(_args: &[&str], _computer: &mut Computer) {
    println!("TODO: print");
}

fn handle_print_registers(_args: &[&str], computer: &mut Computer) {
    println!("{:#?}", computer.cpu.register_file);

    let cpsr = computer.cpu.register_file.cpsr();
    println!("CPSR: {:#?}", cpsr);
    println!("Mode: {:#?}", cpsr.mode().unwrap());
}

fn handle_print_code(_args: &[&str], computer: &mut Computer) {
    let pc_addr = computer.cpu.register_file.lookup(RegisterBank::R15).unwrap().bits;
    assert!(pc_addr % 4 == 0);

    let instrs_before_and_after = 5;
    let amount = instrs_before_and_after * 4;
    let low = if pc_addr < amount {
        0 as address::Address
    } else {
        (pc_addr - amount) as address::Address
    };
    let high = if pc_addr > (u32::max_value() - amount) {
        u32::max_value() as address::Address
    } else {
        (pc_addr + amount) as address::Address
    };

    let mut addr = low;
    println!("");
    while addr <= high {
        if addr == pc_addr as address::Address {
            print!("\t>");
        } else {
            print!("\t ");
        }
        print!(" 0x{:08x}: ", addr);

        match computer.instruction_at(addr) {
            Err(s) => println!("{}", s),
            Ok(instr) => println!("{:?}", instr),
        }
        addr += 4;
    }
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
                    Some((&Command(_, _, handler, _), args)) => {
                        handler(args, computer);
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

fn load_boot_code(path: String) -> Result<Vec<u8>, io::Error> {
    let mut f = try!(File::open(path));
    let mut buf = vec![];
    try!(f.read_to_end(&mut buf));
    Ok(buf)
}

fn main() {
    println!("
ARMOR Debugging Interface
=========================
");

    if let Some(boot_bin_file) = env::args().nth(1) {
        // TODO: endianness
        println!("Loading boot file: {}", boot_bin_file);
        match load_boot_code(boot_bin_file) {
            Ok(boot_code) => {
                let mut computer = Computer::new(boot_code);
                debugger_repl(&mut computer).is_ok();
            },
            Err(_) => panic!("Unexpected error while loading boot code file"),
        }
    } else {
        panic!("missing argument: path to boot binary file");
    }
}
