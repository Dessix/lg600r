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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, EnumString, ToString)]
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
