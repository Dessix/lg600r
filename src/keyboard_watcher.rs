use std::os::unix::prelude::AsRawFd;
use std::fs::File;
use std::{mem, slice};
use linput;
use std::cell::RefCell;

pub struct KeyboardWatcher {
    stream_handle: File,
}

impl KeyboardWatcher {
    pub fn create(f: File) -> Result<KeyboardWatcher, String> {
        {
            let res;
            unsafe {
                let fd: std::os::raw::c_int = f.as_raw_fd();
                res = linput::ioctl(fd, linput::_EVIOCGRAB, 1);
                // debug_assert_eq!(res, 0);
            }
            if res != 0 {
                return Err(format!("Failed to EVIOCGRAB handle to handle; code was {}", res).to_string());
            }
        }

        Ok(KeyboardWatcher {
            stream_handle: f,
        })
    }

    pub fn watch<F: FnMut(u32, bool)->()>(&mut self, mut callback: F, exit: &RefCell<bool>) -> Result<(), Box<dyn (::std::error::Error)>> {
        use std::io::BufReader;
        use std::io::prelude::*;
        let event_size = mem::size_of::<linput::input_event>();
        let mut fb = BufReader::with_capacity(64 * event_size, &mut self.stream_handle);

        loop {
            let mut ev: linput::input_event = unsafe { mem::zeroed() };
            let mut ev2: linput::input_event = unsafe { mem::zeroed() };
            unsafe {
                let event_slice = slice::from_raw_parts_mut(
                    &mut ev as *mut _ as *mut u8,
                    event_size
                );
                fb.read_exact(event_slice)
            }?;
            // Filter to type: EV_MSC, code: MSC_SCAN
            // The following entry after MSC_SCAN is guaranteed the key event in question
            if ev.type_ != 4 || ev.code != 4 { continue }
            unsafe {
                let event_slice2 = slice::from_raw_parts_mut(
                    &mut ev2 as *mut _ as *mut u8,
                    event_size
                );
                fb.read_exact(event_slice2)
            }?;
//            println!("\n{:?}\n  {:?}", &ev, &ev2);
            let pressed = ev2.value != 0;
            let scancode = (ev.value & (!0x70000)) as u32;

            //println!("Read {:#?} with scancode {}", ev, scancode);
            callback(scancode, pressed);
            if *exit.borrow() {
                break;
            }
        }
        Ok(())
    }
}

impl Drop for KeyboardWatcher {
    fn drop(&mut self) {
        unimplemented!()
    }
}
