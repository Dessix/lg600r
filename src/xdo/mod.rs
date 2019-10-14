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

pub mod managed;

#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Key {
    /// alt key on Linux and Windows (option key on macOS)
    Alt,
    /// backspace key
    Backspace,
    /// caps lock key
    CapsLock,
    /// control key
    Control,
    /// delete key
    Delete,
    /// down arrow key
    DownArrow,
    /// end key
    End,
    /// escape key (esc)
    Escape,
    /// F1 key
    F1,
    /// F10 key
    F10,
    /// F11 key
    F11,
    /// F12 key
    F12,
    /// F13 key
    F13,
    /// F14 key
    F14,
    /// F15 key
    F15,
    /// F16 key
    F16,
    /// F17 key
    F17,
    /// F18 key
    F18,
    /// F19 key
    F19,
    /// F20 key
    F20,
    /// F21 key
    F21,
    /// F22 key
    F22,
    /// F23 key
    F23,
    /// F24 key
    F24,
    /// F2 key
    F2,
    /// F3 key
    F3,
    /// F4 key
    F4,
    /// F5 key
    F5,
    /// F6 key
    F6,
    /// F7 key
    F7,
    /// F8 key
    F8,
    /// F9 key
    F9,
    /// home key
    Home,
    /// left arrow key
    LeftArrow,
    /// meta key (also known as "windows", "super", and "command")
    Meta,
    #[serde(rename="Super_L", alias="SuperL", alias="Super")]
    SuperL,
    #[serde(rename="Super_R", alias="SuperR")]
    SuperR,
    #[serde(rename="Hyper_L", alias="HyperL", alias="Hyper")]
    HyperL,
    #[serde(rename="Hyper_R", alias="HyperR")]
    HyperR,
    /// option key on macOS (alt key on Linux and Windows)
    Option,
    /// page down key
    PageDown,
    /// page up key
    PageUp,
    /// return key
    Return,
    /// right arrow key
    RightArrow,
    /// shift key
    Shift,
    /// space key
    Space,
    /// tab key (tabulator)
    Tab,
    /// up arrow key
    UpArrow,
    /// keyboard layout dependent key
    Layout(char),
    /// raw keycode eg 0x38
    Raw(u16),
}

mod formatting_impls {
    use std::fmt::{Display, Debug, Formatter, Error};
    use super::Key;

    impl Display for Key {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
            Debug::fmt(self, f)
        }
    }

    impl Debug for Key {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
            serde_json::to_value(self)
                .map_err(|_| std::fmt::Error)
                .and_then(|s| f.write_str(&s.as_str().unwrap()))
        }
    }

    use std::str::FromStr;

    impl FromStr for Key {
        type Err = Box<dyn std::error::Error>;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            serde_json::from_value(serde_json::Value::String(s.to_string()))
                .map_err(|e| e.into())
        }
    }

}

pub trait KeyboardControllable {
    /// Types the string
    ///
    /// Emits keystrokes such that the given string is inputted.
    ///
    /// You can use many unicode here like: ❤️. This works
    /// regadless of the current keyboardlayout.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use enigo::*;
    /// let mut enigo = Enigo::new();
    /// enigo.key_sequence("hello world ❤️");
    /// ```
    fn key_sequence(&mut self, sequence: &str);

    /// presses a given key down
    fn key_down(&mut self, key: Key);

    /// release a given key formally pressed down by
    /// [key_down](trait.KeyboardControllable.html#tymethod.key_down)
    fn key_up(&mut self, key: Key);

    /// Much like the
    /// [key_down](trait.KeyboardControllable.html#tymethod.key_down) and
    /// [key_up](trait.KeyboardControllable.html#tymethod.key_up)
    /// function they're just invoked consecutively
    fn key_click(&mut self, key: Key);
}
