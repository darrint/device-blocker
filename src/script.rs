use std::collections::BTreeSet;
use schedule::{World, GuestPath, DeviceOverride, ScheduleEntry};

enum Action {
    Accept,
    Drop,
}

fn action_with_override(override_entry: &Option<ScheduleEntry<DeviceOverride>>,
                        device_action: Action)
                        -> Action {
    let device_override = match *override_entry {
        None => None,
        Some(ref entry) => Some(&entry.item),
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

pub fn write_script(world: &World, old_chain: &str, new_chain: &str, exit_interfaces: &BTreeSet<String>, dest: &mut String) {
    dest.push_str(&format!("
set -e
set -x
if iptables -L {old} >/dev/null 2>&1; then
    iptables -D FORWARD -j {old} || true
    iptables -F {old}
    iptables -X {old}
fi
if iptables -L {new} >/dev/null 2>&1; then
    iptables -E {new} {old}
fi
iptables -N {new}
",
                           new = new_chain,
                           old = old_chain));

    for interface in exit_interfaces {
        let action = &format!(
            "iptables -A {new} -i {eth} -j ACCEPT\n",
            new = new_chain, eth = interface);
        dest.push_str(action);
    }

    let sch = &world.schedule;
    let device_override = &sch.override_entry;
    for entry in &sch.open_device_entries {
        let action = action_with_override(device_override, Action::Accept).script();
        dest.push_str(&format!("iptables -A {} -m mac --mac-source {} -j {}\n",
                               new_chain,
                               entry.item.mac,
                               action));
    }

    for dev in &world.closed_devices {
        let action = action_with_override(device_override, Action::Drop).script();
        dest.push_str(&format!("iptables -A {} -m mac --mac-source {} -j {}\n",
                               new_chain,
                               dev.mac,
                               action));
    }

    if world.schedule.guest_entry.item == GuestPath::Closed {
        dest.push_str(&format!("iptables -A {} -j DROP\n", new_chain));
    }

    dest.push_str(&format!("
iptables -I FORWARD 1 -j {new}
if iptables -L {old} >/dev/null 2>&1; then
    iptables -D FORWARD -j {old}
    iptables -F {old}
    iptables -X {old}
fi
",
                           new = new_chain,
                           old = old_chain));

}
