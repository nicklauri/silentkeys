use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    ops::{Deref, DerefMut},
    time::{Duration, SystemTime},
};

use fnv::FnvBuildHasher;
use rdev::{Event, EventType, Key};

/// How long is too fast? It's sub-10ms, but 10ms to make sure.
pub const PRESSED_TOO_FAST_IN_MS: u128 = 10;

/// If the duration is bigger than 100ms, then it is awhile.
pub const AWHILE: Duration = Duration::from_millis(100);

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum KeyPressState {
    Up,
    Down,
}

#[derive(Debug, Clone, Copy)]
pub struct KeyInfo {
    pub key: Key,
    pub state: KeyPressState,
    pub pressed_at: SystemTime,
    pub just_pressed_after_awhile: bool,
}

impl KeyInfo {
    fn new(key: Key, state: KeyPressState, pressed_at: SystemTime) -> Self {
        Self {
            key,
            state,
            pressed_at,
            just_pressed_after_awhile: false,
        }
    }

    fn is_both_down_state(&self, other: Self) -> bool {
        self.key == other.key && self.state == other.state && self.state == KeyPressState::Down
    }

    fn should_ignore(&self) -> bool {
        use Key::*;
        const INCLUDED_KEYS: &'static [Key] = &[
            ShiftLeft,
            Minus,
            Equal,
            KeyQ,
            KeyW,
            KeyE,
            KeyR,
            KeyT,
            KeyY,
            KeyU,
            KeyI,
            KeyO,
            KeyP,
            LeftBracket,
            RightBracket,
            KeyA,
            KeyS,
            KeyD,
            KeyF,
            KeyG,
            KeyH,
            KeyJ,
            KeyK,
            KeyL,
            SemiColon,
            Quote,
            BackSlash,
            IntlBackslash,
            KeyZ,
            KeyX,
            KeyC,
            KeyV,
            KeyB,
            KeyN,
            KeyM,
            Comma,
            Dot,
            Slash,
        ];

        !INCLUDED_KEYS.contains(&self.key)
    }

    /// This function doesn't care about `after.pressed_at` value.
    /// It simply uses its internal `.pressed_at` to calculate with an assumption
    /// that we're doing this realtime.
    /// TODO: real function to calculate between two SystemTime.  
    fn is_pressed_too_quick(&self, after: Self) -> bool {
        if self.just_pressed_after_awhile {
            return false
        }

        let self_elapsed_in_millis = match self.pressed_at.elapsed() {
            Ok(value) => value.as_millis(),
            Err(err) => {
                println!(
                    "error: could not get elapsed value of system time: {:?}",
                    self.pressed_at
                );
                return false
            }
        };

        // Actually, we don't have to
        self.state == KeyPressState::Down
            && after.state == KeyPressState::Up
            && self_elapsed_in_millis <= PRESSED_TOO_FAST_IN_MS
    }

    fn update_after_awhile(&mut self, before: Self) {
        if before.state == KeyPressState::Up
            && self.state == KeyPressState::Down
            && before
                .pressed_at
                .elapsed()
                .map(|dur| dur >= AWHILE)
                .unwrap_or(true)
        {
            self.set_after_awhile();
        }
    }

    fn set_after_awhile(&mut self) {
        self.just_pressed_after_awhile = true;
    }
}

const MAXIMUM_KEY_HISTORY: usize = 6;

type KeyPressedMap = HashMap<Key, KeyInfo, FnvBuildHasher>;

thread_local! {
    static KEY_PRESSED_MAP: RefCell<KeyPressedMap> = RefCell::new(HashMap::default());
}

pub fn should_send_backspace(event: &Event) -> Option<KeyInfo> {
    // Get key and key state out of event type.
    let (key, state) = match event.event_type {
        EventType::KeyPress(key) => (key, KeyPressState::Down),
        EventType::KeyRelease(key) => (key, KeyPressState::Up),
        _ => return None, // Ignore everything that is not one of key events.
    };

    let mut current = KeyInfo::new(key, state, event.time);

    if current.should_ignore() {
        return None
    }

    KEY_PRESSED_MAP.with(|map| {
        let map = &mut *map.borrow_mut();

        // Guaranteed to have the same key.
        let last_key_state = match map.get(&key) {
            None => {
                // If the hasn't been in the map yet, automatically set "after awhile" for it.
                current.set_after_awhile();
                map.insert(key, current);
                return None
            }
            Some(info) => *info,
        };

        // Has the same down state, user is holding the keydown.
        // Do nothing since we don't have to update the value in the map.
        if last_key_state.is_both_down_state(current) {
            return None
        }

        current.update_after_awhile(last_key_state);

        // else: update state in the map.
        map.insert(key, current);

        if last_key_state.is_pressed_too_quick(current) {
            Some(last_key_state)
        }
        else {
            None
        }
    })
}

pub fn clear_map() {
    KEY_PRESSED_MAP.with(|map| {
        map.borrow_mut().clear();
    })
}
