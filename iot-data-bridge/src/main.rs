//! A "hello world" echo server with Tokio
//!
//! This server will create a TCP listener, accept connections in a loop, and
//! write back everything that's read off of each TCP connection.
//!
//! Because the Tokio runtime uses a thread pool, each TCP connection is
//! processed concurrently with all other TCP connections across multiple
//! threads.
//!
//! To see this server in action, you can run this in one terminal:
//!
//!     cargo run --example echo
//!
//! and in another terminal you can run:
//!
//!     cargo run --example connect 127.0.0.1:6142
//!
//! Each line you type in to the `connect` terminal should be echo'd back to
//! you! If you open up multiple terminals running the `connect` example you
//! should be able to see them all make progress simultaneously.

#![warn(rust_2018_idioms)]

use dotenv::dotenv;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;

use std::env;
use std::error::Error;
use std::sync::Arc;

use iot_db_accessor::add_sensor_data;
use sqlx::SqlitePool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
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

        // And this is where much of the magic of this server happens. We
        // crucially want all clients to make progress concurrently, rather than
        // blocking one on completion of another. To achieve this we use the
        // `tokio::spawn` function to execute the work in the background.
        //
        // Essentially here we're executing a new task to run concurrently,
        // which will allow all of our clients to be processed concurrently.

        tokio::spawn(async move {
            let mut buf = vec![0; 1024];

            // In a loop, read data from the socket and write the data back.
            loop {
                let n = socket
                    .read(&mut buf)
                    .await
                    .expect("failed to read data from socket");

                match n {
                    0 => return, // When the client is disconnected return
                    4 => {
                        // Sensorvalue
                        let temp = f32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
                        let _id = add_sensor_value(&pool, temp).await;
                    }
                    _ => {
                        let response = String::from_utf8_lossy(&buf[..n]);
                        println!(
                            "Invalid sensor data: could not process {} (len: {})",
                            response, n
                        );
                    }
                };
            }
        });
    }
}

async fn add_sensor_value(pool: &SqlitePool, value: f32) -> Result<(), Box<dyn Error>> {
    println!("received value from sensor: {}", value);
    let value = value.into();
    add_sensor_data(pool, chrono::Utc::now().naive_utc(), value).await?;
    Ok(())
}
