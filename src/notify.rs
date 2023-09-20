use std::{collections::HashMap, time::Duration};

use anyhow::Result;
use dbus::{
    arg::{PropMap, Variant},
    blocking::{Connection, Proxy},
    Error,
};

use crate::dbus_bindings::{notify::OrgFreedesktopNotifications, State, NOTIF_DEST, NOTIF_PATH};

pub struct Notifier<'b>(Proxy<'b, &'b Connection>);

impl<'b> Notifier<'b> {
    pub fn new(dbus_conn: &'b Connection) -> Result<Self> {
        Ok(Self(dbus_conn.with_proxy(
            NOTIF_DEST,
            NOTIF_PATH,
            Duration::from_secs(5),
        )))
    }

    pub fn low(&self) -> Result<()> {
        trace!("notifying low battery");
        self.0.notify(
            "Batman",
            0,
            "battery-low-symbolic",
            "Low battery",
            "Please connect the charger",
            vec![],
            HashMap::default(),
            5000,
        )?;
        Ok(())
    }

    pub fn critical(&self) -> Result<()> {
        trace!("notifying critical battery");
        let mut hints: PropMap = HashMap::default();
        hints.insert("urgency".to_string(), Variant(Box::new(0u8)));
        self.0.notify(
            "Batman",
            0,
            "battery-low-symbolic",
            "Critical battery",
            "The device will suspend in 60 seconds",
            vec![],
            hints,
            0,
        )?;
        Ok(())
    }

    pub fn state(&self, state: State) -> Result<()> {
        trace!("notifying state");
        self.0.notify(
            "Batman",
            0,
            "battery-low-symbolic",
            &format!("Battery is now {}", state),
            "",
            vec![],
            HashMap::new(),
            0,
        )?;
        Ok(())
    }
}
