use serde::{Deserialize, Serialize};
mod utils;
use cfg_if::cfg_if;
use graphql_client::GraphQLQuery;

// Github Schema DateTime type is just a string
type DateTime = String;
type URI = String;

#[derive(GraphQLQuery)]
#[graphql(schema_path = "schema.json", query_path = "src/head-query.graphql")]
pub struct BranchHeadCommitAuthor;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

#[derive(Debug, Serialize, Deserialize)]
pub struct Branch {
    pub name: String,
    pub commit: Commit,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Commit {
    pub sha: String,
    pub commit: CommitDetails,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommitDetails {
    pub author: Signature,
    pub committer: Signature,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Signature {
    pub name: String,
    pub email: String,
}

cfg_if! {
    if #[cfg(feature = "console_log")] {
        fn init_log() {
            console_log::init_with_level(log::Level::Trace).expect("error initializing console logging");
        }
    } else {
        fn init_log() {}
    }
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    init_log();

    log::info!("It works!");
    log::trace!("Trace");
    log::debug!("debug");
    log::error!("error");
    log::warn!("warn");
}

#[wasm_bindgen]
pub async fn run(repo: String, branch: String) -> Result<JsValue, JsValue> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let url = format!("https://api.github.com/repos/{}/branches/{}", repo, branch);
    let request = Request::new_with_str_and_init(&url, &opts)?;

    request
        .headers()
        .set("Accept", "application/vnd.github.v3+json")?;

    let window = web_sys::window().expect("Should have window");
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

    let resp: Response = resp_value.dyn_into()?;

    let json = JsFuture::from(resp.json()?).await?;

    Ok(JsValue::from(json))
}

#[derive(Debug, Serialize)]
struct GithubHeaders {
    pub authorization: String,
    pub accept: String,
}

#[wasm_bindgen]
pub async fn run_graphql(
    owner: String,
    repo: String,
    branch: String,
    token: String,
) -> Result<JsValue, JsValue> {
    let query = BranchHeadCommitAuthor::build_query(branch_head_commit_author::Variables {
        branch,
        owner,
        repo_name: repo,
    });
    let mut opts = RequestInit::new();
    let headers = JsValue::from_serde(&GithubHeaders {
        authorization: format!("Bearer {}", token),
        accept: "application/vnd.github.v3+json".into(),
    })
    .map_err(|err| err.to_string())?;
    opts.headers(&headers);
    opts.method("POST");
    opts.mode(RequestMode::Cors);
    let body = serde_json::to_string(&query).map_err(|_| JsValue::NULL)?;
    opts.body(Option::Some(&JsValue::from_str(&body)));

    let url = "https://api.github.com/graphql";

    let window = web_sys::window().expect("Should have window here");
    let request = Request::new_with_str_and_init(&url, &opts)?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

    let response: Response = resp_value.dyn_into()?;

    let json = JsFuture::from(response.json()?).await?;

    Ok(JsValue::from(json))
}
