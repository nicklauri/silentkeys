use std::{sync::mpsc::{self, Receiver}};

use tray_item::TrayItem;

#[derive(Debug)]
pub enum SystemTrayMessage {
    Quit,
}

pub fn init_system_tray() -> anyhow::Result<Receiver<SystemTrayMessage>> {
    let mut tray = TrayItem::new("SilentKeys", "silentkeys-icon")?;

    tray.add_label("Silient Keys")?;

    let (tx, rx) = mpsc::channel();

    tray.add_menu_item("Quit", move || {
        println!("info: sending quit message");
        if let Err(err) = tx.send(SystemTrayMessage::Quit) {
            println!("error: could not send quit message, err: {err:?}");
        }
    })?;

    Ok(rx)
}
