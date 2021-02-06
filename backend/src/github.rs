use graphql_client::{GraphQLQuery, Response};

const GITHUB: &str = "https://api.github.com/graphql";

pub struct Github {
    token: String,
}

impl Github {
    pub fn new<Token>(token: Token) -> Self
    where
        Token: Into<String>,
    {
        Github {
            token: token.into(),
        }
    }

    pub async fn graphql<Q: GraphQLQuery>(
        &self,
        query: Q,
        variables: Q::Variables,
    ) -> anyhow::Result<Response<Q::ResponseData>> {
        crate::web::gql_call(
            query,
            variables,
            GITHUB.into(),
            [
                ("Authorization".into(), format!("Bearer {}", &self.token)),
                ("Accept".into(), "application/vnd.github.v3+json".into()),
                ("Content-Type".into(), "application/json".into()),
            ]
            .iter()
            .cloned()
            .collect(),
        )
        .await
    }
}
