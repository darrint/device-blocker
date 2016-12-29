#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;

extern crate serde_json;

mod script;
mod schedule;
mod errors {
    error_chain! { }
}

use script::write_script;
use schedule::{
    World,
    Schedule,
    ScheduleEntry,
    Device,
    DeviceOverride,
    GuestPath,
};

use errors::{Result, ResultExt};

quick_main!(run);


fn run() -> Result<()> {
    let new_chain = String::from("after");
    let old_chain = String::from("before");

    let mut world = World{
        schedule: Schedule{
            guest_entry: ScheduleEntry{
                item: GuestPath::Closed,
                time_bound: None,
            },
            override_entry: None, /*Some(ScheduleEntry{
                item: DeviceOverride::Open,
                time_bound: None,
            }),*/
            open_device_entries: vec!(),
        },
        closed_devices: vec!(
            Device{
                name: String::from("Darrin Tablet"),
                mac: String::from("abc"),
            },
            Device{
                name: String::from("Connor Tablet"),
                mac: String::from("def"),
            },
        ),
    };

    world.open_device("abc", None).unwrap_or(());
    println!("{:#?}", world);
    println!();
    let mut dest = String::new();
    write_script(&world, Some(&old_chain), &new_chain, &mut dest);
    println!("script:\n\n{}", dest);

    Ok(())
}
