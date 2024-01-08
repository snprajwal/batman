use std::{collections::HashMap, time::Duration};

use anyhow::Result;
use dbus::{
    arg::{PropMap, Variant},
    blocking::{Proxy, SyncConnection},
};

use crate::dbus_bindings::{notify::OrgFreedesktopNotifications, State, NOTIF_DEST, NOTIF_PATH};

pub struct Notifier<'b>(Proxy<'b, &'b SyncConnection>);

impl<'b> Notifier<'b> {
    pub fn new(dbus_conn: &'b SyncConnection) -> Result<Self> {
        Ok(Self(dbus_conn.with_proxy(
            NOTIF_DEST,
            NOTIF_PATH,
            Duration::from_secs(5),
        )))
    }

    pub fn low(&self, bat: &str) -> Result<()> {
        trace!("notifying low battery");
        self.0.notify(
            "Batman",
            0,
            "battery-level-10-symbolic",
            &format!("Low battery on {bat}"),
            "Please connect the charger",
            vec![],
            HashMap::default(),
            5000,
        )?;
        Ok(())
    }

    pub fn critical(&self, bat: &str, primary: bool) -> Result<()> {
        trace!("notifying critical battery");
        let mut hints: PropMap = HashMap::default();
        hints.insert("urgency".to_string(), Variant(Box::new(0u8)));
        self.0.notify(
            "Batman",
            0,
            "battery-level-0-symbolic",
            &format!("Critical battery on {bat}"),
            if primary {
                "The device will suspend in 60 seconds"
            } else {
                "Please connect the charger immediately"
            },
            vec![],
            hints,
            0,
        )?;
        Ok(())
    }

    pub fn state(&self, bat: &str, state: &State) -> Result<()> {
        trace!("notifying state");
        self.general(&format!("{bat} is now {state}"))?;
        Ok(())
    }

    pub fn connected(&self, bat: &str, charge: f64) -> Result<()> {
        trace!("notifying battery connected: {bat}");
        self.general(&format!("{bat} connected: {charge}%"))?;
        Ok(())
    }

    pub fn disconnected(&self, bat: &str) -> Result<()> {
        trace!("notifying battery disconnected: {bat}");
        self.general(&format!("{bat} disconnected"))?;
        Ok(())
    }

    fn general(&self, msg: &str) -> Result<()> {
        self.0.notify(
            "Batman",
            0,
            "battery-symbolic",
            msg,
            "",
            vec![],
            HashMap::new(),
            0,
        )?;
        Ok(())
    }
}
