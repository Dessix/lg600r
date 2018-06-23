#![feature(nll)]
#![feature(const_vec_new)]

#[macro_use] extern crate maplit;

use std::fs;
use std::io;
use std::mem;
use std::os::unix::prelude::AsRawFd;
use std::slice;

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

    //ioctl_write_int_bad!(ioctlwriteint, EVIOCGRAB, 1);
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
        4 => "xdotool key Page_Up",
        5 => "xdotool key Page_Down",
        6 => "xdotool key ctrl+c",
        7 => "xdotool key ctrl+shift+c",
        8 => "i3-msg workspace next_on_output",
        9 => "i3-msg move workspace next_on_output",
        10 => "xdotool key ctrl+w",
        11 => "pulseaudio-ctl down",
        12 => "pulseaudio-ctl mute",
        13 => "xdotool key ctrl+z",
        14 => "xdotool key End",
        15 => "xdotool key ctrl+End",
        16 => "xdotool key Return",
        17 => "i3-msg fullscreen",
        18 => "xdotool key ctrl+slash t",
        20 => "xdotool key alt+Left",
        21 => "xdotool key alt+Right",
        22 => "xdotool key ctrl+v",
        23 => "xdotool key ctrl+shift+v",
        24 => "i3-msg workspace prev_on_output",
        25 => "i3-msg move workspace prev_on_output",
        26 => "i3-msg kill",
        27 => "pulseaudio-ctl up",
        28 => "pulseaudio-ctl mute",
        29 => "xdotool key ctrl+shift+z ctrl+y",
        30 => "xdotool key Home",
        31 => "xdotool key ctrl+Home",
        32 => "xdotool key Escape",
        33 => "i3-msg fullscreen",
    };
    commands.iter()
        .map(|(&k, &v)| (k, s(v)))
        .collect()
}

fn load_commands_from_dotfile() -> Option<Vec<(u32, String)>> {
    // TODO: Load configuration from dotfile
    None
}

fn main() {

    let mut commands = build_default_commands();

    match load_commands_from_dotfile() {
        Some(dotcommands) => {
            for (sc, cmd) in dotcommands {
                commands.insert(sc, cmd);
            }
        },
        None => (),
    }

    run(commands).expect("Expected successful run");
}

