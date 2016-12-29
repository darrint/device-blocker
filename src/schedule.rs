use std::time::SystemTime;
use std::vec::Vec;
use errors::{Result};

#[derive(Debug, Clone, Eq, Ord, PartialOrd, PartialEq)]
pub struct Device {
    pub name: String,
    pub mac: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ScheduleEntry<T> {
    pub item: T,
    pub time_bound: Option<SystemTime>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DeviceOverride {
    Open,
    Closed,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum GuestPath {
    Open,
    Closed,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Schedule {
    pub guest_entry: ScheduleEntry<GuestPath>,
    pub override_entry: Option<ScheduleEntry<DeviceOverride>>,
    pub open_device_entries: Vec<ScheduleEntry<Device>>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct World {
    pub schedule: Schedule,
    pub closed_devices: Vec<Device>,
}

impl World {
    pub fn all_devices_sorted(&self, dest: &mut Vec<Device>) {
        dest.clear();
        dest.extend_from_slice(&self.closed_devices);
        for item in &self.schedule.open_device_entries {
            dest.push(item.item.clone());
        }
        dest.sort();
    }

    pub fn open_device(
            &mut self, mac: &str, time_bound: Option<SystemTime>)
            -> Result<()> {
        let result = self.closed_devices.iter()
            .position(|ref d| d.mac == mac);
        if let Some(index) = result {
            let dev = self.closed_devices.remove(index);
            let entry = ScheduleEntry{
                item: dev,
                time_bound: time_bound,
            };
            self.schedule.open_device_entries.push(entry);
            return Ok(());
        } else {
            return Err("mac not found".into());
        }
    }

    pub fn close_device(&mut self, mac: &str) -> Result<()> {
        let result = self.schedule.open_device_entries.iter()
            .position(|ref d| d.item.mac == mac);
        if let Some(index) = result {
            let entry = self.schedule.open_device_entries.remove(index);
            let dev = entry.item;
            self.closed_devices.push(dev);
            return Ok(());
        } else {
            return Err("mac not found".into());
        }
    }
}

#[cfg(test)]
mod test {
    use schedule::{
        Schedule,
        World,
        ScheduleEntry,
        GuestPath,
        Device,
        DeviceOverride,
    };

    fn world_fixture() -> World {
        return World{
            schedule: Schedule{
                guest_entry: ScheduleEntry{
                    item: GuestPath::Closed,
                    time_bound: None,
                },
                override_entry: Some(ScheduleEntry{
                    item: DeviceOverride::Closed,
                    time_bound: None,
                }),
                open_device_entries: vec!(
                    ScheduleEntry{
                        item: Device{
                            name: String::from("TV2"),
                            mac: String::from("1234"),
                        },
                        time_bound: None,
                    },
                    ScheduleEntry{
                        item: Device{
                            name: String::from("TV1"),
                            mac: String::from("5678"),
                        },
                        time_bound: None,
                    },
                ),
            },
            closed_devices: vec!(
                Device{
                    name: String::from("TV4"),
                    mac: String::from("abcd"),
                },
                Device{
                    name: String::from("TV3"),
                    mac: String::from("bbbb"),
                },
            ),
        };
    }

    #[test]
    fn all_devices_sorted() {
        let world = world_fixture();
        let mut dest = vec!();
        world.all_devices_sorted(&mut dest);
        let expected = vec!(
            Device{
                name: String::from("TV1"),
                mac: String::from("5678"),
            },
            Device{
                name: String::from("TV2"),
                mac: String::from("1234"),
            },
            Device{
                name: String::from("TV3"),
                mac: String::from("bbbb"),
            },
            Device{
                name: String::from("TV4"),
                mac: String::from("abcd"),
            },
        );
        assert_eq!(expected, dest);
    }

    #[test]
    fn open_device() {
        let mut world = world_fixture();
        world.open_device("bbbb", None).unwrap();
        let expected = World{
            schedule: Schedule{
                guest_entry: ScheduleEntry{
                    item: GuestPath::Closed,
                    time_bound: None,
                },
                override_entry: Some(ScheduleEntry{
                    item: DeviceOverride::Closed,
                    time_bound: None,
                }),
                open_device_entries: vec!(
                    ScheduleEntry{
                        item: Device{
                            name: String::from("TV2"),
                            mac: String::from("1234"),
                        },
                        time_bound: None,
                    },
                    ScheduleEntry{
                        item: Device{
                            name: String::from("TV1"),
                            mac: String::from("5678"),
                        },
                        time_bound: None,
                    },
                    ScheduleEntry{
                        item: Device{
                            name: String::from("TV3"),
                            mac: String::from("bbbb"),
                        },
                        time_bound: None,
                    },
                ),
            },
            closed_devices: vec!(
                Device{
                    name: String::from("TV4"),
                    mac: String::from("abcd"),
                },
            ),
        };
        assert_eq!(expected, world);
    }

    #[test]
    fn close_device() {
        let mut world = world_fixture();
        world.close_device("5678").unwrap();
        let expected = World{
            schedule: Schedule{
                guest_entry: ScheduleEntry{
                    item: GuestPath::Closed,
                    time_bound: None,
                },
                override_entry: Some(ScheduleEntry{
                    item: DeviceOverride::Closed,
                    time_bound: None,
                }),
                open_device_entries: vec!(
                    ScheduleEntry{
                        item: Device{
                            name: String::from("TV2"),
                            mac: String::from("1234"),
                        },
                        time_bound: None,
                    },
                ),
            },
            closed_devices: vec!(
                Device{
                    name: String::from("TV4"),
                    mac: String::from("abcd"),
                },
                Device{
                    name: String::from("TV3"),
                    mac: String::from("bbbb"),
                },
                Device{
                    name: String::from("TV1"),
                    mac: String::from("5678"),
                },
            ),
        };
        assert_eq!(expected, world);
    }
}
