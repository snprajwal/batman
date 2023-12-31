use std::fmt::Display;

use anyhow::{anyhow, Error};

pub mod device;
pub mod notify;
pub mod systemd;
pub mod upower;

// dbus destinations and paths
pub const UPOWER_DEST: &str = "org.freedesktop.UPower";
pub const UPOWER_PATH: &str = "/org/freedesktop/UPower";
pub const LOGIND_DEST: &str = "org.freedesktop.login1";
pub const LOGIND_PATH: &str = "/org/freedesktop/login1/session/1";
pub const NOTIF_DEST: &str = "org.freedesktop.Notifications";
pub const NOTIF_PATH: &str = "/org/freedesktop/Notifications";
// UPower paths
pub const LINE_POWER_PATH: &str = "/org/freedesktop/UPower/devices/line_power_AC";
pub const PRIMARY_BAT_PATH: &str = "/org/freedesktop/UPower/devices/battery_BAT0";

#[derive(Debug)]
pub enum State {
    Unknown,
    Charging,
    Discharging,
    Empty,
    FullyCharged,
    PendingCharge,
    PendingDischarge,
}

impl TryFrom<u32> for State {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => State::Unknown,
            1 => State::Charging,
            2 => State::Discharging,
            3 => State::Empty,
            4 => State::FullyCharged,
            5 => State::PendingCharge,
            6 => State::PendingDischarge,
            _ => return Err(anyhow!("invalid battery state {value}")),
        })
    }
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            State::Unknown => "unknown",
            State::Charging => "charging",
            State::Discharging => "discharging",
            State::Empty => "empty",
            State::FullyCharged => "fully charged",
            State::PendingCharge => "pending charge",
            State::PendingDischarge => "pending discharge",
        })
    }
}
