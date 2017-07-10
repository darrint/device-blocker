use iron::{Iron, Handler, Request, Response, IronResult, Plugin};
use iron::headers::{ETag, EntityTag};
use iron::modifiers::Header;
use iron::status;
use iron::mime::Mime;
use router::Router;
use params::{Params, Value};
use std::ops::DerefMut;
use checksum::crc64::Crc64;

use serde_json;

use chrono::{UTC, Duration};

use app_server::{AppServerSchedulerWrapped, AppServer, Scheduler};

use ::errors::{ResultExt};

macro_rules! define_handler {
    ($t:ident, $f:ident) => {
        struct $t {
            app_server_scheduler_wrapped: AppServerSchedulerWrapped,
        }

        impl $t {
            fn new(app_server_scheduler_wrapped: AppServerSchedulerWrapped)
                    -> $t {
                $t {
                    app_server_scheduler_wrapped: app_server_scheduler_wrapped,
                }
            }
        }

        impl Handler for $t {
            fn handle(&self, req: &mut Request) -> IronResult<Response> {
                let mut guard = self.app_server_scheduler_wrapped
                    .wrapped_server.lock().unwrap();
                let app_server = guard.deref_mut();
                let scheduler = self.app_server_scheduler_wrapped.clone();
                $f(scheduler, app_server, req)
            }
        }
    }
}

const INDEX_HTML: &'static [u8] = include_bytes!("index.html");
const BUNDLE_JS: &'static [u8] = include_bytes!("bundle.js");

define_handler!(GetWorldHandler, get_world);
fn get_world(
        scheduler: AppServerSchedulerWrapped, app_server: &AppServer,
        _req: &mut Request) -> IronResult<Response> {
    scheduler.kick_scheduler();
    let world = &app_server.world;
    let serialized = itry!(serde_json::to_string_pretty(world));
    Ok(Response::with((status::Ok, serialized)))
}

define_handler!(OpenDeviceHandler, open_device);
fn open_device(
        scheduler: AppServerSchedulerWrapped, app_server: &mut AppServer,
        req: &mut Request) -> IronResult<Response> {
    scheduler.kick_scheduler();
    let params = itry!(req.get_ref::<Params>());

    let mac_value = params.find(&["mac"]);
    let mac_param = match mac_value {
        Some(&Value::String(ref m)) => Some(m.as_ref()),
        _ => None,
    };
    let optional_time_secs_string = params.find(&["time_secs"]);
    let time_bound = itry!(
        match optional_time_secs_string {
            Some(&Value::String(ref tss)) => tss.parse::<i64>()
                .map(|secs| Some(UTC::now() + Duration::seconds(secs))),
            _ => Ok(None),
        }.chain_err(|| "Failed to parse time secs."), status::BadRequest);
    itry!(app_server.open_device(mac_param, time_bound));
    let serialized = itry!(serde_json::to_string_pretty(&app_server.world));
    Ok(Response::with((status::Ok, serialized)))
}

define_handler!(CloseDeviceHandler, close_device);
fn close_device(
        scheduler: AppServerSchedulerWrapped, app_server: &mut AppServer,
        req: &mut Request) -> IronResult<Response> {
    scheduler.kick_scheduler();
    let params = itry!(req.get_ref::<Params>());
    let mac_value = params.find(&["mac"]);
    let mac_param = match mac_value {
        Some(&Value::String(ref m)) => Some(m.as_ref()),
        _ => None,
    };
    itry!(app_server.close_device(mac_param));
    let serialized = itry!(serde_json::to_string_pretty(&app_server.world));
    Ok(Response::with((status::Ok, serialized)))
}

define_handler!(SetGuestHandler, set_guest);
fn set_guest(
        scheduler: AppServerSchedulerWrapped, app_server: &mut AppServer,
        req: &mut Request) -> IronResult<Response> {
    scheduler.kick_scheduler();
    let params = itry!(req.get_ref::<Params>());
    let allow_value = params.find(&["allow"]);
    let allow_param = match allow_value {
        Some(&Value::String(ref m)) => Some(m.as_ref()),
        _ => None,
    };
    itry!(app_server.set_guest_path(allow_param, None));
    let serialized = itry!(serde_json::to_string_pretty(&app_server.world));
    Ok(Response::with((status::Ok, serialized)))
}

