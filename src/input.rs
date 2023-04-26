use std::{sync::mpsc, thread, time::Duration};

use rdev::{Event, EventType, Key, SimulateError};

use crate::{
    buffer,
    sys::{self, event_type::KeyState},
};

/// Delay between send, useful for macOS.
const DELAY_BETWEEN_SEND: u64 = 2;

pub fn handle_key_chattering_events_in_other_thread() {
    let (tx, rx) = mpsc::channel();

    let handle_thread = thread::spawn(move || loop {
        let Ok(event) = rx.recv() else {
                continue
            };

        handle_key_event(event);
    });

    if let Err(err) = rdev::listen(move |ev| drop(tx.send(ev))) {
        println!("error: handling event, err: {err:?}");
    }

    handle_thread.join().unwrap();
}

pub fn handle_key_chattering_events() {
    if let Err(err) = rdev::listen(handle_key_event) {
        println!("error: handling event, err: {err:?}");
    }
}

pub fn handle_key_homemade() {
    sys::windows::keyboard_event_listener(|ev| {
        let event_type = match ev.state {
            KeyState::Up => EventType::KeyRelease(ev.key),
            KeyState::Down => EventType::KeyPress(ev.key),
        };

        let event = Event {
            event_type,
            time: ev.at,
            name: None,
        };

        handle_key_event(event);
    });
}

fn handle_key_event(event: Event) {
    let Some((current_key, caught_key)) = buffer::should_send_backspace(&event) else {
        return
    };

    let caught_key_elapsed = caught_key.elapsed();

    send_backspace();
    match event.event_type {
        EventType::KeyRelease(key) => {
            println!(
                "info: caught the chatter: {:?} (elapsed: {:?} - awhile: {})",
                key, caught_key_elapsed, caught_key.just_pressed_after_awhile,
            );
        }
        other => {
            println!("info: unexpected caught chatter: {other:?}");
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
