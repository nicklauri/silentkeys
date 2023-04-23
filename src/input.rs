use std::{thread, time::Duration};

use rdev::{Event, EventType, Key, SimulateError};

use crate::buffer;

/// Delay between send, useful for macOS.
const DELAY_BETWEEN_SEND: u64 = 2;

pub fn handle_key_chattering_events() {
    loop {
        if let Err(err) = rdev::listen(handle_key_event) {
            println!("error: handling event, err: {err:?}");
        }
    }
}

fn handle_key_event(event: Event) {
    if let Some(caught_key) = buffer::should_send_backspace(&event) {
        
        send_backspace();
        match event.event_type {
            EventType::KeyRelease(key) => {
                println!(
                    "info: caught the chatter: {:?} (elapsed: {:?} - awhile: {})",
                    key,
                    caught_key.pressed_at.elapsed().unwrap(),
                    caught_key.just_pressed_after_awhile,
                );
            }
            other => {
                println!("info: unexpected caught chatter: {other:?}");
            }
        }
    }
}

fn send_backspace() {
    send_key(Key::Backspace);
}

fn send_key(key: Key) {
    send(EventType::KeyPress(key));
    send(EventType::KeyRelease(key));
}

fn send(event_type: EventType) {
    match rdev::simulate(&event_type) {
        Ok(()) => (),
        Err(SimulateError) => {
            println!("error: could not send event: {event_type:?}");
        }
    }

    if (DELAY_BETWEEN_SEND > 0) {
        // Let the OS catchup.
        let delay = Duration::from_millis(DELAY_BETWEEN_SEND);
        thread::sleep(delay);
    }
}
