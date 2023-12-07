#![deny(warnings)]

use glium::Surface;
use std::println;
use llvm_sys::core::LLVMBuildBr;
use llvm_sys::core::LLVMBuildCondBr;
use llvm_sys::core::LLVMBuildICmp;
use llvm_sys::LLVMIntPredicate::LLVMIntEQ;
use llvm_sys::{
    analysis::LLVMVerifyModule,
    core::{
        LLVMAddFunction, LLVMAppendBasicBlockInContext, LLVMBuildAdd, LLVMBuildAlloca,
        LLVMBuildCall2, LLVMBuildFree, LLVMBuildGEP2, LLVMBuildLoad2, LLVMBuildRetVoid,
        LLVMBuildStore, LLVMBuildSub, LLVMBuildTrunc, LLVMBuildZExt, LLVMConstInt,
        LLVMConstPointerNull, LLVMDumpModule, LLVMFunctionType, LLVMGetParam,
        LLVMInt32TypeInContext, LLVMInt64TypeInContext, LLVMInt8TypeInContext,
        LLVMPointerTypeInContext, LLVMPositionBuilderAtEnd, LLVMVoidTypeInContext,
    },
    prelude::*,
    target::{LLVM_InitializeNativeAsmPrinter, LLVM_InitializeNativeTarget},
    target_machine::{
        LLVMCreateTargetMachine, LLVMDisposeTargetMachine, LLVMGetFirstTarget,
        LLVMGetTargetFromTriple, LLVMTargetMachineEmitToFile,
    },
    LLVMValue,
};
use winit::event::Event;
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;

use std::collections::HashMap;
use std::io::prelude::*;
use std::io::{stdin, Read};
use std::time::Instant;

use std::{fs::read, ptr::null_mut};
use std::{fs::File, process::Command};

use std::ffi::{CStr, CString};

use imgui::*;
use winit::event_loop::EventLoop;
use regex::Regex;

const VERSION: &str = env!("CARGO_PKG_VERSION");

