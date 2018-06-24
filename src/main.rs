#![feature(nll)]
#![feature(const_vec_new)]

#[macro_use] extern crate maplit;
extern crate toml;
extern crate xdg;

use std::fs;
use std::io;
use std::mem;
use std::os::unix::prelude::AsRawFd;
use std::slice;

mod config;
mod linput;

fn find_g600() -> io::Result<std::path::PathBuf> {
    const KPREFIX: &'static str = "usb-Logitech_Gaming_Mouse_G600_";
    const KSUFFIX: &'static str = "-if01-event-kbd";

    let kdir = std::path::PathBuf::from("/dev/input/by-id");
    let inputbyid: fs::ReadDir = fs::read_dir(kdir.as_path())?;
    for p in inputbyid {
        let path = p?.path();
        let fname = String::from(path.file_name().unwrap().to_str().unwrap());
        if fname.starts_with(KPREFIX) && fname.ends_with(KSUFFIX) {
            return Ok(path);
        }
    }
    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Failed to find g600"))
}

fn run(commands: std::collections::HashMap<u32, String>) -> std::io::Result<()> {
    println!("Starting G600 Linux controller.\n");
    let g600path = find_g600().expect("Error: Couldn't find G600 input device.");
    {
        let f = match fs::File::open(g600path.clone()) {
            Ok(f) => f,
            Err(e) => {
                let msg = format!(
                    "Error: Couldn't open \"{}\" for reading; reason: {}",
                    g600path.into_os_string().to_str().unwrap(), e);
                return Err(std::io::Error::new(std::io::ErrorKind::NotFound, msg))
            },
        };

        unsafe {
            let fd: std::os::raw::c_int = f.as_raw_fd();
            let res = linput::ioctl(fd, linput::_EVIOCGRAB, 1);
            assert_eq!(res, 0);
        }

        {
            use std::io::BufReader;
            use std::io::prelude::*;
            let event_size = mem::size_of::<linput::input_event>();
            let mut fb = BufReader::with_capacity(64 * event_size, f);

            println!("G600 controller started successfully.\n");
            let mut ev: linput::input_event = unsafe { mem::zeroed() };
            let mut ev2: linput::input_event = unsafe { mem::zeroed() };
            loop {
                unsafe {
                    let event_slice = slice::from_raw_parts_mut(
                        &mut ev as *mut _ as *mut u8,
                        event_size
                    );
                    let event_slice2 = slice::from_raw_parts_mut(
                        &mut ev2 as *mut _ as *mut u8,
                        event_size
                    );
                    fb.read_exact(event_slice)
                        .and_then(|_| fb.read_exact(event_slice2))
                }?;
                if ev.type_ != 4 || ev.code != 4 || ev2.type_ != 1 || ev2.value == 0 { continue }
                let scancode = (ev.value & (!0x70000)) as u32;

                //println!("Read {:#?} with scancode {}", ev, scancode);
                let cmd = commands.get(&scancode);
                match cmd {
                    Some(binding) => {
                        println!("Scancode {} is bound to {}", &scancode, binding);
                        use std::process::Command;
                        let mut output = Command::new("bash")
                            .arg("-c")
                            .arg(binding)
                            .spawn()
                            .expect("Failed to execute subprocess");
                        output.wait().expect("Subprocess should exit");
                        println!("Subprocess finished.");
                    },
                    _ => println!("Scancode {} is unbound", &scancode),
                }
            }
        }
    }
}

fn build_default_commands() -> std::collections::HashMap<u32, String> {
    let s = String::from;
    let commands: std::collections::HashMap<u32, &str> = hashmap! {
        // default commands, applied to all layouts
    };
    commands.iter()
        .map(|(&k, &v)| (k, s(v)))
        .collect()
}

fn run_with_dotfile(path: ::std::path::PathBuf) -> () {
    assert!(path.exists());
    let mut commands = build_default_commands();

    match ::config::load_commands_from_dotfile(&path) {
        Some(dotcommands) => {
            for (sc, cmd) in &dotcommands {
                commands.insert(*sc, cmd.clone());
            }
            println!("Loaded {} commands from dotfile.", dotcommands.len());
        },
        None => (),
    }

    run(commands).expect("Expected successful run");
}

fn main() {
    match config::find_dotfile() {
        Some(dot) => {
            println!("Using config file at {}", dot.to_string_lossy());
            run_with_dotfile(dot);
        },
        _ => {
            println!("No configuration found.");
            println!("Create a config.toml in either ~/.config/lg600r or ~/.lg600r");
        },
    }
}

