use mongodb::{
    options::{ClientOptions, Credential, ServerAddress},
    Database,
};

use super::Secrets;

pub fn database(secrets: &Secrets) -> Database {
    #[cfg(feature = "db_auth")]
    let credential = Credential::builder()
        .username(secrets.db_username().to_owned())
        .password(secrets.db_password().to_owned())
        .build();

    #[cfg(not(feature = "db_auth"))]
    let credential = None;

    let db_url = std::env::var("SLIMEBOT_DB_URL").expect("failed to load db url from environment");

    let options = ClientOptions::builder()
        .app_name("slimebot".to_string())
        .credential(credential)
        .hosts(vec![
            ServerAddress::parse(db_url).expect("db address should be valid")
        ])
        .build();

    mongodb::Client::with_options(options)
        .expect("client options should be valid")
        .database("slimebot")
}
