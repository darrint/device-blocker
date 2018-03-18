use std::io::Read;
use std::ops::DerefMut;
use iron::{Request, Response, IronResult, IronError, Handler};
use iron::Plugin;
use iron::status;
use iron::method;
use iron::mime::Mime;
use iron::Error;
use juniper::{RootNode, InputValue};
use juniper::http;
use serde_json;
use urlencoded::{UrlEncodedQuery};

use app_server::AppServerSchedulerWrapped;
use types::World;
use errors::ErrorKind;

impl ::iron::Error for ErrorKind {
    fn description(&self) -> &str {
        self.description()
    }
}

pub struct QueryRoot;

graphql_object!(QueryRoot: AppServerSchedulerWrapped |&self| {
    field world(&executor) -> World {
        let app_server_scheduler_wrapped = executor.context();
        let mut guard = app_server_scheduler_wrapped
            .wrapped_server.lock().unwrap();
        let app_server = guard.deref_mut();

        app_server.world.clone()
    }
});

pub struct MutationRoot;

graphql_object!(MutationRoot: AppServerSchedulerWrapped |&self| {
    field foo() -> String {
        "Bar".to_owned()
    }
});

fn iron_err(msg: &str) -> IronError {
    IronError::new(ErrorKind::RequestError(msg.to_owned()), status::BadRequest)
}

// Lifted from juniper iron.
fn get_single_value<T>(mut values: Vec<T>) -> IronResult<T> {
    if values.len() == 1 {
        Ok(values.remove(0))
    } else {
        Err(iron_err("duplicate or missing url param"))
    }
}

// Lifted from juniper iron.
fn parse_url_param(params: Option<Vec<String>>) -> IronResult<Option<String>> {
    if let Some(values) = params {
        get_single_value(values).map(Some)
    } else {
        Ok(None)
    }
}

// Lifted from juniper iron.
fn parse_variable_param(params: Option<Vec<String>>) -> IronResult<Option<InputValue>> {
    if let Some(values) = params {
        Ok(
            serde_json::from_str::<InputValue>(get_single_value(values)?.as_ref())
                .map(Some).map_err(|e| iron_err(e.description()))?
        )
    } else {
        Ok(None)
    }
}

pub struct GraphQLHandler<'a> {
    app_server_scheduler_wrapped: AppServerSchedulerWrapped,
    root_node: RootNode<'a, QueryRoot, MutationRoot>,
}

// Lifted from juniper_iron
impl <'a> GraphQLHandler<'a>
{
    /// Build a new GraphQL handler
    ///
    /// The context factory will receive the Iron request object and is
    /// expected to construct a context object for the given schema. This can
    /// be used to construct e.g. database connections or similar data that
    /// the schema needs to execute the query.
    pub fn new(app_server_scheduler_wrapped: AppServerSchedulerWrapped, query: QueryRoot, mutation: MutationRoot) -> Self {
        GraphQLHandler {
            app_server_scheduler_wrapped,
            root_node: RootNode::new(query, mutation),
        }
    }

    fn handle_get(&self, req: &mut Request) -> IronResult<http::GraphQLRequest> {
        let url_query_string = req.get_mut::<UrlEncodedQuery>().map_err(|e| iron_err(e.description()))?;

        let input_query = parse_url_param(url_query_string.remove("query"))?
            .ok_or_else(|| iron_err("No query provided"))?;
        let operation_name = parse_url_param(url_query_string.remove("operationName"))?;
        let variables = parse_variable_param(url_query_string.remove("variables"))?;

        Ok(http::GraphQLRequest::new(
            input_query,
            operation_name,
            variables,
        ))
    }

    fn handle_post(&self, req: &mut Request) -> IronResult<http::GraphQLRequest> {
        let mut request_payload = String::new();
        itry!(req.body.read_to_string(&mut request_payload));

        Ok(
            serde_json::from_str::<http::GraphQLRequest>(request_payload.as_str())
                .map_err(|err| IronError::new(err, status::BadRequest))?,
        )
    }

    fn execute(&self, request: &http::GraphQLRequest) -> IronResult<Response> {
        let response = request.execute(&self.root_node, &self.app_server_scheduler_wrapped.clone());
        let content_type = "application/json".parse::<Mime>().unwrap();
        let json = serde_json::to_string_pretty(&response).unwrap();
        let status = if response.is_ok() {
            status::Ok
        } else {
            status::BadRequest
        };
        Ok(Response::with((content_type, status, json)))
    }
}

impl <'a> Handler for GraphQLHandler<'a> where 'a: 'static {
    fn handle(&self, mut req: &mut Request) -> IronResult<Response> {
        let graphql_request = match req.method {
            method::Get => self.handle_get(&mut req)?,
            method::Post => self.handle_post(&mut req)?,
            _ => return Ok(Response::with(status::MethodNotAllowed)),
        };

        self.execute(&graphql_request)
    }
}
