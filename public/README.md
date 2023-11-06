This is an integration for making your flow function triggerable from webhooks in [flows.network](https://flows.network).

## Usage example
```rust
use webhook_flows::{create_endpoint, request_handler, send_response};

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn on_deploy() {
    create_endpoint().await;
}

#[request_handler]
async fn handler(_headers: Vec<(String, String)>, _subpath: String, _qry: HashMap<String, Value>, _body: Vec<u8>) {
    send_response(
        200,
        vec![(String::from("content-type"), String::from("text/html"))],
        "ok".as_bytes().to_vec(),
    );
}
```

When a request is received, the fn `handler` decorated by macro [`request_handler`](https://docs.rs/webhook-flows/latest/webhook_flows/attr.request_handler.html) will be called. We get the headers, subpath, query and body then set the status, headers and body of the response using [`send_response`](https://docs.rs/webhook-flows/latest/webhook_flows/fn.send_response.html).

You can set method as arguments of `request_handler` to specify which
http method to reponse:
```rust
#[request_handler(GET, POST)]
async fn handler(_headers: Vec<(String, String)>, _subpath: String, _qry: HashMap<String, Value>, _body: Vec<u8>) {
    send_response(
        200,
        vec![(String::from("content-type"), String::from("text/html"))],
        "ok".as_bytes().to_vec(),
    );
}
```
In this case, request with methods other than GET and POST will receive
METHOD_NOT_ALLOWED response. If no method has been speicified, all methods
will be handled.

There is a [`route`](https://docs.rs/webhook-flows/latest/webhook_flows/route/index.html) module for routing paths to different handler functions.
```rust
use webhook_flows::{
    create_endpoint, request_handler,
    route::{get, options, route, RouteError, Router},
    send_response,
};

#[request_handler]
async fn handler() {
    let mut router = Router::new();
    router
        .insert("/options", vec![options(opt)])
        .unwrap();

    router
        .insert(
            "/query/:city",
            vec![options(opt), get(query)],
        )
        .unwrap();

    if let Err(e) = route(router).await {
        match e {
            RouteError::NotFound => {
                send_response(404, vec![], b"No route matched".to_vec());
            }
            RouteError::MethodNotAllowed => {
                send_response(405, vec![], b"Method not allowed".to_vec());
            }
        }
    }
}

async fn options(
    _headers: Vec<(String, String)>,
    _qry: HashMap<String, Value>,
    _body: Vec<u8>,
) {
    // send_response(...)
}

async fn query(
    _headers: Vec<(String, String)>,
    qry: HashMap<String, Value>,
    _body: Vec<u8>,
) {
    // Wildcard in the path will be set in the `qry`.
    let city =  qry.get("city");

    // send_response(...)
}
```

This time, we don't need any arguments in the fn `handler` decorated by macro `request_handler`. Instead we should construct a [`Router`](https://docs.rs/webhook-flows/latest/webhook_flows/route/struct.Router.html), fill it with pairs of path and (Vec<>, [Handler](https://docs.rs/webhook-flows/latest/webhook_flows/route/fn.wrap_handler.html)), then call [`route`](https://docs.rs/webhook-flows/latest/webhook_flows/route/fn.route.html) on it. And In this circumstance, the handler fn would not receive the `subpath` argument.

The whole document is [here](https://docs.rs/webhook-flows).
