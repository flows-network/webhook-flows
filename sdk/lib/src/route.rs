use super::Method;
use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

pub use matchit::Router;

type Handler = Box<
    dyn Fn(
        Vec<(String, String)>,
        HashMap<String, Value>,
        Vec<u8>,
    ) -> Pin<Box<dyn Future<Output = ()>>>,
>;

/// Helper for wrapping function to a Handler.
/// Then the Handler should be inserted into [Router].
pub fn wrap_handler<T>(
    f: fn(Vec<(String, String)>, HashMap<String, Value>, Vec<u8>) -> T,
) -> Handler
where
    T: Future<Output = ()> + 'static,
{
    Box::new(move |a, b, c| Box::pin(f(a, b, c)))
}

extern "C" {
    fn get_event_method_length() -> i32;
    fn get_event_method(p: *mut u8) -> i32;
    fn get_event_headers_length() -> i32;
    fn get_event_headers(p: *mut u8) -> i32;
    fn get_event_query_length() -> i32;
    fn get_event_query(p: *mut u8) -> i32;
    fn get_event_subpath_length() -> i32;
    fn get_event_subpath(p: *mut u8) -> i32;
    fn get_event_body_length() -> i32;
    fn get_event_body(p: *mut u8) -> i32;
}

fn get_request() -> (
    Method,
    Vec<(String, String)>,
    String,
    HashMap<String, Value>,
    Vec<u8>,
) {
    unsafe {
        let l = get_event_method_length();
        let mut event_method = Vec::<u8>::with_capacity(l as usize);
        let c = get_event_method(event_method.as_mut_ptr());
        assert!(c == l);
        event_method.set_len(c as usize);
        let event_method = Method::from_bytes(&event_method).unwrap();

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

        let l = get_event_subpath_length();
        let mut event_subpath = Vec::<u8>::with_capacity(l as usize);
        let c = get_event_subpath(event_subpath.as_mut_ptr());
        assert!(c == l);
        event_subpath.set_len(c as usize);
        let event_subpath = String::from_utf8_lossy(&event_subpath).into_owned();

        let l = get_event_body_length();
        let mut event_body = Vec::<u8>::with_capacity(l as usize);
        let c = get_event_body(event_body.as_mut_ptr());
        assert!(c == l);
        event_body.set_len(c as usize);

        (
            event_method,
            event_headers,
            event_subpath,
            event_query,
            event_body,
        )
    }
}

/// Route error types
pub enum RouteError {
    NotFound,
    MethodNotAllowed,
}

/// Route path to handler.
/// For calling the exact handler, [construct the router](Router) then pass it to this function.
/// ```rust
/// let mut router = Router::new();
/// router
///     .insert("/options", (vec![Method::OPTIONS], new_handler(options)))
///     .unwrap();
/// router
///     .insert(
///         "/get/:city",
///         (vec![Method::GET], new_handler(handler)),
///     )
///     .unwrap();
/// if let Err(e) = route(router).await {
///     send_response(404, vec![], b"No route matched".to_vec())
/// }
/// ```
pub async fn route(router: Router<(Vec<Method>, Handler)>) -> Result<(), RouteError> {
    let (method, headers, subpath, mut qry, body) = get_request();
    let matched = router.at(subpath.as_str()).or(Err(RouteError::NotFound))?;
    for p in matched.params.iter() {
        qry.insert(String::from(p.0), Value::from(p.1));
    }
    let (mv, f) = matched.value;
    for m in mv.iter() {
        if m.eq(&method) {
            f(headers, qry, body).await;
            return Ok(());
        }
    }

    Err(RouteError::MethodNotAllowed)
}
