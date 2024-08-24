use mongodb::{
    options::{ClientOptions, Credential, ServerAddress},
    Database,
};

use super::{config::DbConfig, Secrets};

pub fn database(config: &DbConfig, secrets: &Secrets) -> Database {
    let credential = Credential::builder()
        .username(secrets.db_username().to_owned())
        .password(secrets.db_password().to_owned())
        .build();

    let options = ClientOptions::builder()
        .app_name("slimebot".to_string())
        .credential(credential)
        .hosts(vec![
            ServerAddress::parse(config.url()).expect("db address should be valid")
        ])
        .build();

    mongodb::Client::with_options(options)
        .expect("client options should be valid")
        .database("slimebot")
}
