use std::collections::BTreeSet;

#[derive(Serialize, Deserialize, Debug)]
pub struct Entry {
    pub mac: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Table {
    pub entries: Vec<Entry>,
}

#[derive(
    Debug,
    Clone,
    Eq,
    Ord,
    PartialOrd,
    PartialEq,
    Hash,
    Serialize,
    Deserialize,
)]
pub struct Device {
    pub name: String,
    pub mac: String,
}

#[derive(
    Debug,
    Clone,
    Eq,
    Ord,
    PartialOrd,
    PartialEq,
    Hash,
    Serialize,
    Deserialize,
)]
pub struct ScheduleEntry<T> {
    pub item: T,
    pub time_bound: Option<DateTime<UTC>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum DeviceOverride {
    Open,
    Closed,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum GuestPath {
    Open,
    Closed,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Schedule {
    pub guest_entry: ScheduleEntry<GuestPath>,
    pub override_entry: Option<ScheduleEntry<DeviceOverride>>,
    pub open_device_entries: BTreeSet<ScheduleEntry<Device>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct World {
    pub schedule: Schedule,
    pub closed_devices: BTreeSet<Device>,
    pub unknown_devices: BTreeSet<Device>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub exit_interfaces: BTreeSet<String>,
    pub state_file: String,
    pub dhcp_lease_file: String,
    pub known_devices: BTreeSet<Device>,
}
