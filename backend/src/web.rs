use std::collections::HashMap;

use anyhow::anyhow;
use graphql_client::{GraphQLQuery, Response};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode};

fn js_error(message: String) -> impl Fn(JsValue) -> anyhow::Error {
    move |jsvalue| {
        anyhow!(
            "{}: {}",
            message,
            js_sys::Error::from(jsvalue)
                .message()
                .as_string()
                .unwrap_or("No idea why.".to_string())
        )
    }
}

pub async fn gql_call<Q: GraphQLQuery>(
    _query: Q,
    variables: Q::Variables,
    endpoint: String,
    headers: HashMap<String, String>,
) -> anyhow::Result<Response<Q::ResponseData>> {
    let body = serde_json::to_string(&Q::build_query(variables))?;
    let window = web_sys::window().ok_or(anyhow!("Could not find window"))?;
    let mut request_init = RequestInit::new();
    request_init
        .headers(&JsValue::from_serde(&headers)?)
        .method("POST")
        .mode(RequestMode::Cors)
        .body(Option::Some(&JsValue::from_str(&body)));
    let request = Request::new_with_str_and_init(&endpoint, &request_init)
        .map_err(js_error("Could not create request".into()))?;
    let response = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(js_error("Could not execute request".into()))?;
    let text_promise = response
        .dyn_into::<web_sys::Response>()
        .map_err(js_error("Could not pare response".into()))
        .and_then(|cast| {
            cast.text()
                .map_err(js_error("Could not get text from response".into()))
        })?;

    let text = JsFuture::from(text_promise)
        .await
        .map_err(js_error("Could not resolve text promise".into()))
        .and_then(|jsvalue| jsvalue.as_string().ok_or(anyhow!("JsValue not a string")))?;
    let serde: serde_json::Value = serde_json::from_str(&text)?;
    log::trace!("{:#?}", serde);
    if serde["message"].is_string() {
        Err(anyhow!("Graphql error: {}", &serde["message"]))
    } else {
        serde_json::from_value(serde)
            .map_err(|err| anyhow!("Could not parse response data. {}", err.to_string()))
    }
}
