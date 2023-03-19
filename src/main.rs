use std::fs::read_to_string;
use std::io::{stdin, Read};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    for pos in 1..args.len() {
        execute(&*args[pos]);
    }
}

fn read_in(path: &str) -> String {
    let file = match read_to_string(path) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("Err: {}", err);
            std::process::exit(1);
        }
    };
    file
}

fn execute(path: &str) {
    let file = read_in(path);
    let mut buffer: [u8; 30000] = [0; 30000];
    let mut stc_ptr = 0;
    let mut progr = 0;
    let mut stack = Vec::<(char, usize)>::new();
    let chars: Vec<char> = file.chars().collect();
    loop {
        let _chars = chars[progr];
        match _chars {
            '.' => println!("{}", buffer[stc_ptr]),
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
            std::process::exit(0);
        }
    }
}
