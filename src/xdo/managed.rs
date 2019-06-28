#![allow(dead_code)]
// Derived from "Enigo-rs" source under the following license:
//MIT License
//
//Copyright (c) 2017 pythoneer
//
//Permission is hereby granted, free of charge, to any person obtaining a copy
//of this software and associated documentation files (the "Software"), to deal
//in the Software without restriction, including without limitation the rights
//to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//copies of the Software, and to permit persons to whom the Software is
//furnished to do so, subject to the following conditions:
//
//The above copyright notice and this permission notice shall be included in all
//copies or substantial portions of the Software.
//
//THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//SOFTWARE.
use libc;

use super::{Key, KeyboardControllable};

use self::libc::{c_char, c_int, c_void, useconds_t};
use std::{borrow::Cow, ffi::CString, ptr};

const CURRENT_WINDOW: c_int = 0;
const DEFAULT_DELAY: u64 = 12000;
type Window = c_int;
type Xdo = *const c_void;

#[link(name = "xdo")]
extern "C" {
    fn xdo_free(xdo: Xdo);
    fn xdo_new(display: *const c_char) -> Xdo;

    fn xdo_click_window(xdo: Xdo, window: Window, button: c_int) -> c_int;
    fn xdo_mouse_down(xdo: Xdo, window: Window, button: c_int) -> c_int;
    fn xdo_mouse_up(xdo: Xdo, window: Window, button: c_int) -> c_int;
    fn xdo_move_mouse(xdo: Xdo, x: c_int, y: c_int, screen: c_int) -> c_int;
    fn xdo_move_mouse_relative(xdo: Xdo, x: c_int, y: c_int) -> c_int;

    fn xdo_enter_text_window(
        xdo: Xdo,
        window: Window,
        string: *const c_char,
        delay: useconds_t,
    ) -> c_int;
    fn xdo_send_keysequence_window(
        xdo: Xdo,
        window: Window,
        string: *const c_char,
        delay: useconds_t,
    ) -> c_int;
    fn xdo_send_keysequence_window_down(
        xdo: Xdo,
        window: Window,
        string: *const c_char,
        delay: useconds_t,
    ) -> c_int;
    fn xdo_send_keysequence_window_up(
        xdo: Xdo,
        window: Window,
        string: *const c_char,
        delay: useconds_t,
    ) -> c_int;
}

const MOUSEBUTTON_LEFT: c_int = 1;
const MOUSEBUTTON_MIDDLE: c_int = 2;
const MOUSEBUTTON_RIGHT: c_int = 3;
const MOUSEBUTTON_SCROLL_UP: c_int = 4;
const MOUSEBUTTON_SCROLL_DOWN: c_int = 5;
const MOUSEBUTTON_SCROLL_LEFT: c_int = 6;
const MOUSEBUTTON_SCROLL_RIGHT: c_int = 7;
const MOUSEBUTTON_4: c_int = 8;
const MOUSEBUTTON_5: c_int = 9;

pub struct XdoManaged {
    xdo: Xdo,
    delay: u64,
}
// This is safe, we have a unique pointer.
// TODO: use Unique<c_char> once stable.
unsafe impl Send for XdoManaged {}

impl Default for XdoManaged {
    /// Create a new Enigo instance
    fn default() -> Self {
        Self {
            xdo: unsafe { xdo_new(ptr::null()) },
            delay: DEFAULT_DELAY,
        }
    }
}
impl XdoManaged {
    /// Get the delay per keypress.
    /// Default value is 12000.
    /// This is Linux-specific.
    pub fn delay(&self) -> u64 {
        self.delay
    }
    /// Set the delay per keypress.
    /// This is Linux-specific.
    pub fn set_delay(&mut self, delay: u64) {
        self.delay = delay;
    }

