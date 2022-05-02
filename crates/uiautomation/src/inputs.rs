use std::str::Chars;
use std::thread::sleep;
use std::time::Duration;

use phf::phf_map;
use phf::phf_set;
use windows::Win32::UI::Input::KeyboardAndMouse::*;

use crate::errors::ERR_FORMAT;
use crate::Error;
use crate::Result;

const VIRTUAL_KEYS: phf::Map<&'static str, VIRTUAL_KEY> = phf_map! {
    "CONTROL" => VK_CONTROL, "CTRL" => VK_CONTROL, "LCONTROL" => VK_LCONTROL, "LCTRL" => VK_LCONTROL, "RCONTROL" => VK_RCONTROL, "RCTRL" => VK_RCONTROL,
    "ALT" => VK_MENU, "MENU" => VK_MENU, "LALT" => VK_LMENU, "LMENU" => VK_LMENU, "RALT" => VK_RMENU, "RMENU" => VK_RMENU,
    "SHIFT" => VK_SHIFT, "LSHIFT" => VK_LSHIFT, "RSHIFT" => VK_RSHIFT,
    "WIN" => VK_LWIN, "WINDOWS" => VK_LWIN, "LWIN" => VK_LWIN, "LWINDOWS" => VK_LWIN, "RWIN" => VK_RWIN, "RWINDOWS" => VK_RWIN,
    "LBUTTON" => VK_LBUTTON, "RBUTTON" => VK_RBUTTON, "MBUTTON" => VK_MBUTTON, "XBUTTON1" => VK_XBUTTON1, "XBUTTON2" => VK_XBUTTON2,
    "CANCEL" => VK_CANCEL, "BACK" => VK_BACK, "TAB" => VK_TAB, "RETURN" => VK_RETURN, "PAUSE" => VK_PAUSE, "CAPITAL" => VK_CAPITAL,
    "ESCAPE" => VK_ESCAPE, "ESC" => VK_ESCAPE, "SPACE" => VK_SPACE,
    "PRIOR" => VK_PRIOR, "PAGE_UP" => VK_PRIOR, "NEXT" => VK_NEXT, "PAGE_DOWN" => VK_NEXT, "HOME" => VK_HOME, "END" => VK_END,
    "LEFT" => VK_LEFT, "UP" => VK_UP, "RIGHT" => VK_RIGHT, "DOWN" => VK_DOWN, "PRINT" => VK_PRINT,
    "INSERT" => VK_INSERT, "DELETE" => VK_DELETE,
    "F1" => VK_F1, "F2" => VK_F2, "F3" => VK_F3, "F4" => VK_F4, "F5" => VK_F5, "F6" => VK_F6, "F7" => VK_F7, "F8" => VK_F8, "F9" => VK_F9, "F10" => VK_F10,
    "F11" => VK_F11, "F12" => VK_F12, "F13" => VK_F13, "F14" => VK_F14, "F15" => VK_F15, "F16" => VK_F16, "F17" => VK_F17, "F18" => VK_F18, "F19" => VK_F19,
    "F20" => VK_F20, "F21" => VK_F21, "F22" => VK_F22, "F23" => VK_F23, "F24" => VK_F24,
};

const HOLD_KEYS: phf::Set<&'static str> = phf_set! {
    "CONTROL", "CTRL", "LCONTROL", "LCTRL", "RCONTROL", "RCTRL",
    "ALT", "MENU", "LALT", "LMENU", "RALT", "RMENU",
    "SHIFT", "LSHIFT", "RSHIFT",
    "WIN", "WINDOWS", "LWIN", "LWINDOWS", "RWIN", "RWINDOWS"
};

