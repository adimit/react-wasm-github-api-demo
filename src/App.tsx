import React, { useEffect } from "react";
import {
  Avatar,
  Card,
  CardContent,
  CircularProgress,
  Container,
  FormLabel,
  Grid,
  LinearProgress,
  List,
  ListItem,
  ListItemAvatar,
  ListItemText,
  makeStyles,
  Paper,
  TextField,
  Theme,
  Typography,
} from "@material-ui/core";
import { run_graphql } from "./pkg";

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
        message: string;
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

const RateLimitInfo: React.FC<BackendData["Data"]["rate_limit_info"]> = ({
  used,
  cost,
  limit,
  node_count,
  reset_at,
}) => {
  const diff = Math.floor(
    (new Date(reset_at).getTime() - new Date().getTime()) / 1000 / 60
  );
  const classes = useStyles();
  return (
    <Card className={classes.dataCard}>
      <LinearProgress value={(used / limit) * 100} variant="determinate" />
      <CardContent>
        <Typography variant="h5">Rate Limit</Typography>
        <Typography color="textSecondary">
          {diff >= 0 ? `Resets in ${diff} minutes` : "Already reset."}
        </Typography>
        <Typography>{`Usage: ${used}/${limit}`}</Typography>
        <Typography variant="h6">Last Request</Typography>
        <Typography>{`Cost: ${cost}, nodes: ${node_count}`}</Typography>
      </CardContent>
    </Card>
  );
};

const RepositoryInfo: React.FC<BackendData["Data"]["repo"]> = ({
  name_with_owner,
  owner,
}) => {
  const classes = useStyles();
  return (
    <Card className={classes.dataCard}>
      <CardContent>
        <Typography variant="h5">Repository</Typography>
        <Typography color="textSecondary">{name_with_owner}</Typography>
        <List>
          <ListItem>
            <ListItemAvatar>
              <Avatar alt={owner.name} src={owner.avatar_url} />
            </ListItemAvatar>
            <ListItemText primary="Owner" secondary={owner.name} />
          </ListItem>
        </List>
      </CardContent>
    </Card>
  );
};

const BranchInfo: React.FC<BackendData["Data"]["branch"]> = ({
  name,
  head: { sha, message, author, committer },
}) => {
  const classes = useStyles();
  return (
    <Card className={classes.dataCard}>
      <CardContent>
        <Typography variant="h5">Branch</Typography>
        <Typography color="textSecondary">{name}</Typography>
        <Typography variant="h6">Head</Typography>
        <Typography color="textSecondary">{sha}</Typography>
        <Typography>{message}</Typography>
        <List>
          <ListItem>
            <ListItemAvatar>
              <Avatar alt={author.name} src={author.avatar_url} />
            </ListItemAvatar>
            <ListItemText primary="Author" secondary={author.name} />
          </ListItem>
          <ListItem>
            <ListItemAvatar>
              <Avatar alt={committer.name} src={committer.avatar_url} />
            </ListItemAvatar>
            <ListItemText primary="Committer" secondary={committer.name} />
          </ListItem>
        </List>
      </CardContent>
    </Card>
  );
};

const RenderData: React.FC<BackendData> = ({
  Data: { repo, rate_limit_info, branch },
}) => {
  return (
    <>
      <BranchInfo {...branch} />
      <RepositoryInfo {...repo} />
      <RateLimitInfo {...rate_limit_info} />
    </>
  );
};

const RenderError: React.FC<BackendError> = ({ Error: message }) => (
  <FormLabel error={true}>{message}</FormLabel>
);

const RenderResult: React.FC<BackendData | BackendError> = (props) => {
  if ("Error" in props) {
    return <RenderError {...props} />;
  } else {
    return <RenderData {...props} />;
  }
};

const useStyles = makeStyles((theme: Theme) => ({
  container: {
    marginTop: theme.spacing(4),
  },
  dataCard: {
    margin: theme.spacing(2),
  },
}));

function App() {
  const [{ repo, owner, branch }, setRepositoryInfo] = React.useState<{
    owner: string;
    branch: string;
    repo: string;
  }>({ owner: "", branch: "", repo: "" });
  const [apiKey, setApiKey] = React.useState<string>(
    localStorage.getItem("github.token") ?? ""
  );

  const [{ loading, data }, setFetchResult] = React.useState<{
    loading: boolean;
    data?: BackendData | BackendError;
  }>({ loading: false });
  useEffect(() => {
    if (repo !== "" && branch !== "" && owner !== "" && apiKey !== "") {
      run_graphql(owner, repo, branch, apiKey)
        .then((result: BackendData | BackendError) => {
          setFetchResult({ loading: false, data: result });
        })
        .catch((error: any) => console.error(error));
      setFetchResult({ loading: true });
    }
  }, [repo, owner, branch, apiKey]);

  useEffect(() => {
    apiKey !== "" && localStorage.setItem("github.token", apiKey);
  }, [apiKey]);

  const classes = useStyles();

  return (
    <Container maxWidth="sm" className={classes.container}>
      <Card>
        <CardContent>
          <Grid container={true} justify="space-around">
            <InputField
              id="owner"
              label="Owner"
              setValue={(val) =>
                setRepositoryInfo({ branch, owner: val, repo })
              }
            />
            <InputField
              id="repo"
              label="Repository Name"
              setValue={(val) =>
                setRepositoryInfo({ repo: val, owner, branch })
              }
            />
            <InputField
              id="branch"
              label="Branch Name"
              setValue={(val) =>
                setRepositoryInfo({ repo, owner, branch: val })
              }
            />
            <InputField
              id="token"
              label="Api Token"
              setValue={(val) => setApiKey(val)}
              defaultValue={apiKey}
            />
          </Grid>
        </CardContent>
      </Card>

      <Paper>
        <Grid container={true}>
          {loading && <CircularProgress />}
          {data && <RenderResult {...data} />}
        </Grid>
      </Paper>
    </Container>
  );
}

export default App;
