pub struct Client {
    http: reqwest::Client,
    db: mongodb::Collection<super::Series>,
}
