use crate::linput;
use evdev_rs::enums::{BusType, EventCode, EventType, EV_MSC};
use evdev_rs::{Device, InputEvent, ReadFlag, ReadStatus};
use std::cell::RefCell;
use std::fs::File;
use std::os::unix::prelude::AsRawFd;
use std::{mem, slice};

pub struct KeyboardWatcher {
    device: Device,
}

impl KeyboardWatcher {
    pub fn create(f: File) -> Result<KeyboardWatcher, String> {
        let mut d = Device::new().expect("Libevdev must be installed and available");
        d.set_fd(f).expect("Expected to mount device successfully");
        d.grab(evdev_rs::GrabMode::Grab)
            .expect("Failed to EVIOCGRAB device");
        Ok(KeyboardWatcher { device: d })
    }

    fn next_event_matching<C: (Fn(&InputEvent) -> bool), B: (Fn(&InputEvent) -> bool)>(
        device: &mut Device,
        choose_on: C,
        bail_on: B,
    ) -> Result<InputEvent, InputEvent> {
        let read_flags = evdev_rs::ReadFlag::NORMAL | evdev_rs::ReadFlag::BLOCKING;
        loop {
            match device.next_event(read_flags) {
                Ok((_, ev)) if choose_on(&ev) => return Ok(ev),
                Ok((_, bail)) if bail_on(&bail) => return Err(bail),
                Ok((_, _ev)) => (),
                Err(e) => {
                    println!("Error encountered: {}", e);
                }
            };
        }
    }

    pub fn watch<F: FnMut(u32, bool) -> ()>(
        &mut self,
        mut callback: F,
        exit: &RefCell<bool>,
    ) -> Result<(), Box<dyn (::std::error::Error)>> {
        use std::io::prelude::*;
        use std::io::BufReader;

        let mut bailed_scan = None;
        loop {
            let choose_scan_ev = |ev: &InputEvent| ev.is_code(&EventCode::EV_MSC(EV_MSC::MSC_SCAN));
            let choose_key_ev = |ev: &InputEvent| ev.is_type(&EventType::EV_KEY);
            let scan = match bailed_scan {
                Some(bailed) => bailed,
                None => {
                    let scan_or_bail =
                        Self::next_event_matching(&mut self.device, choose_scan_ev, choose_key_ev);
                    match scan_or_bail {
                        Err(bail) => {
                            println!("Key event / Scans detected out of order from:\n{:#?}", bail);
                            println!(" ... Skipping to next scan...");
                            continue;
                        }
                        Ok(scan) => scan,
                    }
                }
            };
            let scancode = (scan.value & (!0x70000));
            //println!("Scan Event encountered: {:#?} - scancode: {:#?}", &scan, scancode);
            let key_or_bail =
                Self::next_event_matching(&mut self.device, choose_key_ev, choose_scan_ev);

            let key = match key_or_bail {
                Err(bail) => {
                    println!("Key event / Scans detected out of order from:\n{:#?}", bail);
                    println!(" ... Saving as next scan...");
                    bailed_scan = Some(bail);
                    continue;
                }
                Ok(scan) => {
                    bailed_scan = None;
                    scan
                }
            };

            let pressed = key.value != 0;
            callback(scancode as u32, pressed);

            if *exit.borrow() {
                break;
            }
        }
        Ok(())
    }
}

impl Drop for KeyboardWatcher {
    fn drop(&mut self) {}
}
