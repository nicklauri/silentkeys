use std::sync::atomic::{AtomicI32, Ordering};

use atomic_enum::atomic_enum;

use crate::buffer;


#[atomic_enum]
#[derive(PartialEq, Eq)]
pub enum RunMode {
    Disabled,
    Backspace,
}

static RUN_MODE: AtomicRunMode = AtomicRunMode::new(RunMode::Backspace);

pub fn set_run_mode(mode: RunMode) {
    RUN_MODE.store(mode, Ordering::Release);
    
    println!("info: switching to mode: {mode:?}");

    match mode {
        RunMode::Disabled => {
            println!("info: clearing the map...");
            buffer::clear_map();
        }
        RunMode::Backspace => {}
    }
}

pub fn get_run_mode() -> RunMode {
    RUN_MODE.load(Ordering::Acquire)
}
