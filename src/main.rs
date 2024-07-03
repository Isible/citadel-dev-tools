use std::{env, fs, io, path::PathBuf, process::{self, Command, Stdio}};

use clap::Parser;

const BUILD_DIR: &str = "build";

const ASM_FILE: &str = "build/asm/out.asm";
const OBJ_FILE: &str = "build/obj/out.o";
const BIN_FILE: &str = "build/bin/out";
const BUILD_DIRS: [&str; 3] = ["asm", "obj", "bin"];

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[clap(help = "Specify a root dir", short, long)]
    root_dir: Option<PathBuf>,
}

fn main() {
    process::exit(execute_asm(Args::parse()));
}

fn execute_asm(args: Args) -> i32 {
    if let Some(rd) = args.root_dir {
        env::set_current_dir(rd).expect("Failed to set rootdir to specified dir")
    }

    if is_nasm_installed() {
        ensure_dirs_exist().expect("Failed to create directories");

        return match fs::metadata(ASM_FILE) {
            Ok(_) => run_asm_file().expect("Failed to execute asm file"),
            Err(_) => panic!("Could not find output asm file at path: {}", ASM_FILE),
        };
    }
    0
}

fn ensure_dirs_exist() -> io::Result<()> {
    if fs::metadata(BUILD_DIR).is_err() {
        fs::create_dir(BUILD_DIR)?;
    }

    for dir in BUILD_DIRS {
        let path = format!("{BUILD_DIR}/{dir}");
        if fs::metadata(&path).is_err() {
            fs::create_dir(path)?;
        }
    }

    Ok(())
}

fn is_nasm_installed() -> bool {
    let mut cmd = Command::new("which");
    cmd.arg("nasm");

    cmd.stdout(Stdio::null());

    match cmd.status() {
        Ok(status) => status.success(),
        Err(err) => panic!("Failed to check if nasm is installed: {}", err),
    }
}

fn run_asm_file() -> io::Result<i32> {
    let mut assembler = Command::new("nasm");
    assembler
        .arg("-f")
        .arg("elf64")
        .arg("-o")
        .arg(OBJ_FILE)
        .arg(ASM_FILE);
    assembler.spawn()?.wait()?;

    let mut linker = Command::new("ld");
    linker.arg("-s").arg("-o").arg(BIN_FILE).arg(OBJ_FILE);
    linker.spawn()?.wait()?;

    let mut executor = Command::new(format!("./{BIN_FILE}"));
    let exit_code = executor.spawn()?.wait()?.code().unwrap();

    Ok(exit_code)
}
