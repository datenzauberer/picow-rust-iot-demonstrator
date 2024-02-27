#![warn(rust_2018_idioms)]

use dotenvy::dotenv;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;

use tracing::{info, warn};

use std::env;
use std::sync::Arc;

use iot_db_accessor::add_sensor_data;
use sqlx::SqlitePool;

const BUFFER_SIZE: usize = 1024;
const SENSOR_DATA_SIZE: usize = 4;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    tracing_init();

    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;
    let pool = Arc::new(pool);

    // if no host is configured, wildcard address is used
    let serverurl = &env::var("IOT_DATA_BRIDGE_URL")?;
    let serverurl = if serverurl.starts_with(':') {
        format!("0.0.0.0{}", serverurl)
    } else {
        serverurl.to_string()
    };

    let listener = TcpListener::bind(&serverurl).await?;
    println!("IoT Data Bridge is listening on: {}", serverurl);

    loop {
        let pool = Arc::clone(&pool);
        // Asynchronously wait for an inbound socket.
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = vec![0; BUFFER_SIZE];

            loop {
                let n = socket.read(&mut buf).await;

                match n {
                    Ok(SENSOR_DATA_SIZE) => {
                        // Sensorvalue contains 4 byte
                        process_sensor_data(&pool, &buf).await;
                    }
                    Ok(0) => return, // Client disconnected
                    Err(_) => {
                        warn!("Failed to read data from socket");
                    }
                    Ok(n) => {
                        let response = String::from_utf8_lossy(&buf[..n]);
                        warn!(
                            "Invalid sensor data: could not process {} (len: {})",
                            response, n
                        );
                    }
                };
            }
        });
    }
}

fn tracing_init() {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    let filter = EnvFilter::from_default_env();
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();
}

async fn process_sensor_data(pool: &SqlitePool, data: &[u8]) {
    let temp = f32::from_be_bytes([data[0], data[1], data[2], data[3]]);
    // PicoW AnalogDigitalConverter only supports f32, but the backend supports f64
    let value = temp.into();
    info!("received value from sensor: {}", value);
    let result = add_sensor_data(pool, chrono::Utc::now().naive_utc(), value).await;
    if let Err(e) = result {
        warn!("An error occurred: {:?}", e);
    }
}
