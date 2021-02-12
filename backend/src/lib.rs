use anyhow::anyhow;
use cfg_if::cfg_if;
use graphql_client::GraphQLQuery;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

// we import all types from the generated query
use branch_head_commit_author::Variables as QueryVariables;
use branch_head_commit_author::*;

mod utils;
mod web;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Scalar types declared in the schema need to be declared in the
// scope that declares the graphql types
type DateTime = String;
type URI = String;
type GitObjectID = String;

#[derive(GraphQLQuery)]
#[graphql(schema_path = "schema.json", query_path = "src/head-query.graphql")]
pub struct BranchHeadCommitAuthor;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    avatar_url: URI,
    handle: Option<String>,
    name: Option<String>,
    email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Repo {
    name_with_owner: String,
    owner: User,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Branch {
    name: String,
    head: Commit,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Commit {
    author: User,
    committer: User,
    sha: String,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphqlError {
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RateLimitInfo {
    cost: i64,
    limit: i64,
    node_count: i64,
    remaining: i64,
    used: i64,
    reset_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    errors: Option<Vec<GraphqlError>>,
    rate_limit_info: Option<RateLimitInfo>,
    branch: Option<Branch>,
    repo: Option<Repo>,
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

#[wasm_bindgen]
pub fn init() {
    init_log();
    crate::utils::set_panic_hook();
}

impl Into<JsValue> for Data {
    fn into(self) -> JsValue {
        JsValue::from_serde(&self).unwrap()
    }
}

#[wasm_bindgen]
pub async fn run_graphql(
    owner: String,
    repo: String,
    branch: String,
    token: String,
) -> Result<Data, js_sys::Error> {
    run_graphql_private(owner, repo, branch, token)
        .await
        .map_err(|err| js_sys::Error::new(&err.to_string()))
}

fn get_rate_limit_info(rate_limit: BranchHeadCommitAuthorRateLimit) -> RateLimitInfo {
    RateLimitInfo {
        cost: rate_limit.cost,
        limit: rate_limit.limit,
        node_count: rate_limit.node_count,
        remaining: rate_limit.remaining,
        used: rate_limit.used,
        reset_at: rate_limit.reset_at,
    }
}

async fn run_graphql_private(
    owner: String,
    repo: String,
    branch: String,
    token: String,
) -> anyhow::Result<Data> {
    let github = web::github::Github::new(token);
    let response = github
        .graphql(
            BranchHeadCommitAuthor,
            QueryVariables {
                branch: branch.clone(),
                owner,
                repo_name: repo,
            },
        )
        .await?;

    let data = response.data.ok_or(anyhow!("No data on response"))?;
    Ok(Data {
        rate_limit_info: data.rate_limit.map(get_rate_limit_info),
        repo: data.repository.as_ref().map(get_repo_info).transpose()?,
        branch: data
            .repository
            .as_ref()
            .and_then(|repo| repo.ref_.as_ref().map(get_branch_info))
            .transpose()?,
        errors: response.errors.map(|error_list| {
            error_list
                .into_iter()
                .map(|error| GraphqlError {
                    message: error.message,
                })
                .collect::<Vec<GraphqlError>>()
        }),
    })
}

fn get_branch_info(branch: &BranchHeadCommitAuthorRepositoryRef) -> anyhow::Result<Branch> {
    let head = branch
        .target
        .as_ref()
        .ok_or(anyhow!("No target for branch"))?;

    Ok(Branch {
        name: branch.name.to_string(),
        head: get_commit_info_from_target(head)?,
    })
}

fn get_repo_info(repo: &BranchHeadCommitAuthorRepository) -> anyhow::Result<Repo> {
    Ok(Repo {
        name_with_owner: repo.name_with_owner.to_string(),
        owner: get_user_from_owner(&repo.owner),
    })
}

fn get_commit_info_from_target(
    head: &BranchHeadCommitAuthorRepositoryRefTarget,
) -> anyhow::Result<Commit> {
    if let BranchHeadCommitAuthorRepositoryRefTargetOn::Commit(commit) = &head.on {
        let github_author = commit
            .author
            .as_ref()
            .ok_or(anyhow!("No author on commit {}", commit.oid))?;

        let github_committer = commit
            .committer
            .as_ref()
            .ok_or(anyhow!("No committer on commit {}", commit.oid))?;

        let author = User {
            avatar_url: github_author.avatar_url.to_string(),
            name: github_author.name.as_ref().map(String::from),
            handle: github_author
                .user
                .as_ref()
                .map(|user| user.login.to_string()),
            email: github_author.email.as_ref().map(String::from),
        };

        let committer = User {
            avatar_url: github_committer.avatar_url.to_string(),
            name: github_committer.name.as_ref().map(String::from),
            handle: github_committer
                .user
                .as_ref()
                .map(|user| user.login.to_string()),
            email: github_committer.email.as_ref().map(String::from),
        };

        Ok(Commit {
            author,
            committer,
            message: commit.message.to_string(),
            sha: commit.oid.to_string(),
        })
    } else {
        Err(anyhow!("ref does not appear to be a commit"))
    }
}

fn get_user_from_owner(owner: &BranchHeadCommitAuthorRepositoryOwner) -> User {
    match &owner.on {
        BranchHeadCommitAuthorRepositoryOwnerOn::User(user) => User {
            avatar_url: owner.avatar_url.to_string(),
            name: user.name.as_ref().map(String::from),
            email: Option::Some(user.email.to_string()),
            handle: Option::Some(owner.login.to_string()),
        },
        BranchHeadCommitAuthorRepositoryOwnerOn::Organization(orga) => User {
            avatar_url: owner.avatar_url.to_string(),
            name: orga.name.as_ref().map(String::from),
            handle: Option::Some(owner.login.to_string()),
            email: orga.email.as_ref().map(String::from),
        },
    }
}
