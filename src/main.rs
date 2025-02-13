use config::Config;
use sqlx::postgres::PgPoolOptions;
use std::sync::mpsc;
use std::thread;
mod config;
mod sdr_tools;
mod sql_tools;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let mut config = Config::load("config.toml").expect("Error loading config");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.postgres.connection_url)
        .await?;

    let (tx, rx) = mpsc::channel();

    sql_tools::initialize_device(&pool, &mut config).await?;

    thread::spawn(move || {
        sdr_tools::get_signal(&config, tx);
    });
    for received in rx {
        println!("Got: {:?}", received);
    }
    Ok(())
}
