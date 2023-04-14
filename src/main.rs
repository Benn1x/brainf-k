#![deny(warnings)]
use llvm_sys::{
    analysis::LLVMVerifyModule,
    core::{
        LLVMAddFunction, LLVMAppendBasicBlockInContext, LLVMArrayType, LLVMBuildAdd,
        LLVMBuildAlloca, LLVMBuildBr, LLVMBuildCall2, LLVMBuildCondBr, LLVMBuildGEP2,
        LLVMBuildICmp, LLVMBuildLoad2, LLVMBuildRetVoid, LLVMBuildStore, LLVMBuildSub,
        LLVMBuildTrunc, LLVMBuildZExt, LLVMConstInt, LLVMConstPointerNull, LLVMDumpModule,
        LLVMFunctionType, LLVMGetFirstBasicBlock, LLVMGetParam, LLVMInt32TypeInContext,
        LLVMInt64TypeInContext, LLVMInt8TypeInContext, LLVMPointerType, LLVMPointerTypeInContext,
        LLVMPositionBuilderAtEnd, LLVMVoidTypeInContext,
    },
    prelude::*,
    target::{LLVM_InitializeNativeAsmPrinter, LLVM_InitializeNativeTarget},
    target_machine::{
        LLVMCreateTargetMachine, LLVMDisposeTargetMachine, LLVMGetFirstTarget,
        LLVMGetTargetFromTriple, LLVMTargetMachineEmitToFile,
    },
    LLVMValue,
};

use std::io::prelude::*;
use std::io::{stdin, Read};
use std::time::Instant;
use std::{fs::read, ptr::null_mut};
use std::{fs::File, process::Command};

use std::ffi::{CStr, CString};

const VERSION: &str = env!("CARGO_PKG_VERSION");

// TODO: CLI
//

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() <= 2 {
        println!("-b: build Binary Code [].bbf\n-i: run created Binary [].bf\n-l Build it Using LLVM (Not Implemented (yet)) [].bf\n-r: Run the String Code [].bf\n You are using the Version {}", VERSION);
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
            "-v" => println!("{}", VERSION),

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
    let mut llvm = LLVM::new();
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

    pub fn code_gen(&mut self, path: &str) {
        // BASIC BLOCKS END WITH BR/OR RET STMT NEED TO REBUILD BLOCKS; < > Does not work properly
        let chars: Vec<u8> = read_byte(path);
        let mut ptr = 0;
        unsafe {
            let main_fn = self.create_fn("exec");
            let arr = LLVMGetParam(main_fn, 0);
            let m_block = LLVMAppendBasicBlockInContext(self.ctx, main_fn, cstr("").as_ptr());
            let mut for_end = Vec::<LLVMBasicBlockRef>::new();
            let mut jump = Vec::<LLVMBasicBlockRef>::new();
            LLVMPositionBuilderAtEnd(self.builder, m_block);
            let getchar_head = LLVMFunctionType(
                LLVMInt32TypeInContext(self.ctx),
                &mut LLVMVoidTypeInContext(self.ctx),
                0,
                0,
            );
            let putchar_head = LLVMFunctionType(
                LLVMInt32TypeInContext(self.ctx),
                &mut LLVMInt32TypeInContext(self.ctx),
                1,
                0,
            );
            let getchar = LLVMAddFunction(self.module, cstr("getchar").as_ptr(), getchar_head);
            let putchar = LLVMAddFunction(self.module, cstr("putchar").as_ptr(), putchar_head);

            let mut i_ptr = LLVMBuildAlloca(
                self.builder,
                LLVMInt32TypeInContext(self.ctx),
                cstr("ptr").as_ptr(),
            );
            let addend = LLVMConstInt(LLVMInt32TypeInContext(self.ctx), 0 as u64, 0);
            LLVMBuildStore(self.builder, addend, i_ptr);

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
                        // Build Blocks
                        let for_block = LLVMAppendBasicBlockInContext(
                            self.ctx,
                            main_fn,
                            cstr("for-loop").as_ptr(),
                        );
                        LLVMBuildBr(self.builder, for_block);
                        LLVMPositionBuilderAtEnd(self.builder, for_block);
                        let for_end_block = LLVMAppendBasicBlockInContext(
                            self.ctx,
                            main_fn,
                            cstr("for-end-loop").as_ptr(),
                        );
                        jump.push(for_block);

                        //Build Jump
                        let arr_ptr = LLVMBuildLoad2(
                            self.builder,
                            LLVMPointerTypeInContext(self.ctx, 0),
                            arr,
                            cstr("").as_ptr(),
                        );
                        let elem_ptr = LLVMBuildGEP2(
                            self.builder,
                            LLVMPointerType(LLVMInt8TypeInContext(self.ctx), 0),
                            arr_ptr,
                            &mut i_ptr,
                            0,
                            cstr("").as_ptr(),
                        );
                        let elem_value = LLVMBuildLoad2(
                            self.builder,
                            LLVMInt8TypeInContext(self.ctx),
                            elem_ptr,
                            cstr("").as_ptr(),
                        );

                        let if_cond = LLVMBuildICmp(
                            self.builder,
                            llvm_sys::LLVMIntPredicate::LLVMIntEQ,
                            elem_value,
                            LLVMConstInt(LLVMInt8TypeInContext(self.ctx), 0, 0),
                            cstr("if").as_ptr(),
                        );
                        LLVMBuildCondBr(self.builder, if_cond, for_end_block, for_block);

                        // update value
                        let new_value = LLVMBuildSub(
                            self.builder,
                            elem_value,
                            LLVMConstInt(LLVMInt8TypeInContext(self.ctx), 1, 1),
                            cstr("").as_ptr(),
                        );
                        LLVMBuildStore(self.builder, new_value, elem_ptr);
                        for_end.push(for_end_block);
                    }

                    b']' => {
                        LLVMPositionBuilderAtEnd(
                            self.builder,
                            match for_end.pop() {
                                Some(val) => val,
                                None => panic!("Unmatched Paran"),
                            },
                        );
                        jump.pop();
                        match jump.last() {
                            Some(val) => {
                                LLVMBuildBr(self.builder, val.to_owned());
                            }
                            None => (),
                        }
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
                            &mut LLVMConstPointerNull(LLVMArrayType(
                                LLVMVoidTypeInContext(self.ctx),
                                0,
                            )),
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
                        let mut trunc = LLVMBuildLoad2(
                            self.builder,
                            LLVMInt32TypeInContext(self.ctx),
                            i_ptr,
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
                    _ => (),
                }
                ptr += 1;
                if ptr >= chars.len() {
                    let block = LLVMGetFirstBasicBlock(main_fn);
                    LLVMPositionBuilderAtEnd(self.builder, block);
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
                llvm_sys::target_machine::LLVMCodeGenOptLevel::LLVMCodeGenLevelNone,
                llvm_sys::target_machine::LLVMRelocMode::LLVMRelocDefault,
                llvm_sys::target_machine::LLVMCodeModel::LLVMCodeModelDefault,
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
            LLVMDisposeTargetMachine(target_machine);
            Command::new("clang")
                .arg("main.c")
                .arg("exec.o")
                .output()
                .expect("Have You installed `clang' and have the main.c in scoop?");
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
