use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    ops::{Deref, DerefMut},
    time::{Duration, SystemTime},
};

use fnv::{FnvBuildHasher, FnvHasher};
use rdev::{Event, EventType, Key};

use crate::sys::event_type::{KeyState, KeyboardEvent};

/// How long is too fast? It's sub-10ms, but 10ms to make sure.
pub const PRESSED_TOO_FAST_IN_MS: u32 = 10;

/// If the duration is bigger than 100ms, then it is awhile.
pub const AWHILE: Duration = Duration::from_millis(100);
#[derive(Debug, Clone, Copy)]
pub struct KeyInfo {
    pub key: Key,
    pub state: KeyState,
    pub pressed_at: SystemTime,
    pub just_pressed_after_awhile: bool,
}

impl KeyInfo {
    fn new(key: Key, state: KeyState, pressed_at: SystemTime) -> Self {
        Self {
            key,
            state,
            pressed_at,
            just_pressed_after_awhile: false,
        }
    }

    fn from_keyboard_event(keyboard_event: KeyboardEvent) -> Self {
        Self {
            key: keyboard_event.key,
            state: keyboard_event.state,
            pressed_at: keyboard_event.at,
            just_pressed_after_awhile: false,
        }
    }

    fn is_both_down_state(&self, other: Self) -> bool {
        self.key == other.key && self.state == other.state && self.state == KeyState::Down
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
            return false;
        }

        self.state == KeyState::Down
            && after.state == KeyState::Up
            && self.elapsed().as_millis() as u32 <= PRESSED_TOO_FAST_IN_MS
    }

    fn update_after_awhile(&mut self, before: Self) {
        if before.state == KeyState::Up
            && self.state == KeyState::Down
            && before.elapsed() >= AWHILE
        {
            self.set_after_awhile();
        }
    }

    #[inline]
    fn set_after_awhile(&mut self) {
        self.just_pressed_after_awhile = true;
    }

    pub fn elapsed(&self) -> Duration {
        self.pressed_at
            .elapsed()
            .unwrap_or_else(|_| Duration::from_millis(u64::MAX))
    }
}

const MAXIMUM_KEY_HISTORY: usize = 6;
const DEFAULT_KEY_PRESSED_MAP_SIZE: usize = 1 * 1024;

type KeyPressedMap = HashMap<Key, KeyInfo, FnvBuildHasher>;

thread_local! {
    static KEY_PRESSED_MAP: RefCell<KeyPressedMap> = RefCell::new({
        let mut map = HashMap::default();
        map.reserve(DEFAULT_KEY_PRESSED_MAP_SIZE);
        map
    });
}

pub fn should_send_backspace(event: &Event) -> Option<(KeyInfo, KeyInfo)> {
    // Get key and key state out of event type.
    let (key, state) = match event.event_type {
        EventType::KeyPress(key) => (key, KeyState::Down),
        EventType::KeyRelease(key) => (key, KeyState::Up),
        _ => return None, // Ignore everything that is not one of key events.
    };

    let mut current = KeyInfo::new(key, state, event.time);

    if current.should_ignore() {
        return None;
    }

    KEY_PRESSED_MAP.with(|map| {
        let map = &mut *map.borrow_mut();

        // Guaranteed to have the same key.
        let last_key_state = match map.get(&key) {
            None => {
                // If the hasn't been in the map yet, automatically set "after awhile" for it.
                current.set_after_awhile();
                map.insert(key, current);
                return None;
            }
            Some(info) => *info,
        };

        // Has the same down state, user is holding the keydown.
        // Do nothing since we don't have to update the value in the map.
        if last_key_state.is_both_down_state(current) {
            return None;
        }

        current.update_after_awhile(last_key_state);

        // else: update state in the map.
        map.insert(key, current);

        if last_key_state.is_pressed_too_quick(current) {
            Some((current, last_key_state))
        } else {
            None
        }
    })
}

pub fn should_send_backspace_homemade(keyboard_event: KeyboardEvent) -> Option<(KeyInfo, KeyInfo)> {
    let key = keyboard_event.key;
    let mut current = KeyInfo::from_keyboard_event(keyboard_event);

    if current.should_ignore() {
        return None;
    }

    KEY_PRESSED_MAP.with(|map| {
        let map = &mut *map.borrow_mut();

        // Guaranteed to have the same key.
        let last_key_state = match map.get(&key) {
            None => {
                // If the hasn't been in the map yet, automatically set "after awhile" for it.
                current.set_after_awhile();
                map.insert(key, current);
                return None;
            }
            Some(info) => *info,
        };

        // Has the same down state, user is holding the keydown.
        // Do nothing since we don't have to update the value in the map.
        if last_key_state.is_both_down_state(current) {
            return None;
        }

        current.update_after_awhile(last_key_state);

        // else: update state in the map.
        map.insert(key, current);

        if last_key_state.is_pressed_too_quick(current) {
            Some((current, last_key_state))
        } else {
            None
        }
    })
}

pub fn clear_map() {
    KEY_PRESSED_MAP.with(|map| {
        map.borrow_mut().clear();
    })
}
