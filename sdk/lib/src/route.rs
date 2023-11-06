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

/// Helper for wrapping function to a Handler, and then binding to 'GET' method.
pub fn get<T>(
    f: fn(Vec<(String, String)>, HashMap<String, Value>, Vec<u8>) -> T,
) -> (Method, Handler)
where
    T: Future<Output = ()> + 'static,
{
    (Method::GET, Box::new(move |a, b, c| Box::pin(f(a, b, c))))
}

/// Helper for wrapping function to a Handler, and then binding to 'OPTIONS' method.
pub fn options<T>(
    f: fn(Vec<(String, String)>, HashMap<String, Value>, Vec<u8>) -> T,
) -> (Method, Handler)
where
    T: Future<Output = ()> + 'static,
{
    (
        Method::OPTIONS,
        Box::new(move |a, b, c| Box::pin(f(a, b, c))),
    )
}

/// Helper for wrapping function to a Handler, and then binding to 'POST' method.
pub fn post<T>(
    f: fn(Vec<(String, String)>, HashMap<String, Value>, Vec<u8>) -> T,
) -> (Method, Handler)
where
    T: Future<Output = ()> + 'static,
{
    (Method::POST, Box::new(move |a, b, c| Box::pin(f(a, b, c))))
}

/// Helper for wrapping function to a Handler, and then binding to 'PUT' method.
pub fn put<T>(
    f: fn(Vec<(String, String)>, HashMap<String, Value>, Vec<u8>) -> T,
) -> (Method, Handler)
where
    T: Future<Output = ()> + 'static,
{
    (Method::PUT, Box::new(move |a, b, c| Box::pin(f(a, b, c))))
}

/// Helper for wrapping function to a Handler, and then binding to 'DELETE' method.
pub fn delete<T>(
    f: fn(Vec<(String, String)>, HashMap<String, Value>, Vec<u8>) -> T,
) -> (Method, Handler)
where
    T: Future<Output = ()> + 'static,
{
    (
        Method::DELETE,
        Box::new(move |a, b, c| Box::pin(f(a, b, c))),
    )
}

/// Helper for wrapping function to a Handler, and then binding to 'HEAD' method.
pub fn head<T>(
    f: fn(Vec<(String, String)>, HashMap<String, Value>, Vec<u8>) -> T,
) -> (Method, Handler)
where
    T: Future<Output = ()> + 'static,
{
    (Method::HEAD, Box::new(move |a, b, c| Box::pin(f(a, b, c))))
}

/// Helper for wrapping function to a Handler, and then binding to 'TRACE' method.
pub fn trace<T>(
    f: fn(Vec<(String, String)>, HashMap<String, Value>, Vec<u8>) -> T,
) -> (Method, Handler)
where
    T: Future<Output = ()> + 'static,
{
    (Method::TRACE, Box::new(move |a, b, c| Box::pin(f(a, b, c))))
}

/// Helper for wrapping function to a Handler, and then binding to 'PATCH' method.
pub fn patch<T>(
    f: fn(Vec<(String, String)>, HashMap<String, Value>, Vec<u8>) -> T,
) -> (Method, Handler)
where
    T: Future<Output = ()> + 'static,
{
    (Method::PATCH, Box::new(move |a, b, c| Box::pin(f(a, b, c))))
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
///     .insert("/options", vec![options(opt)])
///     .unwrap();
/// router
///     .insert(
///         "/get/:city",
///         vec![options(opt), get(query)],
///     )
///     .unwrap();
/// if let Err(e) = route(router).await {
///     send_response(404, vec![], b"No route matched".to_vec())
/// }
/// ```
pub async fn route(router: Router<Vec<(Method, Handler)>>) -> Result<(), RouteError> {
    let (method, headers, subpath, mut qry, body) = get_request();
    let matched = router.at(subpath.as_str()).or(Err(RouteError::NotFound))?;
    for p in matched.params.iter() {
        qry.insert(String::from(p.0), Value::from(p.1));
    }
    let mh = matched.value;
    for (m, h) in mh.iter() {
        if m.eq(&method) {
            h(headers, qry, body).await;
            return Ok(());
        }
    }

    Err(RouteError::MethodNotAllowed)
}
