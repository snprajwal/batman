#[macro_use]
extern crate log;

mod batman;
mod config;
mod dbus_bindings;
mod notify;

use std::borrow::BorrowMut;

use anyhow::Result;
use batman::Batman;
use dbus::blocking::SyncConnection;
use once_cell::sync::Lazy;

use crate::config::Config;

static SYSTEM: Lazy<SyncConnection> = Lazy::new(|| SyncConnection::new_system().unwrap());
static SESSION: Lazy<SyncConnection> = Lazy::new(|| SyncConnection::new_session().unwrap());

static mut BATMAN: Lazy<Batman> = Lazy::new(|| {
    let mut config = Config::get().unwrap();
    config.validate();
    Batman::new(config, &SYSTEM, &SESSION).unwrap()
});

fn main() -> Result<()> {
    pretty_env_logger::init();
    info!("logging enabled");

    unsafe {
        BATMAN.borrow_mut().watch_connect()?;
        BATMAN.borrow_mut().watch_disconnect()?;
        BATMAN.borrow_mut().watch_change()?;
        // Only those who have suffered long, can see the light within the shadows
        BATMAN.watch(&SYSTEM)
    }
}
