use mongodb::{
    options::{ClientOptions, Credential, ServerAddress},
    Database,
};

use crate::config::DbConfig;

pub async fn connect(config: &DbConfig) -> Database {
    let credential = Credential::builder()
        .username(config.username().to_string())
        .password(config.password().to_string())
        .build();

    let options = ClientOptions::builder()
        .app_name("slimebot".to_string())
        .credential(credential)
        .hosts(vec![ServerAddress::parse(config.url()).unwrap()])
        .build();

    mongodb::Client::with_options(options)
        .unwrap()
        .database("slimebot")
}
