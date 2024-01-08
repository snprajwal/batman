use anyhow::{anyhow, Result};
use dbus::{
    arg::prop_cast,
    blocking::{stdintf::org_freedesktop_dbus::PropertiesPropertiesChanged, Proxy, SyncConnection},
    Message,
};
use std::time::Duration;

use crate::{
    config::Config,
    dbus_bindings::{
        device::OrgFreedesktopUPowerDevice,
        upower::{
            OrgFreedesktopUPower, OrgFreedesktopUPowerDeviceAdded,
            OrgFreedesktopUPowerDeviceRemoved,
        },
        State, LINE_POWER_PATH, PRIMARY_BAT_PATH, UPOWER_DEST, UPOWER_PATH,
    },
    notify::Notifier,
};

pub struct Batman {
    system: &'static SyncConnection,
    upower: Proxy<'static, &'static SyncConnection>,
    primary_bat: Proxy<'static, &'static SyncConnection>,
    auxiliary_bats: Vec<Proxy<'static, &'static SyncConnection>>,
    notifier: Notifier<'static>,
    config: Config,
}

impl Batman {
    pub fn new(
        config: Config,
        system: &'static SyncConnection,
        session: &'static SyncConnection,
    ) -> Result<Batman> {
        let upower_proxy = system.with_proxy(UPOWER_DEST, UPOWER_PATH, Duration::from_secs(5));
        trace!("fetching devices");
        let bats = upower_proxy.enumerate_devices()?;
        debug!("batteries: {bats:?}");
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
        for bat in bats
            .into_iter()
            .filter(|b| b.ne(&primary_bat) && b.to_string().ne(LINE_POWER_PATH))
        {
            auxiliary_bat_proxies.push(system.with_proxy(UPOWER_DEST, bat, Duration::from_secs(5)));
        }
        debug!(
            "identified {} auxiliary batteries",
            auxiliary_bat_proxies.len()
        );

        Ok(Self {
            system,
            upower: upower_proxy,
            primary_bat: primary_bat_proxy,
            auxiliary_bats: auxiliary_bat_proxies,
            notifier: Notifier::new(&session)?,
            config,
        })
    }

    pub fn watch(&self, conn: &SyncConnection) -> Result<()> {
        loop {
            conn.process(Duration::from_secs(self.config.refresh))?;
        }
    }

    pub fn watch_connect(&'static mut self) -> Result<()> {
        // Listen to upower for any batteries added
        self.upower.match_signal(
            |d: OrgFreedesktopUPowerDeviceAdded, _: &SyncConnection, _: &Message| {
                debug!("battery connected: {}", d.device);
                let b = self
                    .system
                    .with_proxy(UPOWER_DEST, d.device, Duration::from_secs(5));
                self.notifier
                    .connected(
                        &b.vendor().expect("failed to get vendor"),
                        b.percentage().expect("failed to get percentage"),
                    )
                    .expect("failed to notify battery connection");
                self.auxiliary_bats.push(b);
                true
            },
        )?;
        Ok(())
    }

    pub fn watch_disconnect(&'static mut self) -> Result<()> {
        // Listen to upower for any batteries removed
        self.upower.match_signal(
            |d: OrgFreedesktopUPowerDeviceRemoved, _: &SyncConnection, _: &Message| {
                debug!("battery disconnected: {}", d.device);
                self.auxiliary_bats.swap_remove(
                    self.auxiliary_bats
                        .iter()
                        .enumerate()
                        .find_map(|(i, b)| {
                            b.path.eq(&d.device).then(|| {
                                self.notifier
                                    .disconnected(&b.vendor().expect("failed to get vendor"))
                                    .expect("failed to notify battery disconnection");
                                i
                            })
                        })
                        .expect("battery not present in list of auxiliary batteries"),
                );
                true
            },
        )?;
        Ok(())
    }

    pub fn watch_change(&'static mut self) -> Result<()> {
        // Listen to main battery for changed properties
        self.primary_bat.match_signal(
            |c: PropertiesPropertiesChanged, _: &SyncConnection, _: &Message| {
                trace!("primary battery changed: {:?}", c);
                if let Some(&state) = prop_cast::<u32>(&c.changed_properties, "State") {
                    self.notifier
                        .state(
                            &self.primary_bat.vendor().expect("failed to get vendor"),
                            &state.try_into().unwrap(),
                        )
                        .unwrap();
                }
                if let Some(&charge) = prop_cast::<f64>(&c.changed_properties, "Percentage") {
                    if self.primary_bat.state().expect("failed to get state")
                        == State::Discharging as u32
                    {
                        self.check_charge(charge, true);
                    }
                }
                true
            },
        )?;
        // Listen to auxiliary batteries for changed properties
        for b in self.auxiliary_bats.iter() {
            b.match_signal(
                |c: PropertiesPropertiesChanged, _: &SyncConnection, _: &Message| {
                    trace!("auxiliary battery changed: {:?}", c);
                    if let Some(&state) = prop_cast::<u32>(&c.changed_properties, "State") {
                        self.notifier
                            .state(
                                &b.vendor().expect("failed to get vendor"),
                                &state.try_into().unwrap(),
                            )
                            .unwrap();
                    }
                    if let Some(&charge) = prop_cast::<f64>(&c.changed_properties, "Percentage") {
                        if self.primary_bat.state().expect("failed to get state")
                            == State::Discharging as u32
                        {
                            self.check_charge(charge, false);
                        }
                    }
                    true
                },
            )?;
        }
        Ok(())
    }

    fn check_charge(&self, charge: f64, primary: bool) {
        if charge < self.config.low.into() {
            self.notifier
                .low(&self.primary_bat.vendor().expect("failed to get vendor"))
                .expect("failed to notify low battery");
        } else if charge < self.config.critical.into() {
            self.notifier
                .critical(
                    &self.primary_bat.vendor().expect("failed to get vendor"),
                    primary,
                )
                .expect("failed to notify critical battery");
        }
    }
}
