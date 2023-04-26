/**
 * Based on rdev. 
 */

use std::time::SystemTime;

use rdev::{Key, EventType};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SysEvent {
    pub event_type: EventType,
    pub at: SystemTime,
}

impl SysEvent {
    pub fn to_keyboard_event(&self) -> Option<KeyboardEvent> {
        KeyboardEvent::from_sys_event(*self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyboardEvent {
    pub key: Key,
    pub state: KeyState,
    pub at: SystemTime,
}

impl KeyboardEvent {
    #[inline]
    fn new(key: Key, state: KeyState, at: SystemTime) -> Self {
        Self { key, state, at }
    }

    #[inline]
    pub fn from_sys_event(sys_event: SysEvent) -> Option<Self> {
        Some(match sys_event.event_type {
            EventType::KeyPress(key) => Self::new(key, KeyState::Down, sys_event.at),
            EventType::KeyRelease(key) => Self::new(key, KeyState::Up, sys_event.at),
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyState {
    Up,
    Down,
}
