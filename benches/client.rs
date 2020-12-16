use chrono::{DateTime, Utc};
use influxdb::Error;
use influxdb::InfluxDbWriteable;
use influxdb::{Client, Query};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::Semaphore;

#[derive(InfluxDbWriteable, Clone)]
struct WeatherReading {
    time: DateTime<Utc>,
    humidity: i32,
    #[influxdb(tag)]
    wind_direction: String,
}

#[tokio::main]
async fn main() {
    let db_name = "bench";
    let url = "http://localhost:8086";
    let number_of_total_requests = 20000;
    let concurrent_requests = 1000;

    let client = Client::new(url, db_name);
    let concurrency_limit = Arc::new(Semaphore::new(concurrent_requests));

    prepare_influxdb(&client, db_name).await;
    let measurements = generate_measurements(number_of_total_requests);
    let (tx, mut rx) = unbounded_channel::<Result<String, Error>>();

    let start = Instant::now();
    for m in measurements {
        let permit = concurrency_limit.clone().acquire_owned().await;
        let client_task = client.clone();
        let tx_task = tx.clone();
        tokio::spawn(async move {
            let res = client_task.query(&m.into_query("weather")).await;
            let _ = tx_task.send(res);
            drop(permit);
        });
    }
    drop(tx);

    let mut successful_count = 0;
    let mut error_count = 0;
    while let Some(res) = rx.recv().await {
        if res.is_err() {
            error_count += 1;
        } else {
            successful_count += 1;
        }
    }

    let end = Instant::now();

    println!(
        "Throughput: {:.1} request/s",
        1000000.0 * successful_count as f64 / (end - start).as_micros() as f64
    );
    println!(
        "{} successful requests, {} errors",
        successful_count, error_count
    );
}

async fn prepare_influxdb(client: &Client, db_name: &str) {
    let create_db_stmt = format!("CREATE DATABASE {}", db_name);
    client
        .query(&Query::raw_read_query(create_db_stmt))
        .await
        .expect("failed to create database");
}

fn generate_measurements(n: u64) -> Vec<WeatherReading> {
    (0..n)
        .collect::<Vec<u64>>()
        .iter_mut()
        .map(|_| WeatherReading {
            time: Utc::now(),
            humidity: 30,
            wind_direction: String::from("north"),
        })
        .collect()
}
