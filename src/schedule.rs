use std::collections::BTreeSet;
use chrono::{DateTime, UTC};
use errors::{Result, ErrorKind};

pub use ::types::{World, Schedule, ScheduleEntry, Device, DeviceOverride, GuestPath};

impl Default for World {
    fn default() -> World {
        World {
            schedule: Schedule {
                guest_entry: ScheduleEntry {
                    item: GuestPath::Closed,
                    time_bound: None,
                },
                override_entry: None,
                open_device_entries: BTreeSet::new(),
            },
            closed_devices: BTreeSet::new(),
            unknown_devices: BTreeSet::new(),
        }
    }
}

impl World {
    pub fn open_device(&mut self, mac: &str, time_bound: Option<DateTime<UTC>>) -> Result<()> {
        let result = self.closed_devices
            .iter()
            .find(|d| d.mac == mac).map(|i| i.clone());
        if let Some(dev) = result {
            self.closed_devices.remove(&dev);
            let entry = ScheduleEntry {
                item: dev.clone(),
                time_bound: time_bound,
            };
            self.schedule.open_device_entries.insert(entry);
            return Ok(());
        } else {
            return Err(ErrorKind::RequestError("mac not found".to_owned()).into());
        }
    }

    pub fn close_device(&mut self, mac: &str) -> Result<()> {
        let result = self.schedule
            .open_device_entries
            .iter()
            .find(|d| d.item.mac == mac).map(|i| i.clone());
        if let Some(entry) = result {
            self.schedule.open_device_entries.remove(&entry);
            let dev = entry.item;
            self.closed_devices.insert(dev);
            return Ok(());
        } else {
            return Err(ErrorKind::RequestError("mac not found".to_owned()).into());
        }
    }
}

#[cfg(test)]
pub mod test {
    use std::collections::BTreeSet;
    use schedule::{Schedule, World, ScheduleEntry, GuestPath, Device, DeviceOverride};

    pub fn world_fixture() -> World {
        return World {
            schedule: Schedule {
                guest_entry: ScheduleEntry {
                    item: GuestPath::Closed,
                    time_bound: None,
                },
                override_entry: Some(ScheduleEntry {
                    item: DeviceOverride::Closed,
                    time_bound: None,
                }),
                open_device_entries: [ScheduleEntry {
                                              item: Device {
                                                  name: "TV2".to_owned(),
                                                  mac: "1234".to_owned(),
                                              },
                                              time_bound: None,
                                          },
                                          ScheduleEntry {
                                              item: Device {
                                                  name: "TV1".to_owned(),
                                                  mac: "5678".to_owned(),
                                              },
                                              time_bound: None,
                                          }].iter().cloned().collect(),
            },
            closed_devices: [Device {
                                     name: "TV4".to_owned(),
                                     mac: "abcd".to_owned(),
                                 },
                                 Device {
                                     name: "TV3".to_owned(),
                                     mac: "bbbb".to_owned(),
                                 }].iter().cloned().collect(),
            unknown_devices: BTreeSet::new(),
        };
    }

    #[test]
    fn open_device() {
        let mut world = world_fixture();
        world.open_device("bbbb", None).unwrap();
        let expected = World {
            schedule: Schedule {
                guest_entry: ScheduleEntry {
                    item: GuestPath::Closed,
                    time_bound: None,
                },
                override_entry: Some(ScheduleEntry {
                    item: DeviceOverride::Closed,
                    time_bound: None,
                }),
                open_device_entries: [ScheduleEntry {
                                              item: Device {
                                                  name: "TV2".to_owned(),
                                                  mac: "1234".to_owned(),
                                              },
                                              time_bound: None,
                                          },
                                          ScheduleEntry {
                                              item: Device {
                                                  name: "TV1".to_owned(),
                                                  mac: "5678".to_owned(),
                                              },
                                              time_bound: None,
                                          },
                                          ScheduleEntry {
                                              item: Device {
                                                  name: "TV3".to_owned(),
                                                  mac: "bbbb".to_owned(),
                                              },
                                              time_bound: None,
                                          }].iter().cloned().collect(),
            },
            closed_devices: [Device {
                                     name: "TV4".to_owned(),
                                     mac: "abcd".to_owned(),
                                 }].iter().cloned().collect(),
            unknown_devices: BTreeSet::new(),
        };
        assert_eq!(expected, world);
    }

    #[test]
    fn close_device() {
        let mut world = world_fixture();
        world.close_device("5678").unwrap();
        let expected = World {
            schedule: Schedule {
                guest_entry: ScheduleEntry {
                    item: GuestPath::Closed,
                    time_bound: None,
                },
                override_entry: Some(ScheduleEntry {
                    item: DeviceOverride::Closed,
                    time_bound: None,
                }),
                open_device_entries: [ScheduleEntry {
                                              item: Device {
                                                  name: "TV2".to_owned(),
                                                  mac: "1234".to_owned(),
                                              },
                                              time_bound: None,
                                          }].iter().cloned().collect(),
            },
            closed_devices: [Device {
                                     name: "TV4".to_owned(),
                                     mac: "abcd".to_owned(),
                                 },
                                 Device {
                                     name: "TV3".to_owned(),
                                     mac: "bbbb".to_owned(),
                                 },
                                 Device {
                                     name: "TV1".to_owned(),
                                     mac: "5678".to_owned(),
                                 }].iter().cloned().collect(),
            unknown_devices: BTreeSet::new(),
        };
        assert_eq!(expected, world);
    }
}