// TODO: CLI
fn main() {
    std::env::set_var("WINIT_UNIX_BACKEND", "x11");
    let args: Vec<String> = std::env::args().collect();
    if args.len() <= 2 {
        println!("-b: build Binary Code [].bbf\n-i: run created Binary [].bf\n-l Build it Using LLVM [].bf\n-r: Run the String Code [].bf\n You are using the Version {}", VERSION);
    }

    for pos in 2..args.len() {
        match &*args[1] {
            "-b" => {
                write(build_bin(&args[pos]), &args[pos]);
            }
            "-i" => interpret(&args[pos]),
            "-l" => {
                if &args[pos] == "info" {
                    println!("Requirements:\n\t-clang\n\t-the main.c https://github.com/Benn1x/brainf-k/blob/master/main.c or your own");
                } else {
                    llvm(&args[pos])
                }
            }
            "-bench" => {
                if &args[pos] == "build" {
                    let start = Instant::now();
                    for _ in 0..500000 {
                        build_bin(&args[pos + 1]);
                    }
                    let duration = start.elapsed();
                    println!(
                        "Time elapsed in total {:?} for each operation in average {:?}",
                        duration,
                        duration / 500000
                    );
                }
                if &args[pos] == "exec" {
                    let start = Instant::now();
                    for _ in 0..500000 {
                        execute(&args[pos + 1]);
                    }
                    let duration = start.elapsed();
                    println!(
                        "Time elapsed in total {:?} for each operation in average {:?}",
                        duration,
                        duration / 500000
                    );
                }
            }
            "-v" => println!("{}", VERSION),

            "-r" => execute(&args[pos]),
            "-d" => {
                println!("{}", String::from_utf8_lossy(&read_byte(&args[pos])));
            }

            "-a" => {
                Analyzer::new(&args[pos]).analyze_gui();
            }
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

fn write_(msg: &mut String, path: &str) {
    let mut out_file = match File::create(path) {
        Ok(val) => val,
        Err(_) => match File::open(path) {
            Ok(val) => val,
            Err(err) => {
                panic!("{}", err);
            }
        },
    };
    out_file.write_all(msg.as_bytes()).expect("Fehler");
}

fn parse(s: &str) -> String {
    let pattern = Regex::new(r"[^\+\-\.,\[\]]").unwrap();
    pattern.replace_all(s, "").to_string()
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
            b'<' => stc_ptr = stc_ptr.wrapping_sub(1),
            b'>' => stc_ptr = stc_ptr.wrapping_add(1),
            b'+' => buffer[stc_ptr] = buffer[stc_ptr].wrapping_add(1),
            b'-' => buffer[stc_ptr] = buffer[stc_ptr].wrapping_sub(1),
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
                stc_ptr = stc_ptr.wrapping_sub(file[progr + 1] as usize);

                progr += 1;
            }
            // >
            4 => {
                stc_ptr = stc_ptr.wrapping_add(file[progr + 1] as usize);

                progr += 1;
            }
            // +
            5 => {
                buffer[stc_ptr] = buffer[stc_ptr].wrapping_add(file[progr + 1]);
                progr += 1;
            }
            // -
            6 => {
                buffer[stc_ptr] = buffer[stc_ptr].wrapping_sub(file[progr + 1]);
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
    let mut llvm = LLVM::new();
    llvm.syntax_check(path);
    llvm.code_gen(path);
    llvm.dump();
    llvm.generate();
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

    pub fn syntax_check(&self, path: &str){
        let chars: Vec<u8> = read_byte(path);
        let mut progr = 0;
        let mut stack = Vec::<(char, u8)>::new();
        loop {
            match chars[progr] {

                b'[' => {
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
                }
                b']' => {
                    let i = stack.last();
                    if let Some(('[', _val)) = i {
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
    }

    pub fn code_gen(&mut self, path: &str) {
        // BASIC BLOCKS END WITH BR/OR RET STMT NEED TO REBUILD BLOCKS
        let chars: Vec<u8> = read_byte(path);
        let mut ptr = 0;
        unsafe {
            // we need a vector to keep track of the fro loops/for loops ends
            let main_fn = self.create_fn("exec");
            let arr = LLVMGetParam(main_fn, 0);
            let m_block = LLVMAppendBasicBlockInContext(self.ctx, main_fn, cstr("").as_ptr());
            let _for_end = Vec::<LLVMBasicBlockRef>::new();
            let _jump = Vec::<LLVMBasicBlockRef>::new();
            LLVMPositionBuilderAtEnd(self.builder, m_block);
            let getchar_head = LLVMFunctionType(LLVMInt32TypeInContext(self.ctx), null_mut(), 0, 0);
            let putchar_head = LLVMFunctionType(
                LLVMInt32TypeInContext(self.ctx),
                &mut LLVMInt32TypeInContext(self.ctx),
                1,
                0,
            );
            let getchar = LLVMAddFunction(self.module, cstr("getchar").as_ptr(), getchar_head);
            let putchar = LLVMAddFunction(self.module, cstr("putchar").as_ptr(), putchar_head);

            let i_ptr = LLVMBuildAlloca(
                self.builder,
                LLVMInt32TypeInContext(self.ctx),
                cstr("ptr").as_ptr(),
            );
            let addend = LLVMConstInt(LLVMInt32TypeInContext(self.ctx), 0 as u64, 0);
            LLVMBuildStore(self.builder, addend, i_ptr);

            let mut depth_vec: Vec<LLVMBasicBlockRef> = Vec::new();

            loop {
                match chars[ptr] {
                    b'>' => {
                        let mut left = ptr + 1;
                        while let b'>' = chars[left] {
                            left += 1;
                            if left >= chars.len() {
                                break;
                            }
                        }

                        let ptr_value = LLVMBuildLoad2(
                            self.builder,
                            LLVMInt32TypeInContext(self.ctx),
                            i_ptr,
                            cstr("").as_ptr(),
                        );
                        let addend =
                            LLVMConstInt(LLVMInt32TypeInContext(self.ctx), (left - ptr) as u64, 0);
                        let new_value =
                            LLVMBuildAdd(self.builder, ptr_value, addend, cstr("").as_ptr());
                        LLVMBuildStore(self.builder, new_value, i_ptr);
                        ptr = left - 1;
                    }
                    b'<' => {
                        let mut left = ptr + 1;
                        while let b'<' = chars[left] {
                            left += 1;
                            if left >= chars.len() {
                                break;
                            }
                        }

                        let ptr_value = LLVMBuildLoad2(
                            self.builder,
                            LLVMInt32TypeInContext(self.ctx),
                            i_ptr,
                            cstr("").as_ptr(),
                        );

                        let addend =
                            LLVMConstInt(LLVMInt32TypeInContext(self.ctx), (left - ptr) as u64, 0);
                        let new_value =
                            LLVMBuildSub(self.builder, ptr_value, addend, cstr("").as_ptr());
                        LLVMBuildStore(self.builder, new_value, i_ptr);
                        ptr = left - 1;
                    }
                    b'+' => {
                        let mut plus = ptr + 1;
                        while let b'+' = chars[plus] {
                            plus += 1;
                            if plus >= chars.len() {
                                break;
                            }
                        }
                        let pos = LLVMBuildLoad2(
                            self.builder,
                            LLVMInt32TypeInContext(self.ctx),
                            i_ptr,
                            cstr("").as_ptr(),
                        );

                        let mut prm = LLVMBuildZExt(
                            self.builder,
                            pos,
                            LLVMInt64TypeInContext(self.ctx),
                            cstr("").as_ptr(),
                        );

                        let elem_ptr = LLVMBuildGEP2(
                            self.builder,
                            LLVMInt8TypeInContext(self.ctx),
                            arr,
                            &mut prm,
                            1,
                            cstr("").as_ptr(),
                        );
                        let elem_value = LLVMBuildLoad2(
                            self.builder,
                            LLVMInt8TypeInContext(self.ctx),
                            elem_ptr,
                            cstr("").as_ptr(),
                        );

                        // Finally, increase the value by one
                        let new_value = LLVMBuildAdd(
                            self.builder,
                            elem_value,
                            LLVMConstInt(LLVMInt8TypeInContext(self.ctx), (plus - ptr) as u64, 0),
                            cstr("").as_ptr(),
                        );
                        LLVMBuildStore(self.builder, new_value, elem_ptr);
                        ptr = plus - 1;
                    }
                    b'-' => {
                        let mut plus = ptr + 1;
                        while let b'-' = chars[plus] {
                            plus += 1;
                            if plus >= chars.len() {
                                break;
                            }
                        }
                        let pos = LLVMBuildLoad2(
                            self.builder,
                            LLVMInt32TypeInContext(self.ctx),
                            i_ptr,
                            cstr("").as_ptr(),
                        );

                        let mut prm = LLVMBuildZExt(
                            self.builder,
                            pos,
                            LLVMInt64TypeInContext(self.ctx),
                            cstr("").as_ptr(),
                        );

                        let elem_ptr = LLVMBuildGEP2(
                            self.builder,
                            LLVMInt8TypeInContext(self.ctx),
                            arr,
                            &mut prm,
                            1,
                            cstr("").as_ptr(),
                        );
                        let elem_value = LLVMBuildLoad2(
                            self.builder,
                            LLVMInt8TypeInContext(self.ctx),
                            elem_ptr,
                            cstr("").as_ptr(),
                        );

                        // Finally, increase the value by one
                        let new_value = LLVMBuildSub(
                            self.builder,
                            elem_value,
                            LLVMConstInt(LLVMInt8TypeInContext(self.ctx), (plus - ptr) as u64, 0),
                            cstr("").as_ptr(),
                        );
                        LLVMBuildStore(self.builder, new_value, elem_ptr);
                        ptr = plus - 1;
                    }
                    b'[' => {
                        let for_block = LLVMAppendBasicBlockInContext(
                            self.ctx,
                            main_fn,
                            cstr("header").as_ptr(),
                        );
                        depth_vec.push(for_block);
                        LLVMBuildBr(self.builder, for_block);
                        LLVMPositionBuilderAtEnd(self.builder, for_block);
                        // check if the current value is 0, the current value is the value at the pointers position in the array
                        let pos = LLVMBuildLoad2(
                            self.builder,
                            LLVMInt32TypeInContext(self.ctx),
                            i_ptr,
                            cstr("").as_ptr(),
                        );

                        let mut prm = LLVMBuildZExt(
                            self.builder,
                            pos,
                            LLVMInt64TypeInContext(self.ctx),
                            cstr("").as_ptr(),
                        );

                        let elem_ptr = LLVMBuildGEP2(
                            self.builder,
                            LLVMInt8TypeInContext(self.ctx),
                            arr,
                            &mut prm,
                            1,
                            cstr("").as_ptr(),
                        );
                        let elem_value = LLVMBuildLoad2(
                            self.builder,
                            LLVMInt8TypeInContext(self.ctx),
                            elem_ptr,
                            cstr("").as_ptr(),
                        );

                        // check if elem_value is 0
                        let if_cond = LLVMBuildICmp(
                            self.builder,
                            LLVMIntEQ,
                            elem_value,
                            LLVMConstInt(LLVMInt8TypeInContext(self.ctx), 0, 0),
                            cstr("").as_ptr(),
                        );

                        // make a cond jump:
                        // if elem_value == 0 jump to for_end_block
                        // else jump to for_block
                        // runs the loop
                        let for_loop = LLVMAppendBasicBlockInContext(
                            self.ctx,
                            main_fn,
                            cstr("loop").as_ptr(),
                        );
                        //after the loop
                        let for_end_block = LLVMAppendBasicBlockInContext(
                            self.ctx,
                            main_fn,
                            cstr("end").as_ptr(),
                        );
                        depth_vec.push(for_end_block);

                        LLVMBuildCondBr(self.builder, if_cond, for_end_block, for_loop);
                        // now position the builder at the end of the loop block
                        LLVMPositionBuilderAtEnd(self.builder, for_loop);
                    }

                    b']' => {
                        let end = depth_vec.pop().unwrap();
                        let loop_ = depth_vec.pop().unwrap();
                        LLVMBuildBr(self.builder, loop_);
                        LLVMPositionBuilderAtEnd(self.builder, end);

                    }

                    b'.' => {
                        let pos = LLVMBuildLoad2(
                            self.builder,
                            LLVMInt32TypeInContext(self.ctx),
                            i_ptr,
                            cstr("").as_ptr(),
                        );

                        let mut prm = LLVMBuildZExt(
                            self.builder,
                            pos,
                            LLVMInt64TypeInContext(self.ctx),
                            cstr("").as_ptr(),
                        );

                        let elem_ptr = LLVMBuildGEP2(
                            self.builder,
                            LLVMInt8TypeInContext(self.ctx),
                            arr,
                            &mut prm,
                            1,
                            cstr("").as_ptr(),
                        );

                        let mut trunc = LLVMBuildLoad2(
                            self.builder,
                            LLVMInt32TypeInContext(self.ctx),
                            elem_ptr,
                            cstr("").as_ptr(),
                        );
                        LLVMBuildCall2(
                            self.builder,
                            putchar_head,
                            putchar,
                            &mut trunc,
                            1,
                            cstr("").as_ptr(),
                        );
                    }
                    b',' => {
                        let input = LLVMBuildCall2(
                            self.builder,
                            getchar_head,
                            getchar,
                            &mut LLVMConstPointerNull(LLVMVoidTypeInContext(self.ctx)),
                            0,
                            cstr("").as_ptr(),
                        );

                        let pos = LLVMBuildLoad2(
                            self.builder,
                            LLVMInt32TypeInContext(self.ctx),
                            i_ptr,
                            cstr("").as_ptr(),
                        );

                        let mut prm = LLVMBuildZExt(
                            self.builder,
                            pos,
                            LLVMInt64TypeInContext(self.ctx),
                            cstr("").as_ptr(),
                        );

                        let elem_ptr = LLVMBuildGEP2(
                            self.builder,
                            LLVMInt8TypeInContext(self.ctx),
                            arr,
                            &mut prm,
                            1,
                            cstr("").as_ptr(),
                        );
                        let trunc = LLVMBuildTrunc(
                            self.builder,
                            input,
                            LLVMInt8TypeInContext(self.ctx),
                            cstr("").as_ptr(),
                        );
                        LLVMBuildStore(self.builder, trunc, elem_ptr);
                    }
                    b'*' => {
                        let trunc = LLVMBuildLoad2(
                            self.builder,
                            LLVMInt32TypeInContext(self.ctx),
                            i_ptr,
                            cstr("").as_ptr(),
                        );
                        let mut add = LLVMBuildAdd(
                            self.builder,
                            trunc,
                            LLVMConstInt(LLVMInt32TypeInContext(self.ctx), 48, 0),
                            cstr("").as_ptr(),
                        );
                        LLVMBuildCall2(
                            self.builder,
                            putchar_head,
                            putchar,
                            &mut add,
                            1,
                            cstr("").as_ptr(),
                        );
                    }
                    _ => (),
                }
                ptr += 1;
                if ptr >= chars.len() {
                    LLVMBuildFree(self.builder, arr);
                    // we need to get the current block
                    // and add a ret void
                    //q: what does the LLVM No predecessors! mean? a: it means that the block is unreachable
                    let end_block =
                        LLVMAppendBasicBlockInContext(self.ctx, main_fn, cstr("end").as_ptr());
                    LLVMBuildBr(self.builder, end_block);
                    LLVMPositionBuilderAtEnd(self.builder, end_block);
                    LLVMBuildRetVoid(self.builder);
                    break;
                }
            }
        }
    }

    pub fn generate(&mut self) {
        unsafe {
            let err = null_mut();
            let mut err_s = null_mut();
            if LLVMVerifyModule(
                self.module,
                llvm_sys::analysis::LLVMVerifierFailureAction::LLVMAbortProcessAction,
                &mut err_s,
            ) == 1
            {
                let x = CStr::from_ptr(err_s);
                panic!("Verifying the Module failed {:?}", x);
            }
            let mut error_str = null_mut();

            LLVM_InitializeNativeTarget();
            LLVM_InitializeNativeAsmPrinter();
            let mut target = LLVMGetFirstTarget();
            if LLVMGetTargetFromTriple(
                "x86_64-unknown-linux-gnu\0".as_ptr() as *const i8,
                &mut target,
                err,
            ) == 1
            {
                let x = CStr::from_ptr(err as *const i8);
                panic!("It failed at Creating a Target! {:?}", x);
            }
            let target_machine = LLVMCreateTargetMachine(
                target,
                "x86_64-unknown-linux-gnu\0".as_ptr() as *const i8,
                cstr("").as_ptr(),
                cstr("").as_ptr(),
                llvm_sys::target_machine::LLVMCodeGenOptLevel::LLVMCodeGenLevelAggressive,
                llvm_sys::target_machine::LLVMRelocMode::LLVMRelocStatic,
                llvm_sys::target_machine::LLVMCodeModel::LLVMCodeModelSmall,
            );
            match LLVMTargetMachineEmitToFile(
                target_machine,
                self.module,
                "exec.o\0".as_ptr() as *mut i8,
                llvm_sys::target_machine::LLVMCodeGenFileType::LLVMObjectFile,
                &mut error_str,
            ) {
                1 => {
                    let x = CStr::from_ptr(error_str);
                    panic!("Generating the Object File failed {:?}", x);
                }
                _ => (),
            }

            let file = File::create("main_link_file.c");
            if let Ok(mut file) = file{
                let _ = file.write_all(b"
#include <stdlib.h>
extern void exec(char *ptr);
int main() {
  char* array = calloc(30000, sizeof(char));
  exec(array);
  return 0;
}

                ");
                LLVMDisposeTargetMachine(target_machine);
                Command::new("clang")
                    .arg("main_link_file.c")
                    .arg("exec.o")
                    .arg("-o")
                    .arg("exec")
                    .output()
                    .expect("Have You installed `clang' and have the main_link_file.c there?");
            }
            else{
                panic!("Unable to cerate file *main_link_file.c*");
            }
        }
    }

    pub fn create_fn(&mut self, name: &str) -> *mut LLVMValue {
        unsafe {
            let mut arry = LLVMPointerTypeInContext(self.ctx, 0);
            let fn_type = LLVMFunctionType(LLVMVoidTypeInContext(self.ctx), &mut arry, 1, 0);
            return LLVMAddFunction(self.module, cstr(name).as_ptr(), fn_type);
        }
    }

    pub fn dump(&self) {
        unsafe {
            LLVMDumpModule(self.module);
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Instr {
    Plus,
    Minus,
    Increase,
    Decrease,
    Read,
    Write,
    LoopStart,
    LoopEnd,
    Null,
}

impl Instr {
    pub fn value(&self) -> isize {
        match &self {
            Self::Read | Self::Write => 10,
            Self::LoopStart | Self::LoopEnd => -1,
            _ => 1,
        }
    }
}
#[derive(Debug, Clone)]
pub struct Analyzer {
    pub pos: usize,
    pub path: String,
    pub score: isize,
    pub comp: HashMap<usize, (Instr, isize)>,
}

impl Analyzer {
    pub fn new(path: &str) -> Self {
        let s_path = String::from(path);
        Self {
            pos: 0,
            path: s_path,
            score: 0,
            comp: HashMap::new(),
        }
    }
    pub fn analyze_gui(&mut self) {
        self.analyze();
        println!("In total the Score of your programm is: {}", self.score);
        let (event_loop, display) = create_window();
        let (mut winit_platform, mut imgui_context) = imgui_init(&display);

        // Create renderer from this crate
        let mut renderer = imgui_glium_renderer::Renderer::init(&mut imgui_context, &display)
            .expect("Failed to initialize renderer");

        // Timer for FPS calculation
        let mut last_frame = std::time::Instant::now();
        let mut analyzer = self.clone();
        let mut min: i32 = 3;
        event_loop.run(move |event, _, control_flow| match event {
            Event::NewEvents(_) => {
                let now = std::time::Instant::now();
                imgui_context.io_mut().update_delta_time(now - last_frame);
                last_frame = now;
            }
            Event::MainEventsCleared => {
                let gl_window = display.gl_window();
                winit_platform
                    .prepare_frame(imgui_context.io_mut(), gl_window.window())
                    .expect("Failed to prepare frame");
                gl_window.window().request_redraw();
            }
            Event::RedrawRequested(_) => {
                // Create frame for the all important `&imgui::Ui`
                let ui = imgui_context.frame();

                ui.window("Analyzer")
                    .size([((analyzer.pos*25) as f32) + 150.0, 600.0], Condition::Always)
                    .build(|| {
                        ui.text(format!(
                            " Total Score of {}: {}",
                            analyzer.path, analyzer.score
                        ));
                        let per_op = analyzer.score as usize / analyzer.pos;
                        ui.text(format!(" Per Operation: {}", per_op));
                        if ui
                            .input_int("Set time per operation", &mut min)
                            .enter_returns_true(true)
                            .build()
                        {}
                        if per_op as i32 >= min {
                            ui.text_colored(
                                [1.0, 0.0, 0.0, 1.0],
                                " Daaaamn your program needs some optimization",
                            );
                        }
                        let mut array = Vec::<f32>::new();
                        for x in 0..=analyzer.comp.len()-1 {
                            let element = match analyzer.comp.get(&x){
                                Some(t) => t,
                                None => &(Instr::Null, 0),
                            };
                            array.push(element.1 as f32);
                        } 
                        ui.plot_lines("heavyness in code", &array).graph_size([(analyzer.pos*25) as f32, 200.0]).build();
                        let mut file_string = String::from_utf8(read_byte(&analyzer.path)).unwrap();
                        if ui.input_text("Editor", &mut file_string ).build() {
                            file_string = parse(&file_string).to_string();
                            if file_string.is_empty() {
                                file_string += "++";
                            }
                            write_(&mut file_string, &analyzer.path);
                            analyzer.analyze();
                        }
                    });

                // Setup for drawing
                let gl_window = display.gl_window();
                let mut target = display.draw();

                // Renderer doesn't automatically clear window
                target.clear_color_srgb(0.0, 0.0, 0.0, 0.0);

                // Perform rendering
                winit_platform.prepare_render(ui, gl_window.window());
                let draw_data = imgui_context.render();
                renderer
                    .render(&mut target, draw_data)
                    .expect("Rendering failed");
                target.finish().expect("Failed to swap buffers");
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            event => {
                let gl_window = display.gl_window();
                winit_platform.handle_event(imgui_context.io_mut(), gl_window.window(), &event);
            }
        });
    }

    pub fn analyze(&mut self){
        let chars = read_byte(&*self.path);
        self.pos = 0;
        self.score = 0;
        self.comp = HashMap::new();
        loop {
            match chars[self.pos] {
                    b'<' =>  match self.comp.insert(self.pos, (Instr::Increase, 1)){
                        Some(_) => panic!("Ühm this should not happen because an position shouldent be writen multiplitimes"),
                        None => (),

                    },
                    
                    b'>' => match self.comp.insert(self.pos, (Instr::Decrease, 1)){
                        Some(_) => panic!("Ühm this should not happen because an position shouldent be writen multiplitimes"),
                        None => (),

                    }

                    b'+' => match self.comp.insert(self.pos, (Instr::Plus, 1)){
                        Some(_) => panic!("Ühm this should not happen because an position shouldent be writen multiplitimes"),
                        None => (),
                    }

                    b'-' => match self.comp.insert(self.pos, (Instr::Minus, 1)){
                        Some(_) => panic!("Ühm this should not happen because an position shouldent be writen multiplitimes"),
                        None => (),
                    }

                    b'[' => match self.comp.insert(self.pos, (Instr::LoopStart, 0)){
                        Some(_) => panic!("Ühm this should not happen because an position shouldent be writen multiplitimes"),
                        None => (),

                    }

                    b']' => match self.comp.insert(self.pos, (Instr::LoopEnd, 0)){
                        Some(_) => panic!("Ühm this should not happen because an position shouldent be writen multiplitimes"),
                        None => (),

                    }

                    b'.' => match self.comp.insert(self.pos, (Instr::Write, 10 )){
                        Some(_) => panic!("Ühm this should not happen because an position shouldent be writen multiplitimes"),
                        None => (),

                    }

                    b',' => match self.comp.insert(self.pos, (Instr::Read, 10)){
                        Some(_) => panic!("Ühm this should not happen because an position shouldent be writen multiplitimes"),
                        None => (),
                    }

                    _ => (),
                }
            self.pos += 1;
            if self.pos >= chars.len() {
                break;
            }
        }
        for i in 0..chars.len() - 1 {
            let instr = self.comp[&i];
            self.score += instr.1;
        }
    }

    pub fn optimize(&mut self) {}
}

fn create_window() -> (EventLoop<()>, glium::Display) {
    let event_loop = EventLoop::new();
    let context = glium::glutin::ContextBuilder::new().with_vsync(true);
    let builder = glium::glutin::window::WindowBuilder::new()
        .with_title("BrainFuck Analyzer")
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(1920f64, 1080f64));
    let display =
        glium::Display::new(builder, context, &event_loop).expect("Failed to initialize display");

    (event_loop, display)
}

fn imgui_init(display: &glium::Display) -> (imgui_winit_support::WinitPlatform, imgui::Context) {
    let mut imgui_context = imgui::Context::create();
    imgui_context.set_ini_filename(None);

    let mut winit_platform = imgui_winit_support::WinitPlatform::init(&mut imgui_context);

    let gl_window = display.gl_window();
    let window = gl_window.window();

    let dpi_mode = imgui_winit_support::HiDpiMode::Default;

    winit_platform.attach_window(imgui_context.io_mut(), window, dpi_mode);

    imgui_context
        .fonts()
        .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);

    (winit_platform, imgui_context)
}
