import React, { useEffect } from "react";
import { CircularProgress, Grid, TextField } from "@material-ui/core";
import * as wasm from "./pkg";

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

function App() {
  const [{ repoName, branch }, setRepositoryInfo] = React.useState<{
    repoName: string;
    branch: string;
  }>({ repoName: "", branch: "" });
  const [{ loading, branchInfo }, setFetchResult] = React.useState<{
    loading: boolean;
    branchInfo?: Branch;
  }>({ loading: false });
  useEffect(() => {
    if (repoName !== "" && branch !== "") {
      setFetchResult({ loading: true });
      wasm
        .run(repoName, branch)
        .then((result: any) => {
          setFetchResult({ loading: false, branchInfo: result });
          console.log(result);
        })
        .catch((error: any) => console.log(error));
    }
  }, [repoName, branch]);

  return (
    <Grid container={true}>
      <Grid item={true} xs={6}>
        <TextField
          id="repoName"
          label="Repository Name"
          onBlur={(e) =>
            setRepositoryInfo({ repoName: e.target.value, branch })
          }
        />
      </Grid>
      <Grid item={true} xs={6}>
        <TextField
          id="branch"
          label="Branch Name"
          onBlur={(e) =>
            setRepositoryInfo({ repoName, branch: e.target.value })
          }
        />
      </Grid>

      <Grid item={true}>
        {loading ? (
          <CircularProgress />
        ) : (
          branchInfo &&
          `Name: ${branchInfo.name}, HEAD: ${branchInfo.commit.sha}, author: ${branchInfo.commit.commit.author.name} <${branchInfo.commit.commit.author.email}>`
        )}
      </Grid>
      <Grid item={true}></Grid>
    </Grid>
  );
}

export default App;
