use crate::sql_tools;
use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::sync::mpsc;

const CAPACITY: usize = 500;
const MAX_DELTA: f32 = 2.0;

pub async fn moving_average(
    rx: mpsc::Receiver<sql_tools::TSData>,
    pool: &sqlx::Pool<sqlx::Postgres>,
) -> Result<(), Box<dyn Error>> {
    let mut queues: HashMap<String, VecDeque<sql_tools::TSData>> = HashMap::new();
    let mut rolling_sums: HashMap<String, f32> = HashMap::new();

    for received in rx {
        let freq_str = format!("{:.3}", received.frequency);
        let queue = queues
            .entry(freq_str.clone())
            .or_insert_with(|| VecDeque::with_capacity(CAPACITY));
        let rolling_sum = rolling_sums.entry(freq_str.clone()).or_insert(0.0);
        if queue.len() < CAPACITY {
            *rolling_sum += &received.rssi;
            queue.push_back(received);
        } else if queue.len() == CAPACITY {
            if let Some(oldest) = queue.pop_front() {
                *rolling_sum -= oldest.rssi;
            }
            *rolling_sum += received.rssi;
            let average = *rolling_sum as f32 / queue.len() as f32;
            let delta = received.rssi - average;
            //println!("frequency: {}", received.frequency);
            //println!("average: {}, Delta: {}", average, delta);
            if delta >= MAX_DELTA {
                sql_tools::insert_time_series_data(pool, received.clone()).await?;
            }

            queue.push_back(received);
        }
    }

    Ok(())
}