define_handler!(SetOverrideAllHandler, set_override_all);
fn set_override_all(
        scheduler: AppServerSchedulerWrapped, app_server: &mut AppServer,
        req: &mut Request) -> IronResult<Response> {
    scheduler.kick_scheduler();
    let params = itry!(req.get_ref::<Params>());
    let override_value = params.find(&["override"]);
    let override_param = match override_value {
        Some(&Value::String(ref m)) => Some(m.as_ref()),
        _ => None,
    };
    itry!(app_server.set_device_override(override_param, None));
    let serialized = itry!(serde_json::to_string_pretty(&app_server.world));
    Ok(Response::with((status::Ok, serialized)))
}

define_handler!(AddDeviceHandler, add_device);
fn add_device(
        scheduler: AppServerSchedulerWrapped, app_server: &mut AppServer,
        req: &mut Request) -> IronResult<Response> {
    scheduler.kick_scheduler();
    let params = itry!(req.get_ref::<Params>());
    let name_value = params.find(&["name"]);
    let name_param = match name_value {
        Some(&Value::String(ref m)) => Some(m.as_ref()),
        _ => None,
    };
    let mac_value = params.find(&["mac"]);
    let mac_param = match mac_value {
        Some(&Value::String(ref m)) => Some(m.as_ref()),
        _ => None,
    };
    itry!(app_server.add_device(mac_param, name_param));
    let serialized = itry!(serde_json::to_string_pretty(&app_server.world));
    Ok(Response::with((status::Ok, serialized)))
}

define_handler!(RefreshDevicesHandler, refresh_devices);
fn refresh_devices(
        scheduler: AppServerSchedulerWrapped, app_server: &mut AppServer,
        _req: &mut Request) -> IronResult<Response> {
    scheduler.kick_scheduler();
    itry!(app_server.refresh_devices());
    let serialized = itry!(serde_json::to_string_pretty(&app_server.world));
    Ok(Response::with((status::Ok, serialized)))
}


struct StaticHandler {
    buf: &'static [u8],
    etag: Header<ETag>,
    mime: Mime,
}

impl StaticHandler {
    fn new(buf: &'static [u8], etag: &str, mime: Mime) -> StaticHandler {
        let etag_header = Header(ETag(EntityTag::new(false, etag.to_owned())));

        StaticHandler {
            buf: buf,
            etag: etag_header,
            mime: mime,
        }
    }
}

impl Handler for StaticHandler {
    fn handle(&self, _req: &mut Request) -> IronResult<Response> {
        Ok(Response::with((
            self.mime.clone(), self.etag.clone(), status::Ok, self.buf)))
    }
}

pub fn run_server(app_server_wrapped: AppServerSchedulerWrapped) {
    let mut router = Router::new();

    router.get(
        "/api",
        GetWorldHandler::new(app_server_wrapped.clone()),
        "get_world");
    router.post(
        "/api/device/open",
        OpenDeviceHandler::new(app_server_wrapped.clone()),
        "open_device");
    router.post(
        "/api/device/close",
        CloseDeviceHandler::new(app_server_wrapped.clone()),
        "close_device");
    router.post("/api/guest",
        SetGuestHandler::new(app_server_wrapped.clone()),
        "set_guest");
    router.post("/api/override_all",
        SetOverrideAllHandler::new(app_server_wrapped.clone()),
        "set_override_all");
    router.post("/api/add_device",
        AddDeviceHandler::new(app_server_wrapped.clone()),
        "add_device");
    router.post("/api/refresh_devices",
        RefreshDevicesHandler::new(app_server_wrapped.clone()),
        "refresh_devices");

    let mut crc = Crc64::new();
    crc.update(INDEX_HTML);
    crc.update(BUNDLE_JS);
    let sum = format!("{:x}", crc.getsum());
    let mime_html: Mime = "text/html".parse().unwrap();
    let mime_js: Mime = "application/javascript".parse().unwrap();

    router.get("/", StaticHandler::new(INDEX_HTML, &sum, mime_html), "index");
    router.get(
        "/bundle.js", StaticHandler::new(BUNDLE_JS, &sum, mime_js),
        "bundle_js");

    let bind = "0.0.0.0:8000";
    Iron::new(router).http(bind).unwrap();
}
