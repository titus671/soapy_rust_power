use config::Config;
use sqlx::postgres::PgPoolOptions;
use std::sync::mpsc;
use std::thread;
mod config;
mod math_tools;
mod sdr_tools;
mod sql_tools;

//#[tokio::main]
async fn run(mut config: Config) -> Result<(), sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.postgres.connection_url)
        .await?;

    let (tx, rx) = mpsc::sync_channel(100);

    sql_tools::initialize_device(&pool, &mut config).await?;

    thread::spawn(move || {
        //sdr_tools::get_signal(&config, tx);
        sdr_tools::output_fft(&config, tx);
    });

    math_tools::moving_average(rx, &pool).await.unwrap();

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let mut config = Config::load("config.toml").expect("Error loading config");
    //sdr_tools::output_raw_iq(&config);
    run(config).await?;
    Ok(())
}
