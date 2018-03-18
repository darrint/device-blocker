use std::collections::BTreeSet;
use chrono::{DateTime, Utc};
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
    pub fn open_device(&mut self, mac: &str, time_bound: Option<DateTime<Utc>>) -> Result<()> {
        let result = self.closed_devices
            .iter()
            .find(|d| d.mac == mac)
            .map(|i| i.clone());
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
            .find(|d| d.item.mac == mac)
            .map(|i| i.clone());
        if let Some(entry) = result {
            self.schedule.open_device_entries.remove(&entry);
            let dev = entry.item;
            self.closed_devices.insert(dev);
            return Ok(());
        } else {
            return Err(ErrorKind::RequestError("mac not found".to_owned()).into());
        }
    }

    pub fn expire_bounded(&mut self, time_bound: DateTime<Utc>) {
        let expired_open: BTreeSet<ScheduleEntry<Device>> = self.schedule
            .open_device_entries
            .iter()
            .filter(|d| d.time_bound.map(|t| t <= time_bound).unwrap_or(false))
            .map(|i| i.clone())
            .collect();
        for expired_entry in expired_open {
            self.schedule.open_device_entries.remove(&expired_entry);
            self.closed_devices.insert(expired_entry.item);
        }
        if self.schedule.guest_entry.time_bound.map(|t| t <= time_bound).unwrap_or(false) {
            self.schedule.guest_entry.item = GuestPath::Closed;
            self.schedule.guest_entry.time_bound = None;
        }
        let mut clear_override = false;
        if let Some(ref inner_override_entry) = self.schedule.override_entry {
            if inner_override_entry.time_bound.map(|t| t <= time_bound).unwrap_or(false) {
                clear_override = true;
            }
        }
        if clear_override {
            self.schedule.override_entry = None;
        }
    }

    pub fn get_soonest_event_time(&self) -> Option<DateTime<Utc>> {
        let mut all_dates : Vec<DateTime<Utc>> = vec!();
        all_dates.extend(self.schedule.guest_entry.time_bound);
        if let Some(ref se) = self.schedule.override_entry {
            all_dates.extend(se.time_bound);
        }
        all_dates.extend(self.schedule.open_device_entries.clone().into_iter().flat_map(|se| se.time_bound));

        all_dates.into_iter().min()
    }
}