    pub fn mouse_move_to(&mut self, x: i32, y: i32) {
        unsafe {
            xdo_move_mouse(self.xdo, x as c_int, y as c_int, 0);
        }
    }
    pub fn mouse_move_relative(&mut self, x: i32, y: i32) {
        unsafe {
            xdo_move_mouse_relative(self.xdo, x as c_int, y as c_int);
        }
    }
    pub fn mouse_down(&mut self, button: u8) {
        unsafe {
            xdo_mouse_down(self.xdo, CURRENT_WINDOW, c_int::from(button));
        }
    }
    pub fn mouse_up(&mut self, button: u8) {
        unsafe {
            xdo_mouse_up(self.xdo, CURRENT_WINDOW, c_int::from(button));
        }
    }
    pub fn mouse_click(&mut self, button: u8) {
        unsafe {
            xdo_click_window(self.xdo, CURRENT_WINDOW, c_int::from(button));
        }
    }
    pub fn mouse_scroll_x(&mut self, length: i32) {
        let button: c_int;
        let mut length = length;

        if length < 0 {
            button = MOUSEBUTTON_SCROLL_LEFT;
        } else {
            button = MOUSEBUTTON_SCROLL_RIGHT;
        }

        if length < 0 {
            length = -length;
        }

        for _ in 0..length {
            self.mouse_click(button as u8);
        }
    }
    pub fn mouse_scroll_y(&mut self, length: i32) {
        let button: c_int;
        let mut length = length;

        if length < 0 {
            button = MOUSEBUTTON_SCROLL_UP;
        } else {
            button = MOUSEBUTTON_SCROLL_DOWN;
        }

        if length < 0 {
            length = -length;
        }

        for _ in 0..length {
            self.mouse_click(button as u8);
        }
    }
}
impl Drop for XdoManaged {
    fn drop(&mut self) {
        unsafe {
            xdo_free(self.xdo);
        }
    }
}

fn keysequence<'a>(key: Key) -> Cow<'a, str> {
    if let Key::Layout(c) = key {
        return Cow::Owned(format!("U{:X}", c as u32));
    }
    #[allow(deprecated)]
        // I mean duh, we still need to support deprecated keys until they're removed
        Cow::Borrowed(match key {
        Key::Alt => "Alt",
        Key::Backspace => "BackSpace",
        Key::CapsLock => "CapsLock",
        Key::Control => "Control",
        Key::Delete => "Delete",
        Key::DownArrow => "Down",
        Key::End => "End",
        Key::Escape => "Escape",
        Key::F1 => "F1",
        Key::F10 => "F10",
        Key::F11 => "F11",
        Key::F12 => "F12",
        Key::F2 => "F2",
        Key::F3 => "F3",
        Key::F4 => "F4",
        Key::F5 => "F5",
        Key::F6 => "F6",
        Key::F7 => "F7",
        Key::F8 => "F8",
        Key::F9 => "F9",
        Key::Home => "Home",
        Key::Layout(_) => unreachable!(),
        Key::LeftArrow => "Left",
        Key::Option => "Option",
        Key::PageDown => "PageDown",
        Key::PageUp => "PageUp",
        Key::Raw(_) => unimplemented!(),
        Key::Return => "Return",
        Key::RightArrow => "Right",
        Key::Shift => "Shift",
        Key::Space => "space",
        Key::Tab => "Tab",
        Key::UpArrow => "Up",

        Key::Meta => "Meta",
    })
}
impl KeyboardControllable for XdoManaged {
    fn key_sequence(&mut self, sequence: &str) {
        let string = CString::new(sequence).unwrap();
        unsafe {
            xdo_enter_text_window(
                self.xdo,
                CURRENT_WINDOW,
                string.as_ptr() as *const c_char,
                self.delay as useconds_t,
            );
        }
    }
    fn key_down(&mut self, key: Key) {
        let string = CString::new(&*keysequence(key)).unwrap();
        unsafe {
            xdo_send_keysequence_window_down(
                self.xdo,
                CURRENT_WINDOW,
                string.as_ptr() as *const c_char,
                self.delay as useconds_t,
            );
        }
    }
    fn key_up(&mut self, key: Key) {
        let string = CString::new(&*keysequence(key)).unwrap();
        unsafe {
            xdo_send_keysequence_window_up(
                self.xdo,
                CURRENT_WINDOW,
                string.as_ptr() as *const c_char,
                self.delay as useconds_t,
            );
        }
    }
    fn key_click(&mut self, key: Key) {
        let string = CString::new(&*keysequence(key)).unwrap();
        unsafe {
            xdo_send_keysequence_window(
                self.xdo,
                CURRENT_WINDOW,
                string.as_ptr() as *const c_char,
                self.delay as useconds_t,
            );
        }
    }
}
