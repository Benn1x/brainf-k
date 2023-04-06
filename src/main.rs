use llvm_sys::{
    core::{LLVMFunctionType, LLVMInt32TypeInContext, LLVMTypeOf, LLVMAddFunction, LLVMVoidType, LLVMSetParamAlignment, LLVMAddAttributeAtIndex, LLVMArrayType, LLVMInt8Type, LLVMPrintTypeToString, LLVMPrintModuleToString, LLVMPointerType, LLVMVoidTypeInContext, LLVMPointerTypeInContext, LLVMDumpModule, LLVMAppendBasicBlockInContext, LLVMPositionBuilderAtEnd, LLVMBuildMalloc, LLVMInt8TypeInContext, LLVMBuildArrayAlloca, LLVMConstInt, LLVMConstArray, LLVMBuildAlloca},
    prelude::*,
};

use std::{fs::read, ptr::null_mut};
use std::fs::File;
use std::io::prelude::*;
use std::io::{stdin, Read};
use std::time::Instant;

use std::ffi::{CStr, CString};

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
    let mut buffer: [u8; 30000] = [0; 30000];
    let mut stc_ptr = 0;
    let mut progr = 0;
    let mut stack = Vec::<(u8, usize)>::new();
    let chars: Vec<u8> = read_byte(path);
    let mut out: Vec<u8> = Vec::new();
    loop {
        let _chars = chars[progr];
        match _chars {
            b'.' => out.push(buffer[stc_ptr]),
            b',' => {
                buffer[stc_ptr] = match stdin().bytes().nth(0) {
                    Some(val) => match val {
                        Ok(val) => val,
                        Err(_) => std::process::exit(1),
                    },
                    None => std::process::exit(1),
                }
            }
            b'<' => stc_ptr -= 1,
            b'>' => stc_ptr += 1,
            b'+' => buffer[stc_ptr] += 1,
            b'-' => buffer[stc_ptr] -= 1,
            b'[' => {
                if buffer[stc_ptr] == 0 {
                    let mut deep = 1;
                    progr += 1;
                    loop {
                        match chars[progr] {
                            b'[' => deep += 1,
                            b']' => deep -= 1,
                            _ => (),
                        }
                        progr += 1;
                        if deep == 0 {
                            break;
                        }
                    }
                    continue;
                } else {
                    stack.push((b'[', progr));
                }
            }
            b']' => {
                let i = stack.last();
                if let Some((b'[', val)) = i {
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
    let mut stack = Vec::<(char, u8)>::new();
    let chars: Vec<u8> = read_byte(path);
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
    bytecode
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
    let _ = read_byte(path);
    let mut llvm = LLVM::new();
    llvm.code_gen(path);
    llvm.dump();

}

fn cstr(s: &str) -> Box<CStr> {
    CString::new(s).unwrap().into_boxed_c_str()
}

struct LLVM {
    ctx: LLVMContextRef,
    builder: LLVMBuilderRef,
    module: LLVMModuleRef,
}

impl LLVM {
    fn new() -> LLVM {
        let name = cstr("BrainFuck");
        unsafe {
            let ctx = llvm_sys::core::LLVMContextCreate();
            let builder = llvm_sys::core::LLVMCreateBuilder();
            let module = llvm_sys::core::LLVMModuleCreateWithNameInContext(name.as_ptr(), ctx);
            LLVM {
                ctx,
                builder,
                module,
            }
        }
    }

    pub fn code_gen(&mut self, path: &str) {
        let file = read_byte(path);
        let chars: Vec<u8> = read_byte(path);
        let mut ptr = 0;
        let main_fn = self.create_main();
        unsafe{
        let m_block = LLVMAppendBasicBlockInContext(self.ctx, main_fn, cstr("").as_ptr());
        LLVMPositionBuilderAtEnd(self.builder, m_block);
        let elem_type = LLVMArrayType(LLVMInt8TypeInContext(self.ctx), 30000);
        let _array_ptr = LLVMBuildArrayAlloca(self.builder, elem_type, LLVMConstInt(LLVMInt8TypeInContext(self.ctx), 1, 0), cstr("array").as_ptr());
        let _ptr = LLVMBuildAlloca(self.builder, LLVMInt8TypeInContext(self.ctx), cstr("ptr").as_ptr());
        
    }
    }

    pub fn create_main(&self)-> LLVMValueRef{
        unsafe{
            let mut void = null_mut();
            let mut fn_type = LLVMFunctionType(LLVMInt32TypeInContext(self.ctx), &mut void , 0 , 0);
            return LLVMAddFunction(self.module, cstr("main").as_ptr(), fn_type);

        }
    }

    pub fn create_fn(&mut self, name: &str){
        unsafe{
            let mut arry = LLVMPointerTypeInContext(self.ctx, 0);
            let mut fn_type = LLVMFunctionType(LLVMVoidTypeInContext(self.ctx), &mut arry, 1 , 0);
            let mut func = LLVMAddFunction(self.module, cstr(name).as_ptr(), fn_type);
            LLVMDumpModule(self.module);
            
        }
    }

    pub fn dump(&self){

        unsafe {LLVMDumpModule(self.module);}
    }
}
