mod dns;
mod routes;

use qso_core::config::Config;
use rocket::{Build, Rocket};

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let config = Config::load_or_default("config.toml");
    let rocket = build_rocket(config);
    rocket.launch().await?;
    Ok(())
}

fn build_rocket(config: Config) -> Rocket<Build> {
    use rocket::figment::Figment;

    let figment = Figment::from(rocket::Config::default())
        .merge(("address", config.server.bind_address.clone()))
        .merge(("port", config.server.port));

    rocket::custom(figment)
        .manage(config)
        .mount("/", routes::routes())
}