const KEYEVENTF_KEYDOWN: KEYBD_EVENT_FLAGS = KEYBD_EVENT_FLAGS(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputItem {
    HoldKey(VIRTUAL_KEY),
    VirtualKey(VIRTUAL_KEY),
    Character(char),
}

impl InputItem {
    fn is_holdkey(&self) -> bool {
        match self {
            Self::HoldKey(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Input {
    holdkeys: Vec<VIRTUAL_KEY>,
    items: Vec<InputItem>,
}

impl Input {
    fn new() -> Self {
        Self {
            holdkeys: Vec::new(),
            items: Vec::new(),
        }
    }

    fn has_holdkey(&self) -> bool {
        !self.holdkeys.is_empty()
    }

    fn has_items(&self) -> bool {
        !self.items.is_empty()
    }

    fn is_holdkey_only(&self) -> bool {
        !self.holdkeys.is_empty() && self.items.is_empty()
    }

    fn push(&mut self, item: InputItem) {
        if let InputItem::HoldKey(key) = item {
            if !self.holdkeys.contains(&key) {
                self.holdkeys.push(key);
            }
        } else {
            self.items.push(item);
        }
    }

    fn push_all(&mut self, items: &Vec<InputItem>) {
        for item in items {
            self.push(*item);
        }
    }

    fn create_inputs(&self) -> Result<Vec<INPUT>> {
        let mut inputs: Vec<INPUT> = Vec::new();

        for holdkey in &self.holdkeys {
            let input = Self::create_virtual_key(*holdkey, KEYEVENTF_KEYDOWN);
            inputs.push(input);
        }

        for item in &self.items {
            match item {
                InputItem::VirtualKey(key) => {
                    inputs.push(Self::create_virtual_key(*key, KEYEVENTF_KEYDOWN));
                    inputs.push(Self::create_virtual_key(*key, KEYEVENTF_KEYUP));
                },
                InputItem::Character(ch) => {
                    Self::create_char_key(&mut inputs, *ch, self.has_holdkey())?;
                },
                _ => (),
            }
        }

        for holdkey in (&self.holdkeys).iter().rev() {
            let input = Self::create_virtual_key(*holdkey, KEYEVENTF_KEYUP);
            inputs.push(input);
        }

        Ok(inputs)
    }

    fn create_virtual_key(key: VIRTUAL_KEY, flags: KEYBD_EVENT_FLAGS) -> INPUT {
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: key,
                    wScan: 0,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }

    fn create_char_key(inputs: &mut Vec<INPUT>, ch: char, hold_mode: bool) -> Result<()> {
        todo!()
    }
}

fn parse_input(expression: &str) -> Result<Vec<Input>> {
    let mut inputs: Vec<Input> = Vec::new();

    let mut expr = expression.chars();
    while let Some((items, is_holdkey)) = next_input(&mut expr)? {
        if let Some(prev) = inputs.last_mut() {
            // if !is_holdkey && (prev.is_holdkey_only() || !prev.has_holdkey()) {
            if (is_holdkey && !prev.has_items()) || (!is_holdkey && (!prev.has_holdkey() || prev.is_holdkey_only())) { 
                prev.push_all(&items);
                continue;
            }
        }

        let mut input = Input::new();
        input.push_all(&items);

        inputs.push(input);
    }

    Ok(inputs)
}

fn next_input(expr: &mut Chars<'_>) -> Result<Option<(Vec<InputItem>, bool)>> {
    if let Some(ch) = expr.next() {
        let next = match ch {
            '{' => {
                let item = read_special_item(expr)?;
                Some((vec![item], item.is_holdkey()))
            }
            '(' => {
                let items = read_group_items(expr)?;
                Some((items, false))
            }
            _ => Some((vec![InputItem::Character(ch)], false)),
        };
        Ok(next)
    } else {
        Ok(None)
    }
}

fn read_special_item(expr: &mut Chars<'_>) -> Result<InputItem> {
    let mut token = String::new();
    let mut matched = false;
    while let Some(ch) = expr.next() {
        if ch == '}' && !token.is_empty() {
            matched = true;
            break;
        } else {
            token.push(ch);
        }
    }

    if matched {
        if token == "(" || token == ")" || token == "{" || token == "}" {
            Ok(InputItem::Character(token.chars().nth(0).unwrap()))
        } else {
            let token = token.to_uppercase();
            if let Some(key) = VIRTUAL_KEYS.get(&token) {
                if HOLD_KEYS.contains(&token) {
                    Ok(InputItem::HoldKey(*key))
                } else {
                    Ok(InputItem::VirtualKey(*key))
                }
            } else {
                Err(Error::new(ERR_FORMAT, "Error Input Format"))
            }
        }
    } else {
        Err(Error::new(ERR_FORMAT, "Error Input Format"))
    }
}

fn read_group_items(expr: &mut Chars<'_>) -> Result<Vec<InputItem>> {
    let mut items: Vec<InputItem> = Vec::new();
    let mut matched = false;

    while let Some((next, _)) = next_input(expr)? {
        if next.len() == 1 && next[0] == InputItem::Character(')') {
            matched = true;
            break;
        }

        items.extend(next);
    }

    if matched {
        Ok(items)
    } else {
        Err(Error::new(ERR_FORMAT, "Error Input Format"))
    }
}

/// Simulate typing keys on keyboard.
#[derive(Debug, Default)]
pub struct Keyboard {
    interval: u64
}

impl Keyboard {
    /// Create a keyboard to simulate typing keys.
    pub fn new() -> Self {
        Self {
            interval: 0
        }
    }

    /// Set the interval time between keys.
    /// 
    /// `interval` is the time number of milliseconds.
    pub fn interval(mut self, interval: u64) -> Self {
        self.interval = interval;
        self
    }

    /// Simulate typing `keys` on keyboard.
    /// 
    /// `{}` is used for some special keys. For example: `{ctrl}{alt}{delete}`, `{shift}{home}`.
    /// 
    /// `()` is used for group keys. For example: `{ctrl}(AB)`.
    /// 
    /// `{}()` can be quoted by `{}`. For example: `{{}Hi,{(}rust!{)}{}}` types as `{Hi,(rust)}`.
    pub fn send_keys(&self, keys: &str) -> Result<()> {
        let inputs = parse_input(keys)?;
        for ref input in inputs {
            self.send_input(input)?;
        }

        Ok(())
    }

    fn send_input(&self, input: &Input) -> Result<()> {
        let input_keys = input.create_inputs()?;
        let events = input_keys.as_slice();

        self.wait();
        Ok(())
    }

    fn wait(&self) {
        if self.interval > 0 {
            sleep(Duration::from_millis(self.interval));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::mem;

    use windows::Win32::Foundation::GetLastError;
    use windows::Win32::UI::Input::KeyboardAndMouse::SendInput;
    use windows::Win32::UI::Input::KeyboardAndMouse::INPUT;
    use windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0;
    use windows::Win32::UI::Input::KeyboardAndMouse::INPUT_KEYBOARD;
    use windows::Win32::UI::Input::KeyboardAndMouse::KEYBDINPUT;
    use windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS;
    use windows::Win32::UI::Input::KeyboardAndMouse::KEYEVENTF_KEYUP;
    use windows::Win32::UI::Input::KeyboardAndMouse::VK_CONTROL;
    use windows::Win32::UI::Input::KeyboardAndMouse::VK_D;
    use windows::Win32::UI::Input::KeyboardAndMouse::VK_DELETE;
    use windows::Win32::UI::Input::KeyboardAndMouse::VK_LBUTTON;
    use windows::Win32::UI::Input::KeyboardAndMouse::VK_LWIN;
    use windows::Win32::UI::Input::KeyboardAndMouse::VK_MENU;
    use windows::Win32::UI::Input::KeyboardAndMouse::VK_SHIFT;

    use crate::inputs::parse_input;
    use crate::inputs::Input;
    use crate::inputs::InputItem;
    use crate::inputs::VIRTUAL_KEYS;

    #[test]
    fn test_virtual_keys() {
        let key = VIRTUAL_KEYS.get("LBUTTON");
        assert_eq!(key, Some(&VK_LBUTTON));
    }

    #[test]
    fn show_desktop() {
        let inputs: [INPUT; 4] = [
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_LWIN,
                        wScan: 0,
                        dwFlags: KEYBD_EVENT_FLAGS::default(),
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            },
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_D,
                        wScan: 0,
                        dwFlags: KEYBD_EVENT_FLAGS::default(),
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            },
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_D,
                        wScan: 0,
                        dwFlags: KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            },
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_LWIN,
                        wScan: 0,
                        dwFlags: KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            },
        ];

        let sent = unsafe { SendInput(&inputs, mem::size_of::<INPUT>() as _) };

        if sent == 0 {
            let err = unsafe { GetLastError() };
            println!("error code: {}", err.0);
        }

        assert_eq!(sent as usize, inputs.len());
    }

    #[test]
    fn test_parse_input_1() {
        assert_eq!(
            parse_input("{ctrl}c").unwrap(),
            vec![Input {
                holdkeys: vec![VK_CONTROL],
                items: vec![InputItem::Character('c')]
            }]
        );
    }

    #[test]
    fn test_parse_input_2() {
        assert_eq!(
            parse_input("{ctrl}{alt}{delete}").unwrap(),
            vec![Input {
                holdkeys: vec![VK_CONTROL, VK_MENU],
                items: vec![InputItem::VirtualKey(VK_DELETE)]
            }]
        );
    }

    #[test]
    fn test_parse_input_3() {
        assert_eq!(
            parse_input("{shift}(ab)").unwrap(),
            vec![Input {
                holdkeys: vec![VK_SHIFT],
                items: vec![InputItem::Character('a'), InputItem::Character('b')]
            }]
        );
    }
}
