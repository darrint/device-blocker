use nickel::{Nickel, HttpRouter, FormBody, ErrorHandler, NickelError, Request, Action, Continue};
use nickel::status::StatusCode;

use serde_json;

use std::fmt::Write;
use chrono::{UTC, Duration};

use app_server::{AppServerSchedulerWrapped, Scheduler};

use ::errors::{Result, ResultExt, Error, ErrorKind};

#[derive(Clone, Copy)]
pub struct LogErrorHandler;

impl<D> ErrorHandler<D> for LogErrorHandler {
    fn handle_error(&self, err: &mut NickelError<D>, _req: &mut Request<D>) -> Action {
        println!("Error: {}", &err.message);
        Continue(())
    }
}

fn concat_err<T>(res: Result<T>) -> Result<T> {
    res.map_err(|top_err| {
        let mut message = String::new();
        for err in top_err.iter() {
            writeln!(message, "{:?}", err).expect("Failed to build error message");
        }
        message.into()
    })
}

trait ConcatExt<T> {
    fn concat(self) -> Result<T>;
    fn status_err(self) -> ::std::result::Result<T, (StatusCode, Error)>;
}

fn set_status_for_error(err: Error) -> (StatusCode, Error) {
    match err {
        Error(ErrorKind::RequestError(_), _) => (StatusCode::BadRequest, err),
        _ => (StatusCode::InternalServerError, err),
    }
}

impl<T> ConcatExt<T> for Result<T> {
    fn concat(self) -> Result<T> {
        concat_err(self)
    }
    fn status_err(self) -> ::std::result::Result<T, (StatusCode, Error)> {
        self.concat().map_err(|e| set_status_for_error(e))
    }
}

pub fn run_server(app_server: AppServerSchedulerWrapped) {
    let mut server = Nickel::with_data(app_server);

    server.handle_error(LogErrorHandler {});
    server.get("/api",
               middleware!(|req, res| <AppServerSchedulerWrapped>
        let app_server_scheduler = &mut req.server_data();
        app_server_scheduler.clone().kick_scheduler();
        let app_server = &app_server_scheduler.wrapped_server.lock().unwrap();
        let world = &app_server.world;
        try_with!(
            res,
            serde_json::to_string_pretty(world)
                .or(Err("Failed to serialize".to_owned())))
    ));

    server.post("/api/device/open",
                middleware!(|req, res| <AppServerSchedulerWrapped>
        let app_server_scheduler = &mut req.server_data();
        let app_server = &mut app_server_scheduler.wrapped_server.lock().unwrap();
        app_server_scheduler.clone().kick_scheduler();
        let params = try_with!(res, req.form_body());
        let mac_param = params.get("mac");
        let optional_time_secs_string = params.get("time_secs");
        let time_bound = try_with!(
            res,
            match optional_time_secs_string {
                None => Ok(None),
                Some(tss) => tss.parse::<i64>()
                    .map(|secs| Some(UTC::now() + Duration::seconds(secs))),
            }.chain_err(|| "Failed to parse time secs.").status_err());
        try_with!(
            res,
            app_server.open_device(mac_param, time_bound).status_err());
        try_with!(
            res,
            serde_json::to_string_pretty(&app_server.world)
                .or(Err("Failed to serialize".to_owned())))
    ));

    server.post("/api/device/close",
                middleware!(|req, res| <AppServerSchedulerWrapped>
        let app_server_scheduler = &mut req.server_data();
        app_server_scheduler.clone().kick_scheduler();
        let app_server = &mut app_server_scheduler.wrapped_server.lock().unwrap();
        let params = try_with!(res, req.form_body());
        let mac_param = params.get("mac");
        try_with!(
            res,
            app_server.close_device(mac_param).status_err());
        try_with!(
            res,
            serde_json::to_string_pretty(&app_server.world)
                .or(Err("Failed to serialize".to_owned())))
    ));

    server.post("/api/guest",
                middleware!(|req, res| <AppServerSchedulerWrapped>
        let app_server_scheduler = &mut req.server_data();
        app_server_scheduler.clone().kick_scheduler();
        let app_server = &mut app_server_scheduler.wrapped_server.lock().unwrap();
        let params = try_with!(res, req.form_body());
        let allow_param = params.get("allow");
        try_with!(
            res,
            app_server.set_guest_path(allow_param, None).status_err());
        try_with!(
            res,
            serde_json::to_string_pretty(&app_server.world)
                .or(Err("Failed to serialize".to_owned())))
    ));

    server.post("/api/override_all",
                middleware!(|req, res| <AppServerSchedulerWrapped>
        let app_server_scheduler = &mut req.server_data();
        app_server_scheduler.clone().kick_scheduler();
        let app_server = &mut app_server_scheduler.wrapped_server.lock().unwrap();
        let params = try_with!(res, req.form_body());
        let override_param = params.get("override");
        try_with!(
            res,
            app_server.set_device_override(override_param, None).status_err());
        try_with!(
            res,
            serde_json::to_string_pretty(&app_server.world)
                .or(Err("Failed to serialize".to_owned())))
    ));

    server.post("/api/add_device",
                middleware!(|req, res| <AppServerSchedulerWrapped>
        let app_server_scheduler = &mut req.server_data();
        app_server_scheduler.clone().kick_scheduler();
        let app_server = &mut app_server_scheduler.wrapped_server.lock().unwrap();
        let params = try_with!(res, req.form_body());
        let mac_param = params.get("mac");
        let name_param = params.get("name");
        try_with!(
            res,
            app_server.add_device(mac_param, name_param).status_err());
        try_with!(
            res,
            serde_json::to_string_pretty(&app_server.world)
                .or(Err("Failed to serialize".to_owned())))
    ));

    let bind = "0.0.0.0:8000";
    server.listen(bind).unwrap();
}