#[cfg(test)]
pub mod test {
    use std::collections::BTreeSet;
    use schedule::{Schedule, World, ScheduleEntry, GuestPath, Device, DeviceOverride};
    use chrono::{Utc, TimeZone};

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
                                      }]
                    .iter()
                    .cloned()
                    .collect(),
            },
            closed_devices: [Device {
                                 name: "TV4".to_owned(),
                                 mac: "abcd".to_owned(),
                             },
                             Device {
                                 name: "TV3".to_owned(),
                                 mac: "bbbb".to_owned(),
                             }]
                .iter()
                .cloned()
                .collect(),
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
                                      }]
                    .iter()
                    .cloned()
                    .collect(),
            },
            closed_devices: [Device {
                                 name: "TV4".to_owned(),
                                 mac: "abcd".to_owned(),
                             }]
                .iter()
                .cloned()
                .collect(),
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
                                      }]
                    .iter()
                    .cloned()
                    .collect(),
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
                             }]
                .iter()
                .cloned()
                .collect(),
            unknown_devices: BTreeSet::new(),
        };
        assert_eq!(expected, world);
    }

    #[test]
    fn do_timed_events() {
        let mut world = World {
            schedule: Schedule {
                guest_entry: ScheduleEntry {
                    item: GuestPath::Open,
                    time_bound: Some(Utc.ymd(2017, 2, 1).and_hms(11, 0, 0)),
                },
                override_entry: Some(ScheduleEntry {
                    item: DeviceOverride::Open,
                    time_bound: Some(Utc.ymd(2017, 2, 1).and_hms(11, 0, 0)),
                }),
                open_device_entries: [ScheduleEntry {
                                          item: Device {
                                              name: "TV2".to_owned(),
                                              mac: "1234".to_owned(),
                                          },
                                          time_bound: Some(Utc.ymd(2017, 2, 1)
                                              .and_hms(10, 0, 0)),
                                      },
                                      ScheduleEntry {
                                          item: Device {
                                              name: "TV1".to_owned(),
                                              mac: "5678".to_owned(),
                                          },
                                          time_bound: Some(Utc.ymd(2017, 2, 1)
                                              .and_hms(11, 0, 0)),
                                      }]
                    .iter()
                    .cloned()
                    .collect(),
            },
            closed_devices: [Device {
                                 name: "TV4".to_owned(),
                                 mac: "abcd".to_owned(),
                             },
                             Device {
                                 name: "TV3".to_owned(),
                                 mac: "bbbb".to_owned(),
                             }]
                .iter()
                .cloned()
                .collect(),
            unknown_devices: BTreeSet::new(),
        };
        world.expire_bounded(Utc.ymd(2017, 2, 1).and_hms(10, 30, 0));
        let expected_1 = World {
            schedule: Schedule {
                guest_entry: ScheduleEntry {
                    item: GuestPath::Open,
                    time_bound: Some(Utc.ymd(2017, 2, 1).and_hms(11, 0, 0)),
                },
                override_entry: Some(ScheduleEntry {
                    item: DeviceOverride::Open,
                    time_bound: Some(Utc.ymd(2017, 2, 1).and_hms(11, 0, 0)),
                }),
                open_device_entries: [ScheduleEntry {
                                          item: Device {
                                              name: "TV1".to_owned(),
                                              mac: "5678".to_owned(),
                                          },
                                          time_bound: Some(Utc.ymd(2017, 2, 1)
                                              .and_hms(11, 0, 0)),
                                      }]
                    .iter()
                    .cloned()
                    .collect(),
            },
            closed_devices: [Device {
                                 name: "TV4".to_owned(),
                                 mac: "abcd".to_owned(),
                             },
                             Device {
                                 name: "TV2".to_owned(),
                                 mac: "1234".to_owned(),
                             },
                             Device {
                                 name: "TV3".to_owned(),
                                 mac: "bbbb".to_owned(),
                             }]
                .iter()
                .cloned()
                .collect(),
            unknown_devices: BTreeSet::new(),
        };
        assert_eq!(expected_1, world);

        world.expire_bounded(Utc.ymd(2017, 2, 1).and_hms(11, 30, 0));
        let expected_2 = World {
            schedule: Schedule {
                guest_entry: ScheduleEntry {
                    item: GuestPath::Closed,
                    time_bound: None,
                },
                override_entry: None,
                open_device_entries: BTreeSet::new(),
            },
            closed_devices: [Device {
                                 name: "TV4".to_owned(),
                                 mac: "abcd".to_owned(),
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
                                 name: "TV1".to_owned(),
                                 mac: "5678".to_owned(),
                             }]
                .iter()
                .cloned()
                .collect(),
            unknown_devices: BTreeSet::new(),
        };
        assert_eq!(expected_2, world);
    }

    #[test]
    fn test_get_soonest_event_time() {
        let mut world = world_fixture();
        assert_eq!(None, world.get_soonest_event_time());
        let date_3 = Utc.ymd(2017, 2, 3).and_hms(0, 0, 0);
        world.schedule.guest_entry.time_bound = Some(date_3);
        assert_eq!(Some(date_3), world.get_soonest_event_time());
        let date_2 = Utc.ymd(2017, 2, 2).and_hms(0, 0, 0);
        world.schedule.override_entry.as_mut().unwrap().time_bound = Some(date_2);
        assert_eq!(Some(date_2), world.get_soonest_event_time());
        let date_1 = Utc.ymd(2017, 2, 1).and_hms(0, 0, 0);
        let entry = ScheduleEntry {
            item: Device{mac: "".to_owned(), name: "".to_owned()},
            time_bound: Some(date_1),
        };
        world.schedule.open_device_entries.insert(entry);
        assert_eq!(Some(date_1), world.get_soonest_event_time());
    }
}
