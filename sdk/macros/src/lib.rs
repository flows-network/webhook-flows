use proc_macro::TokenStream;
use quote::{quote, ToTokens};

#[proc_macro_attribute]
pub fn request_handler(_: TokenStream, item: TokenStream) -> TokenStream {
    let ast: syn::ItemFn = syn::parse(item).unwrap();
    let func_ident = ast.sig.ident.clone();

    let gen = quote! {
        mod webhook_flows_macros {
            extern "C" {
                pub fn get_event_headers_length() -> i32;
                pub fn get_event_headers(p: *mut u8) -> i32;
                pub fn get_event_query_length() -> i32;
                pub fn get_event_query(p: *mut u8) -> i32;
                pub fn get_event_body_length() -> i32;
                pub fn get_event_body(p: *mut u8) -> i32;
            }
        }

        fn __request() -> Option<(Vec<(String, String)>, HashMap<String, Value>, Vec<u8>)> {
            unsafe {
                let l = webhook_flows_macros::get_event_headers_length();
                let mut event_headers = Vec::<u8>::with_capacity(l as usize);
                let c = webhook_flows_macros::get_event_headers(event_headers.as_mut_ptr());
                assert!(c == l);
                event_headers.set_len(c as usize);
                let event_headers = serde_json::from_slice(&event_headers).unwrap();

                let l = webhook_flows_macros::get_event_query_length();
                let mut event_query = Vec::<u8>::with_capacity(l as usize);
                let c = webhook_flows_macros::get_event_query(event_query.as_mut_ptr());
                assert!(c == l);
                event_query.set_len(c as usize);
                let event_query = serde_json::from_slice(&event_query).unwrap();

                let l = webhook_flows_macros::get_event_body_length();
                let mut event_body = Vec::<u8>::with_capacity(l as usize);
                let c = webhook_flows_macros::get_event_body(event_body.as_mut_ptr());
                assert!(c == l);
                event_body.set_len(c as usize);

                Some((event_headers, event_query, event_body))
            }
        }

        #[no_mangle]
        #[tokio::main(flavor = "current_thread")]
        pub async fn __webhook__on_request_received() {
            if let Some((headers, qry, body)) = __request() {
                #func_ident(headers, qry, body).await;
            }
        }
    };

    let ori_run_str = ast.to_token_stream().to_string();
    let x = gen.to_string() + &ori_run_str;
    x.parse().unwrap()
}
