#[macro_use]
extern crate log;

mod batman;
mod config;
mod dbus_bindings;
mod notify;

use anyhow::Result;
use batman::Batman;
use dbus::blocking::Connection;

use crate::config::Config;

fn main() -> Result<()> {
    pretty_env_logger::init();
    info!("logging enabled");

    let mut config = Config::get()?;
    config.validate();

    let system = Connection::new_system()?;
    let session = Connection::new_session()?;

    let batman = Batman::new(config, &system, &session)?;
    batman.add_signals()?;

    // Only those who have suffered long, can see the light within the shadows
    batman.watch(&system)
}
