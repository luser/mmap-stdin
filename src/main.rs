use nix::sys::stat::{self, SFlag};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::path::PathBuf;
use std::ops::Deref;
use std::os::unix::io::{AsRawFd, FromRawFd};

type Error = Box<dyn std::error::Error>;

#[derive(Debug)]
enum Input {
    Stdin,
    File(PathBuf),
}

fn open(input: Input) -> Result<Box<dyn Deref<Target=[u8]>>, Error> {
    match input {
        Input::Stdin => {
            let stdin = io::stdin();
            let fd = stdin.as_raw_fd();
            if SFlag::from_bits_truncate(stat::fstat(fd)?.st_mode).contains(SFlag::S_IFREG) {
                let fd = nix::unistd::dup(fd)?;
                // This is unsafe because it takes ownership of the fd but it's fine
                // because we just dup'ed it above.
                let f = unsafe { File::from_raw_fd(fd) };
                Ok(Box::new(unsafe { memmap::Mmap::map(&f)? }))
            } else {
                // Not a regular file, just read the entire thing.
                let mut buffer = Vec::new();
                let mut l = stdin.lock();
                // read the whole file
                l.read_to_end(&mut buffer)?;
                Ok(Box::new(buffer))
            }
        }
        Input::File(path) => {
            let mut f = File::open(path)?;
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer)?;
            Ok(Box::new(buffer))
        }
    }
}

fn main() -> Result<(), Error> {
    let arg = env::args_os().nth(1);
    let input = match arg {
        None => Input::Stdin,
        Some(a) => Input::File(a.into()),
    };
    let input = open(input)?;
    let bytes = input.deref();
    println!("Input is {} bytes", bytes.len());
    let newlines = memchr::memchr_iter(b'\n', bytes).count();
    println!("Input has {} lines", newlines);
    Ok(())
}
