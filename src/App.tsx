import React, { useEffect } from "react";
import { CircularProgress, Grid, TextField } from "@material-ui/core";
import { run } from "./pkg";

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

function App() {
  const [{ repoName, branch }, setRepositoryInfo] = React.useState<{
    repoName: string;
    branch: string;
  }>({ repoName: "", branch: "" });
  const [{ loading, branchInfo, error }, setFetchResult] = React.useState<{
    loading: boolean;
    branchInfo?: Branch;
    error?: GithubError;
  }>({ loading: false });
  useEffect(() => {
    if (repoName !== "" && branch !== "") {
      setFetchResult({ loading: true });
      run(repoName, branch)
        .then((result: GithubError | Branch) => {
          console.log(result);
          if ("message" in result) {
            setFetchResult({ loading: false, error: result });
          } else {
            setFetchResult({ loading: false, branchInfo: result });
          }
        })
        .catch((error: any) => console.error(error));
    }
  }, [repoName, branch]);

  return (
    <Grid container={true} spacing={6}>
      <Grid item={true} xs={6}>
        <InputField
          id="repoName"
          label="Repository Name"
          setValue={(val) => setRepositoryInfo({ branch, repoName: val })}
        />
      </Grid>
      <Grid item={true} xs={6}>
        <InputField
          id="branch"
          label="Branch Name"
          setValue={(val) => setRepositoryInfo({ repoName, branch: val })}
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
