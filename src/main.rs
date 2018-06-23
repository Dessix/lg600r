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

                println!("Read {:#?} with scancode {}", ev, scancode);
                let cmd = commands.get(&scancode);
                match cmd {
                    Some(binding) => {
                        println!("{} is bound to {}", &scancode, binding);
                        use std::process::Command;
                        let mut output = Command::new("bash")
                            .arg("-c")
                            .arg(binding)
                            .spawn()
                            .expect("Failed to execute subprocess");
                        output.wait().expect("Subprocess should exit");
                        println!("Subprocess finished.");
                    },
                    _ => println!("{} is unbound", &scancode),
                }
            }
        }
    }
}

fn main() {
    let commands: std::collections::HashMap<u32, String> = {;
        let s = String::from;
        hashmap!{
            4 => s("xdotool key Page_Up"), // scroll left
            5 => s("xdotool key Page_Down"), // scroll right
            6 => s("xdotool key ctrl+c"), // G8
            7 => s("xdotool key ctrl+shift+c"), // G7
            8 => s("i3-msg workspace next_on_output"), // G9
            9 => s("i3-msg move workspace next_on_output"), // G10
            10 => s("xdotool key ctrl+w"), // G11
            11 => s("pulseaudio-ctl down"), // G12
            12 => s("pulseaudio-ctl mute"), // G13
            13 => s("xdotool key ctrl+z"), // G14
            14 => s("xdotool key End"), // G15
            15 => s("xdotool key ctrl+End"), // G16
            16 => s("xdotool key Return"), // G17
            17 => s("i3-msg fullscreen"), // G18
            18 => s("xdotool key ctrl+slash t"), // G19
            19 => s(""), // G20
            20 => s("xdotool key alt+Left"), // G-shift + scroll left
            21 => s("xdotool key alt+Right"), // G-shift + scroll right
            22 => s("xdotool key ctrl+v"), // G-shift + G8
            23 => s("xdotool key ctrl+shift+v"), // G-shift + G7
            24 => s("i3-msg workspace prev_on_output"), // G-shift + G9
            25 => s("i3-msg move workspace prev_on_output"), // G-shift + G10
            26 => s("i3-msg kill"), // G-shift + G11
            27 => s("pulseaudio-ctl up"), // G-shift + G12
            28 => s("pulseaudio-ctl mute"), // G-shift + G13
            29 => s("xdotool key ctrl+shift+z ctrl+y"), // G-shift + G14
            30 => s("xdotool key Home"), // G-shift + G15
            31 => s("xdotool key ctrl+Home"), // G-shift + G16
            32 => s("xdotool key Escape"), // G-shift + G17
            33 => s("i3-msg fullscreen"), // G-shift + G18
            34 => s(""), // G-shift + G19
            35 => s(""), // G-shift + G20
        }
    };

    // TODO: Load configuration from dotfile?

    run(commands).expect("Expected successful run");
}

