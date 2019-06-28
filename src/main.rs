#![feature(nll)]
#![feature(const_vec_new)]

#[macro_use] extern crate maplit;
extern crate toml;
extern crate xdg;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate libc;

use std::fs;
use std::io;
use std::cell::RefCell;
use std::collections::BTreeMap;
use config::BindingType;

mod config;
mod linput;
mod keyboard_watcher;
mod xdo;

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

fn format_gkey(gkey: u32) -> String {
    match gkey {
        k if k >= 100 => format!("G^{}", &(k - 100)),
        k => format!("G{}", &k),
    }
}

fn run(commands: std::collections::BTreeMap<u32, (u32, BindingType)>, scancodes_by_gkey: std::collections::BTreeMap<u32, u32>) -> Result<(), Box<dyn (::std::error::Error)>> {
    println!("Starting G600 Linux controller.\n");
    let g600path = find_g600().expect("Error: Couldn't find G600 input device.");
    let f = fs::File::open(g600path.clone())
        .map_err(|e| {
            let msg = format!(
                "Error: Couldn't open \"{}\" for reading; reason: {}",
                g600path.into_os_string().to_str().unwrap(), e);
            Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, msg))
        })?;
    let gkeys_by_scancode = scancodes_by_gkey.iter()
        .map(|(x, y)| (*y, *x))
        .collect::<BTreeMap<_, _>>();
    let exit = RefCell::new(false);
    keyboard_watcher::KeyboardWatcher::create(f)
        .map_err(|err| {
            Box::new(std::io::Error::new(::std::io::ErrorKind::PermissionDenied, err)).into()
        })
        .and_then(|mut watcher| {
            println!("G600 controller started successfully.\n");
            use ::xdo::managed as xmanaged;
            let mut xdm = xmanaged::XdoManaged::default();
            watcher.watch(|scancode, pressed| {
                let cmd = commands.get(&scancode);
                match cmd {
                    Some((gkey, binding)) => {
                        println!("{} (Scancode {}) is bound to {:?}", format_gkey(*gkey), &scancode, binding);
                        match (binding, pressed) {
                            (BindingType::Command(cmd), true) => {
                                use std::process::Command;
                                let mut output = Command::new("bash")
                                    .arg("-c")
                                    .arg(cmd)
                                    .spawn()
                                    .expect("Failed to execute subprocess");
                                output.wait().expect("Subprocess should exit");
                                println!("Subprocess finished.");
                            },
                            (BindingType::Command(_), false) => (),
                            (BindingType::EmulateMouse(button), pressed) => {
                                if pressed {
                                    xdm.mouse_down(*button);
                                } else {
                                    xdm.mouse_up(*button);
                                }
                            },
                            _ => {},
                        }
                    },
                    _ => {
                        println!("Scancode {} ({}) is unbound", &scancode, format_gkey(*gkeys_by_scancode.get(&scancode).unwrap()));
                    },
                }
            }, &exit)?;
            Ok(())
        })
}

fn build_default_commands() -> std::collections::HashMap<u32, BindingType> {
    let commands: std::collections::HashMap<u32, BindingType> = hashmap! {
        // default commands, applied to all layouts
    };
    commands
}

fn run_with_dotfile(path: ::std::path::PathBuf) -> Result<(), Box<dyn (::std::error::Error)>> {
    assert!(path.exists());

    ::config::load_configuration_from_dotfile(&path)
        .map(|config::Configuration { bindings: dotcommands, scancodes }| {
            let mut commands = build_default_commands();
            for (sc, cmd) in &dotcommands {
                commands.insert(*sc, cmd.clone());
            }
            println!(
                "Loaded {} commands and {} scancode mappings from dotfile.",
                dotcommands.len(),
                scancodes.len(),
            );
            let scancodes_by_gkey =
                scancodes.iter().cloned().collect::<BTreeMap<_,_>>();
            let commands = commands.iter()
                .map(|(gkey, command)| {
                    if let Some(&scancode) = scancodes_by_gkey.get(&gkey) {
                        (scancode, (*gkey, command.clone()))
                    } else {
                        eprintln!("GKey {} not mapped to scancode; using as scancode", &gkey);
                        (*gkey, (*gkey, command.clone()))
                    }
                })
                .collect::<BTreeMap<u32, (u32, BindingType)>>();
            (commands, scancodes_by_gkey)
        })
        .and_then(|(commands, scancodes_by_gkey)| {
            run(commands, scancodes_by_gkey)
        })
}

fn main() {
    match config::find_dotfile() {
        Some(dot) => {
            println!("Using config file at {}", dot.to_string_lossy());
            if let Err(e) = run_with_dotfile(dot) {
                eprintln!("{}", &e);
            }
        },
        _ => {
            println!("No configuration found.");
            println!("Create a config.toml in either ~/.config/lg600r or ~/.lg600r");
        },
    }
}

