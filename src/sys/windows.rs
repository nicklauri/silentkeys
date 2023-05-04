use rdev::{Key, SimulateError};
use winbindings::Win32::Foundation::WIN32_ERROR;

use super::event_type::{KeyState, KeyboardEvent, SysEvent};

/// Send two events: `KeyPressed` and `KeyRelease`.
pub fn simulate_pressing_key(key: Key) -> Result<(), SimulateError> {
    win::press_key(key)
}

pub fn send_keyboard_event(key: Key, state: KeyState) -> Result<(), SimulateError> {
    match state {
        KeyState::Down => win::send_keydown_event(key),
        KeyState::Up => win::send_keyup_event(key),
    }
}

pub fn send_keyboard_pressing_sequence(keys: &[Key]) {
    for &key in keys.iter() {
        win::press_key(key);
    }
}

type KeyboardEventHookFn = fn(KeyboardEvent);

pub fn keyboard_event_listener(hookfn: KeyboardEventHookFn) -> Result<(), WIN32_ERROR> {
    let result = win::setup_keyboard_listener(hookfn);

    if result.is_ok() {
        ctrlc::set_handler(move || {
            win::remove_keyboard_listener();

            println!("info: removed keyboard hook and exit.");
            std::process::exit(0);
        });
    }

    win::wait_for_messages();

    result
}

mod win {
    use std::{
        cell::Cell,
        mem, ptr,
        sync::atomic::AtomicPtr,
        thread,
        time::{Duration, SystemTime},
    };

    use rdev::{Button, EventType, Key, Keyboard, SimulateError};
    use winbindings::Win32::{
        Foundation::{GetLastError, HMODULE, HWND, LPARAM, LRESULT, WIN32_ERROR, WPARAM},
        UI::{
            Input::KeyboardAndMouse::{
                SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
                KEYEVENTF_KEYUP, VIRTUAL_KEY,
            },
            WindowsAndMessaging::{
                CallNextHookEx, GetMessageA, SetWindowsHookA, SetWindowsHookExA,
                UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT, WHEEL_DELTA, WH_KEYBOARD_LL,
                WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP,
                WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SYSKEYDOWN, WM_SYSKEYUP, WM_XBUTTONDOWN,
            },
        },
    };

    use crate::sys::event_type::{KeyState, KeyboardEvent};

    use super::KeyboardEventHookFn;

    pub fn wait_for_messages() {
        unsafe {
            GetMessageA(ptr::null_mut(), HWND(0), 0, 0);
        }
    }

    // SAFETY: super unsafe!
    static mut HOOK_ID: Cell<HHOOK> = Cell::new(HHOOK(0));

    pub fn setup_keyboard_listener(hookfn: KeyboardEventHookFn) -> Result<(), WIN32_ERROR> {
        unsafe {
            let hook = SetWindowsHookExA(
                WH_KEYBOARD_LL,
                Some(raw_keyboard_inspector_hook),
                HMODULE(0),
                0,
            );

            if hook.as_ref().map(|h| h.is_invalid()).unwrap_or(false) {
                let error = GetLastError();
                return Err(error);
            }

            let hook_id = HOOK_ID.get();

            if !hook_id.is_invalid() {
                UnhookWindowsHookEx(hook_id);
            }

            HOOK_ID.set(hook.unwrap());

            KEYBOARD_INSPECTOR_HOOK = hookfn;

            Ok(())
        }
    }

    pub fn remove_keyboard_listener() -> bool {
        unsafe {
            let hook_id = HOOK_ID.get();

            HOOK_ID.set(HHOOK(0));

            UnhookWindowsHookEx(HOOK_ID.get()).as_bool()
        }
    }

    /// UNSAFE: this is unsafe af, make sure this is not use in multithread context!
    static mut KEYBOARD_INSPECTOR_HOOK: KeyboardEventHookFn = drop;
    // static mut KEYBOARD_INTERCEPTER_HOOK: fn(KeyboardEvent) -> Option<KeyEvent> = drop;

    unsafe extern "system" fn raw_keyboard_inspector_hook(
        code: i32,
        param: WPARAM,
        lpdata: LPARAM,
    ) -> LRESULT {
        const HC_ACTION: i32 = 0; // IDK what this is.

        if code == HC_ACTION {
            if let Some(event) = convert(param, lpdata) {
                KEYBOARD_INSPECTOR_HOOK(event);
            }
        }

        CallNextHookEx(HHOOK(0), code, param, lpdata)
    }

