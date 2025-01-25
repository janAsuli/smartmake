use std::{
    env::current_dir, fs::read_dir, io::Result, path::Path, process::Command,
    thread::available_parallelism,
};

use clap::Parser;

// A program to build your project with the build system that you use
#[derive(Parser)]
#[command(version, about)]
struct Args {
    // The number of threads
    #[arg(short, long)]
    threads: Option<usize>,
}

#[derive(Debug)]
enum BuildProgram {
    Make,
    Ninja,
    Cargo,
    CMake,
}

impl BuildProgram {
    fn from_filename(s: &str) -> Option<BuildProgram> {
        match s {
            "makefile" | "Makefile" | "GNUmakefile" => Some(BuildProgram::Make),
            "build.ninja" => Some(BuildProgram::Ninja),
            "Cargo.toml" => Some(BuildProgram::Cargo),
            "CMakeLists.txt" => Some(BuildProgram::CMake),
            _ => None,
        }
    }

    fn build_make_command<P: AsRef<Path>>(threads: usize, directory: Option<P>) -> Command {
        let mut command = Command::new("make");
        command.arg("-j").arg(threads.to_string());
        if let Some(dir) = directory {
            command.arg("-C").arg(dir.as_ref().as_os_str());
        }
        command
    }

    fn build_ninja_command<P: AsRef<Path>>(threads: usize, directory: Option<P>) -> Command {
        let mut command = Command::new("ninja");
        command.arg("-j").arg(threads.to_string());
        if let Some(dir) = directory {
            command.arg("-C").arg(dir.as_ref().as_os_str());
        }
        command
    }

    fn build_cmake_command<P: AsRef<Path>>(threads: usize, directory: Option<P>) -> Command {
        let mut command = Command::new("cmake");
        command.arg("-C");
        if let Some(dir) = directory {
            command.arg(dir.as_ref().as_os_str());
        } else {
            command.arg(".");
        }
        command.arg("-j").arg(threads.to_string());
        command
    }

    fn run<P: AsRef<Path>>(self, threads: usize, directory: Option<P>) {
        let command = match self {
            BuildProgram::Make => BuildProgram::build_make_command(threads, directory),
            BuildProgram::Ninja => BuildProgram::build_ninja_command(threads, directory),
            BuildProgram::CMake => BuildProgram::build_cmake_command(threads, directory),
            BuildProgram::Cargo => Command::new("cargo build"),
        };
        println!("Command: {:?}", command);
    }
}

fn get_build_system<P: AsRef<Path>>(path: P) -> Result<Option<BuildProgram>> {
    for entry in read_dir(path)? {
        if let Some(name) = entry?.file_name().to_str() {
            let build_program = BuildProgram::from_filename(name);
            if build_program.is_some() {
                return Ok(build_program);
            }
        }
    }
    Ok(None)
}

fn main() {
    let args = Args::parse();

    let threads = args
        .threads
        .unwrap_or(available_parallelism().unwrap().get());

    let cwd = current_dir().unwrap();
    let build_system_maybe = get_build_system(cwd).unwrap();
    if let Some(build_system) = build_system_maybe {
        build_system.run::<&str>(threads, None);
    } else {
        println!("No build system found");
    }
}
