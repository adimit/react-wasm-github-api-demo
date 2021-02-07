# Rust 🦀 + Create React App ⚛️ + Typescript

This is a demo application that fetches generic information about a repository using the Github Graphql API. This repository is meant to serve as a springboard for creating new applications that use React and Rust/WASM.

A hosted version is [available here](https://aleks.bg/rust-cra). You will need a Github API token with read permissions. Get one [here](https://github.com/settings/tokens).

The application requests information from the Github Graphql API using the Browser's fetch API through [web-sys](https://crates.io/crates/web-sys) with Rust's [graphql-client](https://crates.io/crates/graphql-client). The results are shown in a simple [React app](https://reactjs.org/docs/create-a-new-react-app.html).

## How to get Started

You will need [`yarn`](https://yarnpkg.com/), [Rust](https://www.rust-lang.org/tools/install) and [`wasm-pack`](https://rustwasm.github.io/wasm-pack/). There are detailed instructions on how to set up your computer for Rust & WASM development in the [Rust & Webassembly book](https://rustwasm.github.io/docs/book/).

Check out or fork this repository

```
git clone git@github.com:adimit/react-wasm-github-api-demo.git
cd react-wasm-github-api-demo
```

then run install the JS dependencies and run the test server:

```
yarn install
yarn start
```

The application should now be reachable from `http://localhost:3000`.

Whenever you edit a TypeScript *or* Rust file, or edit your graphql schema, queries or `Cargo.toml`, the server should automatically pick up the changes and reload.

After changes to the Rust side of things, you may need to reload the page in the browser manually.

## Downloading the Graphql Schema

To re-fetch the Github Graphql schema, use the `yarn` target `download-schema`. Don't forget to set the env variable `GITHUB_API_TOKEN` first.

```
env GITHUB_API_TOKEN="xxx" yarn download-schema
```

# About the Code

## Graphql Web Client

The [Graphql Web Client](https://github.com/graphql-rust/graphql-client/blob/master/graphql_client/src/web.rs) is currently [incompatible](https://github.com/graphql-rust/graphql-client/issues/331) with the stabilized futures API. As a workaround, this repository contains an almost verbatim copy of the graphql web client.

You can re-use the client in `web.rs` for your own application, or wait until the necessary [PR is merged](https://github.com/graphql-rust/graphql-client/pull/327) upstream.

## Plumbing

The symbolic link `src/pkg` links the Rust WASM JS output into the JS application. To import any function exported in `lib.rs` with the appropriate `wasm_bindgen` signature, just use

```
import { yourFunction } from `./pkg`;
```

Take care to adjust the relative path to `pkg`. This setup is a bit awkward, but allows for seamless recompilation of the application whenever Rust components change.
