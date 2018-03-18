use std::collections::BTreeSet;
use std::sync::{Arc, Mutex, Condvar};
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::ops::DerefMut;
use files::{write_json_file, read_json_file};
use schedule::{World, Device, ScheduleEntry, GuestPath, DeviceOverride};
use script_handler::ScriptHandler;
use config::{Config, reconcile_config};
use chrono::{DateTime, Utc};
use time::Duration;
use ::script_handler::HandleScript;
use ::script::write_script;
use errors::{Result, ResultExt, ErrorKind};

pub type AppServerWrapped = Arc<Mutex<AppServer>>;

pub fn new_wrapped_scheduler(wrapped_server: AppServerWrapped) -> AppServerScheduler {
    AppServerScheduler {
        wrapped_server,
        condvar: Condvar::new(),
    }
}

pub struct AppServerScheduler {
    pub wrapped_server: AppServerWrapped,
    condvar: Condvar,
}

pub type AppServerSchedulerWrapped = Arc<AppServerScheduler>;

pub trait Scheduler {
    fn kick_scheduler(self);
}

impl Scheduler for AppServerSchedulerWrapped {
    fn kick_scheduler(self) {
        let scheduler = &self;
        scheduler.condvar.notify_one();
    }
}

pub fn run_expiration(wrapped_scheduler: &mut AppServerSchedulerWrapped) {
    let condvar = &wrapped_scheduler.condvar;
    let mut guard = wrapped_scheduler.wrapped_server.lock().unwrap();
    loop {
        let option_max_date: Option<DateTime<Utc>> = {
            let world = &guard.world;
            world.get_soonest_event_time()
        };
        let now : DateTime<Utc> = Utc::now();
        let dur = option_max_date.map(|max_date|
            max_date.signed_duration_since(now)).unwrap_or_else(|| Duration::days(30));
        let std_dur = dur.to_std().unwrap_or_else(|_| ::std::time::Duration::new(0, 0));
        let (g2, _) = condvar.wait_timeout(guard, std_dur).unwrap();
        guard = g2;
        {
            {
                let world = &mut guard.deref_mut().world;
                let now = Utc::now();
                world.expire_bounded(now);
            }
            guard.refresh_world().unwrap_or_else(|err| println!("{:?}", err));
        };
    }
}

pub trait RequestErrExt<'a> {
    fn require_param(&self, String) -> Result<&'a str>;
}

impl<'a> RequestErrExt<'a> for Option<&'a str> {
    fn require_param(&self, msg: String) -> Result<&'a str> {
        self.ok_or(ErrorKind::RequestError(msg).into())
    }
}

pub struct AppServer {
    pub world: World,
    pub handler: ScriptHandler,
    pub config: Config,
    pub config_file: String,
}

impl AppServer {
    pub fn open_device(&mut self,
                       mac_param: Option<&str>,
                       time_bound: Option<DateTime<Utc>>)
                       -> Result<()> {
        let mac = mac_param.require_param("Missing mac parameter".to_owned())?;
        self.world.open_device(mac, time_bound)?;
        self.refresh_world()
    }

    pub fn close_device(&mut self, mac_param: Option<&str>) -> Result<()> {
        let mac = mac_param.require_param("Missing mac parameter".to_owned())?;
        self.world.close_device(mac)?;
        self.refresh_world()
    }

    pub fn set_guest_path(&mut self,
                          allow_param: Option<&str>,
                          time_bound: Option<DateTime<Utc>>)
                          -> Result<()> {
        let allow_str = allow_param.require_param("Missing allow parameter".to_owned())?;
        let allow = if allow_str.to_lowercase() == "true" {
            GuestPath::Open
        } else {
            GuestPath::Closed
        };
        self.world.schedule.guest_entry = ScheduleEntry {
            item: allow,
            time_bound,
        };
        self.refresh_world()
    }

    pub fn set_device_override(&mut self,
                               override_param: Option<&str>,
                               time_bound: Option<DateTime<Utc>>)
                               -> Result<()> {
        let override_str = override_param.require_param("Missing override paramter".to_owned())?;
        let override_arg = if override_str.to_lowercase() == "null" {
            None
        } else if override_str.to_lowercase() == "true" {
            Some(DeviceOverride::Open)
        } else {
            Some(DeviceOverride::Closed)
        };
        self.world.schedule.override_entry = override_arg.map(|i| {
            ScheduleEntry {
                item: i,
                time_bound,
            }
        });
        self.refresh_world()
    }

    pub fn add_device(&mut self, mac_param: Option<&str>, name_param: Option<&str>) -> Result<()> {
        let mac = mac_param.require_param("Missing mac parameter".to_owned())?;
        let name = name_param.require_param("Missing name parameter".to_owned())?;
        let dev = Device {
            mac: mac.to_owned(),
            name: name.to_owned(),
        };
        self.config.known_devices.insert(dev);
        self.write_config()?;
        let mut devs = BTreeSet::new();
        read_dhcp_devices(&self.config.dhcp_lease_file, &mut devs)?;
        let reconcile_result = {
            reconcile_config(&self.config, &devs, &mut self.world)
        };
        if reconcile_result.updated_world {
            self.refresh_world()?;
        }
        Ok(())
    }

    pub fn refresh_devices(&mut self) -> Result<()> {
        let mut devs = BTreeSet::new();
        read_dhcp_devices(&self.config.dhcp_lease_file, &mut devs)?;
        let reconcile_result = {
            reconcile_config(&self.config, &devs, &mut self.world)
        };
        if reconcile_result.updated_world {
            self.refresh_world()?;
        }
        Ok(())
    }

    fn handle_script(&self) -> Result<()> {
        let mut script = String::new();
        write_script(&self.world,
                     "old_blocked_devices",
                     "blocked_devices",
                     &self.config.exit_interfaces,
                     &mut script);
        self.handler.handle(&script)
    }

    pub fn refresh_world(&self) -> Result<()> {
        self.write_world()?;
        self.handle_script()
    }

    pub fn write_world(&self) -> Result<()> {
        write_json_file(&self.config.state_file, &self.world)
            .chain_err(|| "Failed to write new state_file")
    }

    fn write_config(&self) -> Result<()> {
        write_json_file(&self.config_file, &self.config)
            .chain_err(|| "Failed to write new config file")
    }

    pub fn read_or_create_world(&mut self) -> Result<()> {
        self.world = read_json_file(&self.config.state_file).unwrap_or_default();
        self.write_world()
    }
}

pub fn read_dhcp_devices(dhcp_leases_file: &str, devs: &mut BTreeSet<Device>) -> Result<()> {
    let reader =
        File::open(dhcp_leases_file).chain_err(|| "Failed to open dhcp lease file.")?;
    let buf_reader = BufReader::new(reader);
    for line_result in buf_reader.lines() {
        let line = line_result.chain_err(|| "Failed to read line")?;
        let parts: Vec<String> = line.splitn(4, |c: char| c.is_whitespace())
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let mac = &parts[1];
        let name = &parts[3];
        let dev = Device {
            mac: mac.clone(),
            name: name.clone(),
        };
        devs.insert(dev);
    }
    Ok(())
}
