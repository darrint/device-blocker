use std::collections::BTreeSet;
use std::iter::FromIterator;
use chrono::{DateTime, Utc};
use juniper::{GraphQLType};

#[derive(Serialize, Deserialize, Debug, GraphQLObject)]
pub struct Entry {
    pub mac: String,
}

#[derive(Serialize, Deserialize, Debug, GraphQLObject)]
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
    GraphQLObject,
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
pub struct ScheduleEntry<T: GraphQLType> {
    pub item: T,
    pub time_bound: Option<DateTime<Utc>>,
}

graphql_object!(ScheduleEntry<GuestPath>: () as "ScheduleEntryGuestPath" |&self| {
    field item() -> &GuestPath {&self.item},
    field time_bound() -> Option<DateTime<Utc>> {self.time_bound},
});

graphql_object!(ScheduleEntry<DeviceOverride>: () as "ScheduleEntryDeviceOverride" |&self| {
    field item() -> &DeviceOverride {&self.item},
    field time_bound() -> Option<DateTime<Utc>> {self.time_bound},
});

graphql_object!(ScheduleEntry<Device>: () as "ScheduleEntryDevice" |&self| {
    field item() -> &Device {&self.item},
    field time_bound() -> Option<DateTime<Utc>> {self.time_bound},
});

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, GraphQLEnum)]
pub enum DeviceOverride {
    Open,
    Closed,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, GraphQLEnum)]
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

fn set_to_vec<T: ::std::clone::Clone>(input: &BTreeSet<T>) -> Vec<T> {
    Vec::from_iter(input.clone().to_owned())
}

graphql_object!(Schedule: () |&self| {
    field guest_entry() -> &ScheduleEntry<GuestPath> {&self.guest_entry},
    field override_entry() -> &Option<ScheduleEntry<DeviceOverride>> {&self.override_entry},
    field open_device_entries() -> Vec<ScheduleEntry<Device>> {set_to_vec(&self.open_device_entries)},
});

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct World {
    pub schedule: Schedule,
    pub closed_devices: BTreeSet<Device>,
    pub unknown_devices: BTreeSet<Device>,
}

graphql_object!(World: () |&self| {
    field schedule() -> &Schedule {&self.schedule},
    field closed_devices() -> Vec<Device> {set_to_vec(&self.closed_devices)},
    field unknown_devices() -> Vec<Device> {set_to_vec(&self.unknown_devices)},
});


#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub exit_interfaces: BTreeSet<String>,
    pub state_file: String,
    pub dhcp_lease_file: String,
    pub known_devices: BTreeSet<Device>,
}
