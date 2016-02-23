extern crate armor;

use std::cmp;
use std::io::prelude::*;
use std::io;
use armor::computer::Computer;
use armor::address::{
    Addressable,
    Region,
};
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
        0u64
    } else {
        (pc_addr - amount) as u64
    };
    let high = if pc_addr > (u32::max_value() - amount) {
        u32::max_value() as u64
    } else {
        (pc_addr + amount) as u64
    };

    // let ref mem = computer.mem.address_space;
    let mut addr = low;
    while addr <= high {
        if addr == pc_addr as u64 {
            print!("\t>");
        } else {
            print!("\t ");
        }
        print!(" 0x{:08x}: ", addr);
        // TODO: Is the u-boot.bin code really little endian, or is
        // there a problem with how the bits are printed?
        //
        // There are claims that RPI2 is little endian by default, and
        // pre-compiled big-endian kernels aren't well known.
        // (see http://raspberrypi.stackexchange.com/questions/7279/big-endian-distribution-for-the-raspberry-pi)
        match computer.mem.get32(addr, false) {
            None => println!("[unitialized]"),
            Some(word) => {
                match computer.cpu.decode_instruction(word) {
                    None => println!("[invalid or unrecognized instruction]: {:032b}", word),
                    Some(instr) => println!("{:?}", instr),
                }
            },
        }
        addr += 4;
    }
    println!("PC Address: 0x{:08x}", pc_addr);
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

fn main() {
    println!("
ARMOR Debugging Interface
=========================
");

    // First several instructions from a u-boot binary compiled on a
    // Raspberry Pi 2.
    let boot_code = vec![
        // Vector table starting at 0x00000000.
        0xbe, 0x00, 0x00, 0xea, // Reset (supervisor)
        0x14, 0xf0, 0x9f, 0xe5, // Undefined instruction (undefined)
        0x14, 0xf0, 0x9f, 0xe5, // Software interrupt (supervisor)
        0x14, 0xf0, 0x9f, 0xe5, // Abort - prefetch (abort)
        0x14, 0xf0, 0x9f, 0xe5, // Abort - data (abort)
        0x14, 0xf0, 0x9f, 0xe5, // Reserved (reserved)
        0x14, 0xf0, 0x9f, 0xe5, // IRQ
        0x14, 0xf0, 0x9f, 0xe5, // FIQ
        // End of vector table
        0x60, 0x80, 0x00, 0x00,
        0xc0, 0x80, 0x00, 0x00,
    ];
    let computer = &mut Computer::new(boot_code);
    debugger_repl(computer).is_ok();
}
