use anyhow::anyhow;
use serde::{Deserialize, Serialize};
mod github;
mod utils;
mod web;
use cfg_if::cfg_if;
use graphql_client::GraphQLQuery;
use wasm_bindgen::prelude::*;

// we import all types from the generated query
use branch_head_commit_author::Variables as QueryVariables;
use branch_head_commit_author::*;

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
    errors: Vec<GraphqlError>,
    rate_limit_info: RateLimitInfo,
    branch: Branch,
    repo: Repo,
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

async fn run_graphql_private(
    owner: String,
    repo: String,
    branch: String,
    token: String,
) -> anyhow::Result<Data> {
    let github = github::Github::new(token);
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
    let rate_limit = data
        .rate_limit
        .ok_or(anyhow!("No rate_limit on response data"))?;
    let repository = data.repository.ok_or(anyhow!("No repository in data"))?;
    let branch_ref = repository.ref_.ok_or(anyhow!(
        "No branch {} on repository {}",
        &branch,
        repository.name_with_owner
    ))?;
    let head = branch_ref.target.ok_or(anyhow!(
        "No target for branch {} on repository {}",
        &branch,
        repository.name_with_owner
    ))?;

    Ok(Data {
        rate_limit_info: RateLimitInfo {
            cost: rate_limit.cost,
            limit: rate_limit.limit,
            node_count: rate_limit.node_count,
            remaining: rate_limit.remaining,
            used: rate_limit.used,
            reset_at: rate_limit.reset_at,
        },
        repo: Repo {
            name_with_owner: repository.name_with_owner,
            owner: get_user_from_owner(repository.owner)?,
        },
        branch: Branch {
            name: branch_ref.name,
            head: get_commit_info_from_target(head)?,
        },
        errors: response.errors.map_or(vec![], |error_list| {
            error_list
                .into_iter()
                .map(|error| GraphqlError {
                    message: error.message,
                })
                .collect::<Vec<GraphqlError>>()
        }),
    })
}

fn get_commit_info_from_target(
    head: BranchHeadCommitAuthorRepositoryRefTarget,
) -> anyhow::Result<Commit> {
    if let BranchHeadCommitAuthorRepositoryRefTargetOn::Commit(commit) = head.on {
        let github_author = commit
            .author
            .ok_or(anyhow!("No author on commit {}", commit.oid))?;

        let github_committer = commit
            .committer
            .ok_or(anyhow!("No committer on commit {}", commit.oid))?;

        let author = User {
            avatar_url: github_author.avatar_url,
            name: github_author.name,
            handle: github_author.user.map(|user| user.login),
            email: github_author.email,
        };

        let committer = User {
            avatar_url: github_committer.avatar_url,
            name: github_committer.name,
            handle: github_committer.user.map(|user| user.login),
            email: github_committer.email,
        };

        Ok(Commit {
            author,
            committer,
            message: commit.message,
            sha: commit.oid,
        })
    } else {
        Err(anyhow!("ref does not appear to be a commit"))
    }
}

fn get_user_from_owner(owner: BranchHeadCommitAuthorRepositoryOwner) -> anyhow::Result<User> {
    match owner.on {
        BranchHeadCommitAuthorRepositoryOwnerOn::User(user) => Ok(User {
            avatar_url: owner.avatar_url,
            name: user.name,
            email: Option::Some(user.email),
            handle: Option::Some(owner.login),
        }),
        BranchHeadCommitAuthorRepositoryOwnerOn::Organization(orga) => Ok(User {
            avatar_url: owner.avatar_url,
            name: orga.name,
            handle: Option::Some(owner.login),
            email: orga.email,
        }),
    }
}
