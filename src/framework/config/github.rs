use serde::Deserialize;

#[derive(serde::Deserialize, Debug, Clone)]
pub struct GithubConfig {
    #[serde(deserialize_with = "deserialize_repo")]
    pub repository: [String; 2],

    #[cfg(feature = "github_bot")]
    pub bot: BotConfig,
}

#[cfg(feature = "github_bot")]
#[derive(serde::Deserialize, Debug, Clone)]
pub struct BotConfig {}

fn deserialize_repo<'de, D>(deserializer: D) -> Result<[String; 2], D::Error>
where
    D: serde::Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;
    let (user, repo) = string
        .split_once("/")
        .ok_or_else(|| serde::de::Error::custom("not slash-delimited"))?;

    Ok([user.to_owned(), repo.to_owned()])
}
