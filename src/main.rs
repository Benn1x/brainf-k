use std::fs::File;
use std::fs::{read, read_to_string};
use std::io::prelude::*;
use std::io::{stdin, Read};
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() <= 2 {
        println!("-b: build Binary Code [].bbf\n-i: run created Binary [].bf\n-l Build it Using LLVM (Not Implemented (yet)) [].bf\n-r: Run the String Code [].bf");
    }
    for pos in 2..args.len() {
        match &*args[1] {
            "-b" => {
                write(build_bin(&args[pos]), &args[pos]);
            }
            "-i" => interpret(&args[pos]),
            "-l" => llvm(&args[pos]),
            "-bench" => {
                let start = Instant::now();
                for _ in 0..500000 {
                    build_bin(&args[pos]);
                }
                let duration = start.elapsed();
                println!(
                    "Time elapsed in total {:?} for each operation in average {:?}",
                    duration,
                    duration / 500000
                );
            }

            "-r" => execute(&args[pos]),
            _ => (),
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

fn read_byte(path: &str) -> Vec<u8> {
    match read(path) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("Err: {}", err);
            std::process::exit(1);
        }
    }
}
fn write(bytes: Vec<u8>, path: &str) {
    let mut out_file = match File::create(format!("{}.bbf", path)) {
        Ok(val) => val,
        Err(_) => match File::open(format!("{}.bbf", path)) {
            Ok(val) => val,
            Err(err) => {
                panic!("{}", err);
            }
        },
    };
    out_file.write_all(&bytes[..]).expect("Fehler");
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

/// CHAR NUM PAR   OPR
///   . = 1   0  (WRITE)
///   , = 2   0  (READ)
///   < = 3   1  (LEFT)
///   > = 4   1  (RIGHT)
///   + = 5   1  (INCREASE)
///   - = 6   1  (DECREASE)
///   [ = 7   0  (START FOR LOOP)
///   ] = 8   0  (END FOR LOOP)

fn build_bin(path: &str) -> Vec<u8> {
    let file = read_in(path);
    let mut stack = Vec::<(char, u8)>::new();
    let chars: Vec<u8> = file.bytes().collect();
    let mut bytecode = Vec::<u8>::new();
    let mut progr = 0;

    loop {
        match chars[progr] {
            b'.' => bytecode.push(1),
            b',' => bytecode.push(2),
            b'<' => {
                let mut left = progr + 1;
                while let b'<' = chars[left] {
                    left += 1;
                    if left >= chars.len() {
                        break;
                    }
                }
                bytecode.push(3);
                bytecode.push((left - progr) as u8);
                progr = left - 1;
            }
            b'>' => {
                let mut right = progr + 1;
                while let b'>' = chars[right] {
                    right += 1;
                    if right >= chars.len() {
                        break;
                    }
                }
                bytecode.push(4);
                bytecode.push((right - progr) as u8);
                progr = right - 1;
            }
            b'+' => {
                let mut plus = progr + 1;
                while let b'+' = chars[plus] {
                    plus += 1;
                    if plus >= chars.len() {
                        break;
                    }
                }
                bytecode.push(5);
                bytecode.push((plus - progr) as u8);
                progr = plus - 1;
            }
            b'-' => {
                let mut minus = progr + 1;
                while let b'-' = chars[minus] {
                    minus += 1;
                    if minus >= chars.len() {
                        break;
                    }
                }
                bytecode.push(6);
                bytecode.push((minus - progr) as u8);
                progr = minus - 1;
            }
            b'[' => {
                bytecode.push(7);
                let mut unmatch = progr + 1;
                let mut deep = 1;
                loop {
                    match chars[unmatch] {
                        b'[' => deep += 1,
                        b']' => deep -= 1,
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
                stack.push(('[', unmatch as u8));
                bytecode.push(unmatch as u8);
            }
            b']' => {
                let i = stack.last();
                if let Some(('[', val)) = i {
                    bytecode.push(8);
                    bytecode.push(*val);
                    stack.pop();
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
    return bytecode;
}

fn interpret(path: &str) {
    let file = read_byte(path);
    let mut buffer: [u8; 30000] = [0; 30000];
    let mut stc_ptr = 0;
    let mut progr = 0;
    let mut stack = Vec::<(u8, usize)>::new();
    let mut out: Vec<u8> = Vec::new();

    loop {
        match file[progr] {
            // .
            1 => out.push(buffer[stc_ptr]),
            // ,
            2 => {
                buffer[stc_ptr] = match stdin().bytes().nth(0) {
                    Some(val) => match val {
                        Ok(val) => val,
                        Err(_) => std::process::exit(1),
                    },
                    None => std::process::exit(1),
                }
            }
            // <
            3 => {
                stc_ptr -= file[progr + 1] as usize;

                progr += 1;
            }
            // >
            4 => {
                stc_ptr += file[progr + 1] as usize;

                progr += 1;
            }
            // +
            5 => {
                buffer[stc_ptr] += file[progr + 1];
                progr += 1;
            }
            // -
            6 => {
                buffer[stc_ptr] -= file[progr + 1];
                progr += 1;
            }
            // [
            7 => {
                if buffer[stc_ptr] == 0 {
                    stc_ptr = file[progr + 1] as usize;
                } else {
                    stack.push((7, progr));
                }
            }
            // ]
            8 => {
                let i = stack.last();
                if let Some((7, val)) = i {
                    if buffer[stc_ptr] != 0 {
                        progr = *val;
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

        if progr >= file.len() {
            let s = match std::str::from_utf8(&out[..]) {
                Ok(v) => v,
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };

            println!("output: {}", s);
            break;
        }
    }
}

fn llvm(path: &str) {
    let _ = read_in(path);
}
