import React, { useEffect } from "react";
import { CircularProgress, Grid, TextField } from "@material-ui/core";
import { run_graphql } from "./pkg";

interface Signature {
  name: string;
  email: string;
}

interface CommitDetails {
  author: Signature;
  commiter: Signature;
}
interface Commit {
  sha: string;
  commit: CommitDetails;
}

interface Branch {
  name: string;
  commit: Commit;
}

interface GithubError {
  message: string;
  documentation_url: string;
}

const InputField = ({
  setValue,
  ...props
}: {
  id: string;
  label: string;
  setValue: (val: string) => void;
  defaultValue?: string;
}) => (
  <TextField
    onKeyPress={(e) => {
      if (e.key === "Enter") {
        setValue((e.target as any).value);
      }
    }}
    onBlur={(e) => setValue(e.target.value)}
    {...props}
  />
);

interface User {
  avatar_url: string;
  email?: string;
  handle?: string;
  name?: string;
}

interface BackendData {
  Data: {
    branch: {
      name: string;
      head: {
        author: User;
        committer: User;
        sha: string;
      };
    };
    rate_limit_info: {
      cost: number;
      limit: number;
      node_count: number;
      remaining: number;
      reset_at: string;
      used: number;
    };
    errors: [{ message: string }];
    repo: {
      name_with_owner: string;
      owner: User;
    };
  };
}

interface BackendError {
  Error: string;
}

function App() {
  const [{ repo, owner, branch }, setRepositoryInfo] = React.useState<{
    owner: string;
    branch: string;
    repo: string;
  }>({ owner: "adimit", branch: "master", repo: "config" });
  const [apiKey, setApiKey] = React.useState<string>(
    localStorage.getItem("github.token") ?? ""
  );

  const [{ loading, branchInfo, error }, setFetchResult] = React.useState<{
    loading: boolean;
    branchInfo?: Branch;
    error?: GithubError;
  }>({ loading: false });
  useEffect(() => {
    if (repo !== "" && branch !== "" && owner !== "" && apiKey !== "") {
      run_graphql(owner, repo, branch, apiKey)
        .then((result: BackendData | BackendError) => {
          setFetchResult({ loading: false });
          console.log("graphql ", result);
        })
        .catch((error: any) => console.error(error));
      setFetchResult({ loading: true });
    }
  }, [repo, owner, branch, apiKey]);

  useEffect(() => {
    apiKey !== "" && localStorage.setItem("github.token", apiKey);
  }, [apiKey]);

  return (
    <Grid container={true} spacing={6}>
      <Grid item={true} xs={3}>
        <InputField
          id="owner"
          label="Owner"
          setValue={(val) => setRepositoryInfo({ branch, owner: val, repo })}
        />
      </Grid>
      <Grid item={true} xs={3}>
        <InputField
          id="repo"
          label="Repository Name"
          setValue={(val) => setRepositoryInfo({ repo: val, owner, branch })}
        />
      </Grid>
      <Grid item={true} xs={3}>
        <InputField
          id="branch"
          label="Branch Name"
          setValue={(val) => setRepositoryInfo({ repo, owner, branch: val })}
        />
      </Grid>
      <Grid item={true} xs={3}>
        <InputField
          id="token"
          label="Api Token"
          setValue={(val) => setApiKey(val)}
          defaultValue={apiKey}
        />
      </Grid>

      <Grid item={true}>
        {loading && <CircularProgress />}
        {branchInfo &&
          `Name: ${branchInfo.name}, HEAD: ${branchInfo.commit.sha}, author: ${branchInfo.commit.commit.author.name} <${branchInfo.commit.commit.author.email}>`}
        {error && <a href={error.documentation_url}>{error.message}</a>}
      </Grid>
      <Grid item={true}></Grid>
    </Grid>
  );
}

export default App;
