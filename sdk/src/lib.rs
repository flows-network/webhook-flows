//! Make a flow function triggerable from webhooks in [Flows.network](https://flows.network)
//!
//! # Quick Start
//!
//! To get started, let's write a very tiny flow function.
//!
//! ```rust
//! use webhook_flows::{request_received, send_response};
//!
//! #[no_mangle]
//! #[tokio::main(flavor = "current_thread")]
//! pub async fn run() {
//!     request_received(handler).await;
//! }
//!
//! async fn handler(_headers: Vec<(String, String)>, _qry: HashMap<String, Value>, _body: Vec<u8>) {
//!     send_response(
//!         200,
//!         vec![(String::from("content-type"), String::from("text/html"))],
//!         "ok".as_bytes().to_vec(),
//!     );
//! }
//! ```
//!
//! When a new request is received the callback closure of function [request_received()] will be called and [send_response()] is used to make the response.

use http_req::request;
use lazy_static::lazy_static;
use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;

lazy_static! {
    static ref WEBHOOK_API_PREFIX: String = String::from(
        std::option_env!("WEBHOOK_API_PREFIX").unwrap_or("https://webhook.flows.network/api")
    );
}
const WEBHOOK_ENTRY_URL: &str = "https://code.flows.network/webhook";

extern "C" {
    fn is_listening() -> i32;
    fn get_flows_user(p: *mut u8) -> i32;
    fn get_flow_id(p: *mut u8) -> i32;
    fn get_event_headers_length() -> i32;
    fn get_event_headers(p: *mut u8) -> i32;
    fn get_event_query_length() -> i32;
    fn get_event_query(p: *mut u8) -> i32;
    fn get_event_body_length() -> i32;
    fn get_event_body(p: *mut u8) -> i32;
    fn set_error_log(p: *const u8, len: i32);
    fn set_output(p: *const u8, len: i32);
    fn set_response(p: *const u8, len: i32);
    fn set_response_headers(p: *const u8, len: i32);
    fn set_response_status(status: i32);
    // fn redirect_to(p: *const u8, len: i32);
}

/// Register a callback closure with the query and body of the request for the webhook service.
///
/// The query is formed as a [HashMap]. For example, say the entrypoint of the webhook service is
/// `https://code.flows.network/webhook/6rtSi9SEsC?param=hello`
/// then the query will look like `HashMap("param", Value::String("hello"))`
///
/// The body is the raw bytes of the request body.
pub async fn request_received<F, Fut>(cb: F)
where
    F: FnOnce(Vec<(String, String)>, HashMap<String, Value>, Vec<u8>) -> Fut,
    Fut: Future<Output = ()>,
{
    unsafe {
        match is_listening() {
            // Calling register
            1 => {
                let mut flows_user = Vec::<u8>::with_capacity(100);
                let c = get_flows_user(flows_user.as_mut_ptr());
                flows_user.set_len(c as usize);
                let flows_user = String::from_utf8(flows_user).unwrap();

                let mut flow_id = Vec::<u8>::with_capacity(100);
                let c = get_flow_id(flow_id.as_mut_ptr());
                if c == 0 {
                    panic!("Failed to get flow id");
                }
                flow_id.set_len(c as usize);
                let flow_id = String::from_utf8(flow_id).unwrap();

                let mut writer = Vec::new();
                let res = request::get(
                    format!(
                        "{}/{}/{}/listen",
                        WEBHOOK_API_PREFIX.as_str(),
                        flows_user,
                        flow_id
                    ),
                    &mut writer,
                )
                .unwrap();

                match res.status_code().is_success() {
                    true => {
                        let listener: Value = serde_json::from_slice(&writer).unwrap();
                        let l_key = listener["l_key"].as_str().unwrap();
                        let output = format!("Webhook endpoint: {}/{}", WEBHOOK_ENTRY_URL, l_key);
                        set_output(output.as_ptr(), output.len() as i32);
                    }
                    false => {
                        set_error_log(writer.as_ptr(), writer.len() as i32);
                    }
                }
            }
            _ => {
                if let Some((header, qry, body)) = message_from_request() {
                    cb(header, qry, body).await;
                }
            }
        }
    }
}

/// Set the response for the webhook service.
pub fn send_response(status: u16, headers: Vec<(String, String)>, body: Vec<u8>) {
    let headers = serde_json::to_string(&headers).unwrap_or_default();
    unsafe {
        set_response_status(status as i32);
        set_response_headers(headers.as_ptr(), headers.len() as i32);
        set_response(body.as_ptr(), body.len() as i32);
    }
}

fn message_from_request() -> Option<(Vec<(String, String)>, HashMap<String, Value>, Vec<u8>)> {
    unsafe {
        let l = get_event_headers_length();
        let mut event_headers = Vec::<u8>::with_capacity(l as usize);
        let c = get_event_headers(event_headers.as_mut_ptr());
        assert!(c == l);
        event_headers.set_len(c as usize);
        let event_headers = serde_json::from_slice(&event_headers).unwrap();

        let l = get_event_query_length();
        let mut event_query = Vec::<u8>::with_capacity(l as usize);
        let c = get_event_query(event_query.as_mut_ptr());
        assert!(c == l);
        event_query.set_len(c as usize);
        let event_query = serde_json::from_slice(&event_query).unwrap();

        let l = get_event_body_length();
        let mut event_body = Vec::<u8>::with_capacity(l as usize);
        let c = get_event_body(event_body.as_mut_ptr());
        assert!(c == l);
        event_body.set_len(c as usize);

        Some((event_headers, event_query, event_body))
    }
}