    // unsafe extern "system" fn raw_keyboard_intercepter_hook(
    //     code: i32,
    //     param: WPARAM,
    //     lpdata: LPARAM,
    // ) -> LRESULT {
    //     const HC_ACTION: i32 = 0; // IDK what this is.

    //     if code == HC_ACTION {
    //         if let Some(event) = convert(param, lpdata) {
    //             KEYBOARD_INTERCEPTER_HOOK(event);
    //         }
    //     }

    //     CallNextHookEx(HHOOK(0), code, param, lpdata)
    // }

    unsafe fn convert(param: WPARAM, lpdata: LPARAM) -> Option<KeyboardEvent> {
        match param.0 as u32 {
            WM_KEYDOWN | WM_SYSKEYDOWN => {
                let code = get_code(lpdata);
                let key = key_from_code(code);
                Some(KeyboardEvent {
                    key,
                    state: KeyState::Down,
                    at: SystemTime::now(),
                })
            }
            WM_KEYUP | WM_SYSKEYUP => {
                let code = get_code(lpdata);
                let key = key_from_code(code);
                Some(KeyboardEvent {
                    key,
                    state: KeyState::Up,
                    at: SystemTime::now(),
                })
            }
            // WM_LBUTTONDOWN => Some(EventType::ButtonPress(Button::Left)),
            // WM_LBUTTONUP => Some(EventType::ButtonRelease(Button::Left)),
            // WM_MBUTTONDOWN => Some(EventType::ButtonPress(Button::Middle)),
            // WM_MBUTTONUP => Some(EventType::ButtonRelease(Button::Middle)),
            // WM_RBUTTONDOWN => Some(EventType::ButtonPress(Button::Right)),
            // WM_RBUTTONUP => Some(EventType::ButtonRelease(Button::Right)),
            // WM_XBUTTONDOWN => {
            //     let code = get_button_code(lpdata) as u8;
            //     Some(EventType::ButtonPress(Button::Unknown(code)))
            // }
            // WM_XBUTTONUP => {
            //     let code = get_button_code(lpdata) as u8;
            //     Some(EventType::ButtonRelease(Button::Unknown(code)))
            // }
            // WM_MOUSEMOVE => {
            //     let (x, y) = get_point(lpdata);
            //     Some(EventType::MouseMove {
            //         x: x as f64,
            //         y: y as f64,
            //     })
            // }
            // WM_MOUSEWHEEL => {
            //     let delta = get_delta(lpdata) as c_short;
            //     Some(EventType::Wheel {
            //         delta_x: 0,
            //         delta_y: (delta / WHEEL_DELTA) as i64,
            //     })
            // }
            // WM_MOUSEHWHEEL => {
            //     let delta = get_delta(lpdata) as c_short;
            //     Some(EventType::Wheel {
            //         delta_x: (delta / WHEEL_DELTA) as i64,
            //         delta_y: 0,
            //     })
            // }
            _ => None,
        }
    }

    unsafe fn get_code(lpdata: LPARAM) -> VIRTUAL_KEY {
        let kb = *(lpdata.0 as *const KBDLLHOOKSTRUCT);
        VIRTUAL_KEY(kb.vkCode as u16)
    }

    const KEYEVENTF_KEYDOWN: KEYBD_EVENT_FLAGS = KEYBD_EVENT_FLAGS(0);

    pub fn press_key(key: Key) -> Result<(), SimulateError> {
        press_key_for_duration(key, Duration::ZERO)
    }

    pub fn press_key_for_duration(key: Key, duration: Duration) -> Result<(), SimulateError> {
        send_keydown_event(key)?;

        if (!duration.is_zero()) {
            thread::sleep(duration);
        }

        send_keyup_event(key)
    }

    pub fn send_keydown_event(key: Key) -> Result<(), SimulateError> {
        let Some(key_code) = code_from_key(key) else {
            println!("error: send_keydown_event: could not parse key {key:?} to virtual key code.");
            return Err(SimulateError)
        };

        unsafe { _sim_kb_event(KEYEVENTF_KEYDOWN, key_code) }
    }

    pub fn send_keyup_event(key: Key) -> Result<(), SimulateError> {
        let Some(key_code) = code_from_key(key) else {
            println!("error: send_keyup_event: could not parse key {key:?} to virtual key code.");
            return Err(SimulateError)
        };

        unsafe { _sim_kb_event(KEYEVENTF_KEYUP, key_code) }
    }

