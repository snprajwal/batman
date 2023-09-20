use anyhow::{anyhow, Result};
use dbus::{
    blocking::{
        stdintf::org_freedesktop_dbus::{EmitsChangedSignal, PropertiesPropertiesChanged},
        Connection, Proxy,
    },
    Error, Message,
};
use std::{thread, time::Duration};

use crate::{
    config::Config,
    dbus_bindings::{
        device::OrgFreedesktopUPowerDevice,
        upower::{
            OrgFreedesktopUPower, OrgFreedesktopUPowerDeviceAdded,
            OrgFreedesktopUPowerDeviceRemoved,
        },
        State, PRIMARY_BAT_PATH, UPOWER_DEST, UPOWER_PATH,
    },
    notify::Notifier,
};

pub struct Batman<'b> {
    upower: Proxy<'b, &'b Connection>,
    primary_bat: Proxy<'b, &'b Connection>,
    auxiliary_bats: Vec<Proxy<'b, &'b Connection>>,
    notifier: Notifier<'b>,
    config: Config,
}

impl<'b> Batman<'b> {
    // The number of seconds to wait before taking action on critical battery level
    const CRITICAL_ACTION_DELAY: u64 = 60;

    pub fn new(
        config: Config,
        system: &'b Connection,
        session: &'b Connection,
    ) -> Result<Batman<'b>> {
        let upower_proxy = system.with_proxy(UPOWER_DEST, UPOWER_PATH, Duration::from_secs(5));
        trace!("fetching devices");
        let bats = upower_proxy.enumerate_devices()?;
        trace!("identifying primary battery");
        let primary_bat = bats
            .iter()
            .find(|b| b.to_string().eq(PRIMARY_BAT_PATH))
            .ok_or(anyhow!("failed to find primary battery"))?
            .clone();
        debug!("identified primary battery");
        let primary_bat_proxy =
            system.with_proxy(UPOWER_DEST, primary_bat.clone(), Duration::from_secs(5));
        trace!("identifying auxiliary batteries");
        let mut auxiliary_bat_proxies = vec![];
        for bat in bats.into_iter().filter(|b| b.ne(&primary_bat)) {
            auxiliary_bat_proxies.push(system.with_proxy(UPOWER_DEST, bat, Duration::from_secs(5)));
        }
        debug!(
            "identified {} auxiliary batteries",
            auxiliary_bat_proxies.len()
        );

        Ok(Self {
            upower: upower_proxy,
            primary_bat: primary_bat_proxy,
            auxiliary_bats: auxiliary_bat_proxies,
            notifier: Notifier::new(&session)?,
            config,
        })
    }

    pub fn add_signals(&self) -> Result<()> {
        // Listen to upower for any batteries added or removed
        self.upower.match_signal(
            |d: OrgFreedesktopUPowerDeviceAdded, _: &Connection, _: &Message| {
                println!("Device added: {}", d.device);
                true
            },
        )?;
        self.upower.match_signal(
            |d: OrgFreedesktopUPowerDeviceRemoved, _: &Connection, _: &Message| {
                println!("Device removed: {}", d.device);
                true
            },
        )?;
        // Listen to main battery for changed properties
        self.primary_bat.match_signal(
            |c: PropertiesPropertiesChanged, _: &Connection, _: &Message| {
                println!("Primary battery changed: {:?}", c);
                true
            },
        )?;

        Ok(())
    }

    pub fn watch(&self, conn: &Connection) -> Result<()> {
        loop {
            conn.process(Duration::from_secs(self.config.refresh))?;
        }
    }
}
