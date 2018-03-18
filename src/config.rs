use std::collections::BTreeSet;
pub use ::types::Config;
use schedule::{World, Device};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ReconcileResult {
    pub updated_world: bool,
}

pub fn reconcile_config(config: &Config,
                        dhcp_devs: &BTreeSet<Device>,
                        world: &mut World)
                        -> ReconcileResult {
    let mut config_set: BTreeSet<Device> = BTreeSet::new();
    for device in &config.known_devices {
        config_set.insert(device.clone());
    }

    let mut world_set: BTreeSet<Device> = BTreeSet::new();
    for device in &world.closed_devices {
        world_set.insert(device.clone());
    }
    for device in world.schedule.open_device_entries.iter().map(|e| &e.item) {
        world_set.insert(device.clone());
    }

    let config_set = config_set;
    let world_set = world_set;

    world.closed_devices = world.closed_devices
        .clone()
        .into_iter()
        .filter(|d| config_set.contains(d))
        .collect();

    world.schedule.open_device_entries = world.schedule
        .open_device_entries
        .clone()
        .into_iter()
        .filter(|e| config_set.contains(&e.item))
        .collect();

    let new_device_iter = config_set.difference(&world_set);
    for new_device in new_device_iter {
        world.closed_devices.insert(new_device.clone());
    }

    let known_set: BTreeSet<String> = world.closed_devices
        .iter()
        .chain(world.schedule.open_device_entries.iter().map(|e| &e.item))
        .map(|d| d.mac.to_uppercase())
        .collect();
    let unknown_devs = dhcp_devs.iter()
        .filter(|d| !known_set.contains(&d.mac.to_uppercase()))
        .cloned()
        .collect();
    world.unknown_devices = unknown_devs;

    ReconcileResult { updated_world: !(config_set == world_set) }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeSet;
    use schedule::test::world_fixture;
    use config::{reconcile_config, Config, ReconcileResult};
    use schedule::{Device, ScheduleEntry};

    fn unknown_devs_fixture() -> BTreeSet<Device> {
        let mut devs = BTreeSet::new();
        devs.insert(Device {
            name: "TV2".to_owned(),
            mac: "1234".to_owned(),
        });
        // Has different case than in world.
        devs.insert(Device {
            name: "TV3".to_owned(),
            mac: "bBbB".to_owned(),
        });
        devs.insert(Device {
            name: "TV20".to_owned(),
            mac: "2020".to_owned(),
        });
        devs.insert(Device {
            name: "TV21".to_owned(),
            mac: "2121".to_owned(),
        });
        devs
    }

    fn config_fixture() -> Config {
        Config {
            exit_interfaces: BTreeSet::new(),
            dhcp_lease_file: "".to_owned(),
            state_file: "".to_owned(),
            known_devices: [Device {
                                name: "TV1".to_owned(),
                                mac: "5678".to_owned(),
                            },
                            Device {
                                name: "TV2".to_owned(),
                                mac: "1234".to_owned(),
                            },
                            Device {
                                name: "TV3".to_owned(),
                                mac: "bbbb".to_owned(),
                            },
                            Device {
                                name: "TV4".to_owned(),
                                mac: "abcd".to_owned(),
                            }]
                .iter()
                .cloned()
                .collect(),
        }
    }

    #[test]
    fn no_changes() {
        let config = config_fixture();
        let mut world = world_fixture();
        let unknown_devs = BTreeSet::new();

        let expected_world = world.clone();

        let result = reconcile_config(&config, &unknown_devs, &mut world);
        assert_eq!(ReconcileResult { updated_world: false }, result);
        assert_eq!(expected_world, world);
    }

    #[test]
    fn some_changes() {
        let mut config = config_fixture();
        config.known_devices.insert(Device {
            name: "TV5".to_owned(),
            mac: "xyz".to_owned(),
        });

        let mut world = world_fixture();
        world.schedule.open_device_entries.insert(ScheduleEntry {
            item: Device {
                name: "TV6".to_owned(),
                mac: "qrs".to_owned(),
            },
            time_bound: None,
        });
        let tv2_item = world.schedule
            .open_device_entries
            .iter()
            .find(|e| e.item.name == "TV2")
            .unwrap()
            .clone();
        world.schedule.open_device_entries.remove(&tv2_item);
        world.closed_devices.insert(Device {
            name: "TV7".to_owned(),
            mac: "tuv".to_owned(),
        });

        let unknown_devs = unknown_devs_fixture();

        let mut expected_world = world_fixture();
        let tv2_item = expected_world.schedule
            .open_device_entries
            .iter()
            .find(|e| e.item.name == "TV2")
            .unwrap()
            .clone();
        expected_world.schedule.open_device_entries.remove(&tv2_item);
        expected_world.closed_devices.insert(Device {
            name: "TV5".to_owned(),
            mac: "xyz".to_owned(),
        });
        expected_world.closed_devices.insert(Device {
            name: "TV2".to_owned(),
            mac: "1234".to_owned(),
        });
        expected_world.unknown_devices.insert(Device {
            name: "TV20".to_owned(),
            mac: "2020".to_owned(),
        });
        expected_world.unknown_devices.insert(Device {
            name: "TV21".to_owned(),
            mac: "2121".to_owned(),
        });

        let result = reconcile_config(&config, &unknown_devs, &mut world);
        assert_eq!(ReconcileResult { updated_world: true }, result);
        println!("{:#?}\n{:#?}", expected_world, world);
        assert_eq!(expected_world, world);
    }
}