    unsafe fn _sim_kb_event(
        flags: KEYBD_EVENT_FLAGS,
        vk: VIRTUAL_KEY,
    ) -> Result<(), SimulateError> {
        let input_union = KEYBDINPUT {
            wVk: vk,
            wScan: 0,
            dwFlags: flags,
            time: 0,
            dwExtraInfo: 0,
        };

        let input: INPUT = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 { ki: input_union },
        };

        check_simulate_error(SendInput(&[input], mem::size_of::<INPUT>() as i32))
    }

    // TODO: use smallvec to send many keyboard events at once.

    fn check_simulate_error(return_code: u32) -> Result<(), SimulateError> {
        if return_code == 1 {
            Ok(())
        } else {
            Err(SimulateError)
        }
    }

    // Copied from rdev! <3
    macro_rules! decl_keycodes {
        ($($key:ident, $code:literal),*) => {
            //TODO: make const when rust lang issue #49146 is fixed
            pub fn code_from_key(key: Key) -> Option<VIRTUAL_KEY> {
                match key {
                    $(
                        Key::$key => Some(VIRTUAL_KEY($code)),
                    )*
                    Key::Unknown(code) => Some(VIRTUAL_KEY(code.try_into().ok()?)),
                    _ => None,
                }
            }

            //TODO: make const when rust lang issue #49146 is fixed
            pub fn key_from_code(code: VIRTUAL_KEY) -> Key {
                match code.0 {
                    $(
                        $code => Key::$key,
                    )*
                    code => Key::Unknown(code.into())
                }
            }
        };
    }

    // https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
    // We redefined here for Letter and number keys which are not in winapi crate (and don't have a name either in win32)
    decl_keycodes! {
        Alt, 164,
        AltGr, 165,
        Backspace, 0x08,
        CapsLock, 20,
        ControlLeft, 162,
        ControlRight, 163,
        Delete, 46,
        DownArrow, 40,
        End, 35,
        Escape, 27,
        F1, 112,
        F10, 121,
        F11, 122,
        F12, 123,
        F2, 113,
        F3, 114,
        F4, 115,
        F5, 116,
        F6, 117,
        F7, 118,
        F8, 119,
        F9, 120,
        Home, 36,
        LeftArrow, 37,
        MetaLeft, 91,
        PageDown, 34,
        PageUp, 33,
        Return, 0x0D,
        RightArrow, 39,
        ShiftLeft, 160,
        ShiftRight, 161,
        Space, 32,
        Tab, 0x09,
        UpArrow, 38,
        PrintScreen, 44,
        ScrollLock, 145,
        Pause, 19,
        NumLock, 144,
        BackQuote, 192,
        Num1, 49,
        Num2, 50,
        Num3, 51,
        Num4, 52,
        Num5, 53,
        Num6, 54,
        Num7, 55,
        Num8, 56,
        Num9, 57,
        Num0, 48,
        Minus, 189,
        Equal, 187,
        KeyQ, 81,
        KeyW, 87,
        KeyE, 69,
        KeyR, 82,
        KeyT, 84,
        KeyY, 89,
        KeyU, 85,
        KeyI, 73,
        KeyO, 79,
        KeyP, 80,
        LeftBracket, 219,
        RightBracket, 221,
        KeyA, 65,
        KeyS, 83,
        KeyD, 68,
        KeyF, 70,
        KeyG, 71,
        KeyH, 72,
        KeyJ, 74,
        KeyK, 75,
        KeyL, 76,
        SemiColon, 186,
        Quote, 222,
        BackSlash, 220,
        IntlBackslash, 226,
        KeyZ, 90,
        KeyX, 88,
        KeyC, 67,
        KeyV, 86,
        KeyB, 66,
        KeyN, 78,
        KeyM, 77,
        Comma, 188,
        Dot, 190,
        Slash, 191,
        Insert, 45,
        //KP_RETURN, 13,
        KpMinus, 109,
        KpPlus, 107,
        KpMultiply, 106,
        KpDivide, 111,
        Kp0, 96,
        Kp1, 97,
        Kp2, 98,
        Kp3, 99,
        Kp4, 100,
        Kp5, 101,
        Kp6, 102,
        Kp7, 103,
        Kp8, 104,
        Kp9, 105,
        KpDelete, 110
    }
}
