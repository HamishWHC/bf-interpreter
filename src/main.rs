use std::io::prelude::*;
use std::process;

const TAPE_SIZE: usize = 30000;

fn main() -> () {
    let args: Vec<String> = std::env::args().collect();
    let argument = &args.get(1);
    match argument {
        Some(argument) => {
            if argument.as_str() == "--help" {
                println!("Usage: {} <filename>\nNot specifying a filename or piping input to stdin will open an interative shell.", args.get(0).unwrap());
                process::exit(0);
            } else {
                let contents = std::fs::read_to_string(argument).unwrap_or_else(|error| {
                    eprintln!("File read error: {}", error);
                    process::exit(1);
                });
                let program = parse(&contents);
                execute(&program, &mut 0, &mut [0; TAPE_SIZE]);
            }
        }
        _ => {
            if atty::isnt(atty::Stream::Stdin) {
                let buf = std::io::stdin().bytes().map(|b| b.unwrap());
                let program = parse(&buf.map(|b| char::from(b)).collect::<String>());
                execute(&program, &mut 0, &mut [0; TAPE_SIZE]);
            } else {
                // shell();
                eprintln!("Usage: {} <filename>", args.get(0).unwrap());
                process::exit(1);
            }
        }
    };
}

const HELP_MSG: &str = "Enter 'help' for this help message and 'exit' to exit.";

fn shell() {
    println!("Entering interactive shell. {}", HELP_MSG);
    let mut input = String::new();
    while input.as_str() != "exit" {
        input = String::new();
        print!(">>> ");
        if let Err(e) = std::io::stdin().read_line(&mut input) {
            eprintln!("Stdin read error: {}", e);
            process::exit(1);
        }
        if input.as_str() == "help" {
            println!("{}", HELP_MSG);
        } else {
            let program = parse(&input);
            execute(&program, &mut 0, &mut [0; TAPE_SIZE]);
        }
    }
}

type Series = Vec<Node>;

#[derive(Debug, Clone)]
enum Node {
    Left,
    Right,
    Increment,
    Decrement,
    Output,
    InputReplace,
    Loop(Series),
}

fn parse(string: &str) -> Series {
    let mut program = vec![];
    let mut chars = string.chars();
    while let Some(ch) = &chars.next() {
        let node = match ch {
            '<' => Some(Node::Left),
            '>' => Some(Node::Right),
            '+' => Some(Node::Increment),
            '-' => Some(Node::Decrement),
            '.' => Some(Node::Output),
            ',' => Some(Node::InputReplace),
            '[' => {
                let mut loop_level = 1;
                let mut sub_string: String = String::new();
                for ss_ch in chars.clone() {
                    if ss_ch == '[' {
                        loop_level += 1;
                    } else if ss_ch == ']' {
                        loop_level -= 1;
                        if loop_level == 0 {
                            break;
                        }
                    }
                    sub_string.push(ss_ch);
                }
                if loop_level > 0 {
                    eprintln!("Loop not closed.");
                    process::exit(1);
                }
                for _ in 0..sub_string.len() + 1 {
                    chars.next();
                }
                Some(Node::Loop(parse(&sub_string)))
            }
            _ => None,
        };
        if let Some(n) = node {
            program.push(n)
        };
    }
    program
}

fn execute(program: &Series, addr_pointer: &mut usize, tape: &mut [u8; TAPE_SIZE]) -> () {
    for node in program {
        match node {
            Node::Left => {
                if *addr_pointer == 0 {
                    *addr_pointer = TAPE_SIZE - 1;
                } else {
                    *addr_pointer -= 1;
                }
            }
            Node::Right => {
                if *addr_pointer == TAPE_SIZE - 1 {
                    *addr_pointer = 0;
                } else {
                    *addr_pointer += 1;
                }
            }
            Node::Increment => {
                if tape[*addr_pointer] == 255 {
                    tape[*addr_pointer] = 0;
                } else {
                    tape[*addr_pointer] += 1;
                }
            }
            Node::Decrement => {
                if tape[*addr_pointer] == 0 {
                    tape[*addr_pointer] = 255;
                } else {
                    tape[*addr_pointer] -= 1;
                }
            }
            Node::Output => print!("{}", char::from(tape[*addr_pointer])),
            Node::InputReplace => {
                eprintln!("The ',' operator is not supported.");
                process::exit(1);
            }
            Node::Loop(sub_program) => {
                if tape[*addr_pointer] != 0 {
                    execute(sub_program, addr_pointer, tape)
                }
                while tape[*addr_pointer] > 0 {
                    execute(sub_program, addr_pointer, tape)
                }
            }
        };
    }
}
