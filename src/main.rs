use std::fs::read_to_string;
use std::fs::File;
use std::io::prelude::*;
use std::io::{stdin, Read};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    for pos in 2..args.len() {
        match &*args[1] {
            "-b" => build_bin(&args[pos]),

            _ => execute(&args[pos]),
        }
    }
}

fn read_in(path: &str) -> String {
    match read_to_string(path) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("Err: {}", err);
            std::process::exit(1);
        }
    }
}

fn execute(path: &str) {
    let file = read_in(path);
    let mut buffer: [u8; 30000] = [0; 30000];
    let mut stc_ptr = 0;
    let mut progr = 0;
    let mut stack = Vec::<(char, usize)>::new();
    let chars: Vec<char> = file.chars().collect();
    let mut out: Vec<u8> = Vec::new();
    loop {
        let _chars = chars[progr];
        match _chars {
            '.' => out.push(buffer[stc_ptr]),
            ',' => {
                buffer[stc_ptr] = match stdin().bytes().nth(0) {
                    Some(val) => match val {
                        Ok(val) => val,
                        Err(_) => std::process::exit(1),
                    },
                    None => std::process::exit(1),
                }
            }
            '<' => stc_ptr -= 1,
            '>' => stc_ptr += 1,
            '+' => buffer[stc_ptr] += 1,
            '-' => buffer[stc_ptr] -= 1,
            '[' => {
                if buffer[stc_ptr] == 0 {
                    let mut deep = 1;
                    progr += 1;
                    loop {
                        match chars[progr] {
                            '[' => deep += 1,
                            ']' => deep -= 1,
                            _ => (),
                        }
                        progr += 1;
                        if deep == 0 {
                            break;
                        }
                    }
                    continue;
                } else {
                    stack.push(('[', progr));
                }
            }
            ']' => {
                let i = stack.last();
                if let Some(('[', val)) = i {
                    if buffer[stc_ptr] != 0 {
                        progr = val + 1;
                    } else {
                        stack.pop();
                    }
                } else {
                    eprintln!("Unmatched ']'");
                    std::process::exit(1);
                }
            }
            _ => (),
        }
        progr += 1;

        if progr >= chars.len() {
            let s = match std::str::from_utf8(&out[..]) {
                Ok(v) => v,
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };

            println!("output: {}", s);
            break;
        }
    }
}

fn build_bin(path: &str) {
    let file = read_in(path);
    let mut stack = Vec::<char>::new();
    let chars: Vec<char> = file.chars().collect();
    let mut bytecode = Vec::<&str>::new();
    let mut progr = 0;

    loop {
        match chars[progr] {
            '.' => bytecode.push("WRITE"),
            ',' => bytecode.push("INPUT"),
            '<' => bytecode.push("NEXT"),
            '>' => bytecode.push("BACKE"),
            '+' => bytecode.push("INCREASE"),
            '-' => bytecode.push("DECREASE"),
            '[' => {
                bytecode.push("FOR");
                stack.push('[');
                let mut unmatch = progr + 1;
                let mut deep = 1;
                loop {
                    match chars[unmatch] {
                        '[' => deep += 1,
                        ']' => deep -= 1,
                        _ => (),
                    }
                    unmatch += 1;
                    if deep == 0 {
                        break;
                    }
                    if unmatch >= chars.len() {
                        panic!("Unmatched ]");
                    }
                }
            }
            ']' => {
                let i = stack.last();
                if let Some('[') = i {
                    stack.pop();
                    bytecode.push("END");
                } else {
                    eprintln!("Unmatched ']'");
                    std::process::exit(1);
                }
            }
            _ => (),
        }

        progr += 1;
        if progr >= chars.len() {
            break;
        }
    }
    let mut out_file = match File::create(format!("{}.bbf", path)) {
        Ok(val) => val,
        Err(_) => match File::open(format!("{}.bbf", path)) {
            Ok(val) => val,
            Err(err) => {
                panic!("{}", err);
            }
        },
    };
    for instr in bytecode {
        out_file.write_all(instr.as_bytes()).unwrap();
        out_file.write_all(b"\n").unwrap();
    }
}
