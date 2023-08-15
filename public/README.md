This is an integration for making your flow function triggerable from webhooks in [flows.network](https://flows.network).

## Usage example
```rust
use webhook_flows::{request_received, send_response};

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn run() {
    request_received(handler).await;
}

async fn handler(_headers: Vec<(String, String)>, _qry: HashMap<String, Value>, _body: Vec<u8>) {
    send_response(
        200,
        vec![(String::from("content-type"), String::from("text/html"))],
        "ok".as_bytes().to_vec(),
    );
}
```

When a request is received, the callback closure in [`request_received`](https://docs.rs/webhook-flows/latest/webhook_flows/fn.request_received.html) will be called. We get the headers, query and body then set the status, headers and body of the response using [`send_response`](https://docs.rs/webhook-flows/latest/webhook_flows/fn.send_response.html).

The whole document is [here](https://docs.rs/webhook-flows).
