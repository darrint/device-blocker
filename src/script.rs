use schedule::{
    World,
    GuestPath,
    DeviceOverride,
    ScheduleEntry,
};

enum Action {
    Accept,
    Drop,
}

fn action_with_override (
    override_entry: &Option<ScheduleEntry<DeviceOverride>>,
    device_action: Action) -> Action {
    let device_override = match override_entry {
        &None => None,
        &Some(ref entry) => Some(&entry.item),
    };
    match device_override {
        None => device_action,
        Some(&DeviceOverride::Open) => Action::Accept,
        Some(&DeviceOverride::Closed) => Action::Drop,
    }
}

impl Action {
    fn script(self) -> &'static str {
        match self {
            Action::Accept => "ACCEPT",
            Action::Drop => "DROP",
        }
    }
}

pub fn write_script(world : &World, old_chain: Option<&String>, new_chain: &String, dest: &mut String) {
    dest.push_str(&format!("iptables -N {}\n", new_chain));

    let sch = &world.schedule;
    let device_override = &sch.override_entry;
    for entry in &sch.open_device_entries {
        let action = action_with_override(
            device_override, Action::Accept)
            .script();
        dest.push_str(&format!("iptables -A {} -m mac --mac-source {} -j {}\n", new_chain, entry.item.mac, action));
    }

    for dev in &world.closed_devices {
        let action = action_with_override(
            device_override, Action::Drop)
            .script();
        dest.push_str(&format!("iptables -A {} -m mac --mac-source {} -j {}\n", new_chain, dev.mac, action));
    }

    if world.schedule.guest_entry.item == GuestPath::Closed {
        dest.push_str(&format!("iptables -A {} -j DROP\n", new_chain));
    }

    dest.push_str(&format!("iptables -I FORWARD 0 -j {}\n", new_chain));
    if let Some(old) = old_chain {
        dest.push_str(&format!("iptables -D FORWARD -j {}\n", old));
    }

}
