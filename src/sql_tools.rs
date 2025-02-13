use crate::Config;
use sqlx::Error;
use sqlx::{FromRow, Pool, Postgres};
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug)]
pub struct TSData {
    pub time: Duration,
    pub id: Uuid,
    pub rssi: f64,
    pub frequency: f32,
}

impl TSData {
    pub fn new(time: Duration, id: Uuid, rssi: f64, frequency: f32) -> Self {
        TSData {
            time,
            id,
            rssi,
            frequency,
        }
    }
}

#[derive(FromRow)]
struct SQLDevice {
    name: String,
    geohash: String,
}

pub async fn insert_time_series_data(pool: &Pool<Postgres>, data: TSData) -> Result<(), Error> {
    sqlx::query(
        "INSERT INTO sensor_data (
                time,
                id,
                rssi,
                frequency
                )
                VALUES(to_timestamp($1),$2,$3,$4);",
    )
    .bind(data.time)
    .bind(data.id)
    .bind(data.rssi)
    .bind(data.frequency)
    .execute(pool)
    .await?;
    Ok(())
}

async fn update_database_field(
    pool: &Pool<Postgres>,
    (id, field, value): (&Uuid, &str, &str),
) -> Result<(), Error> {
    let allowed_columns = ["name", "geohash"];
    if !allowed_columns.contains(&field) {
        return Err(Error::ColumnNotFound(field.into()));
    }
    let query_str = format!("UPDATE sensor_metadata SET {} = $2 WHERE id = $1", field);

    sqlx::query(&query_str)
        .bind(id)
        .bind(value)
        .execute(pool)
        .await?;
    eprintln!("updated {}:{}", field, value);
    Ok(())
}
pub async fn initialize_device(
    pool: &Pool<Postgres>,
    config: &mut Config,
) -> Result<(), sqlx::Error> {
    if config.id == None {
        let id: Uuid = sqlx::query_scalar(
            "INSERT INTO sensor_metadata (
            name,
            geohash)
        VALUES ($1, $2)
        RETURNING id;
    ",
        )
        .bind("test_name")
        .bind("9r10g4ck1ft")
        .fetch_one(pool)
        .await?;
        config.id = Some(id);
        config.save("config.toml").expect("Error saving config");
        println!("{}", id);
    } else {
        let device: SQLDevice =
            sqlx::query_as("SELECT id, name, geohash FROM sensor_metadata WHERE id = $1")
                .bind(config.id)
                .fetch_optional(pool)
                .await?
                .expect("Should have a value from the database");
        if device.name != config.name {
            update_database_field(&pool, (&config.id.unwrap(), "name", &config.name)).await?;
        } else if device.geohash != config.geohash {
            update_database_field(&pool, (&config.id.unwrap(), "geohash", &config.geohash)).await?;
        }
    }
    Ok(())
}
