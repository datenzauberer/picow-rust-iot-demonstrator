//! Sends data in a triangular pattern (from 10 to 15)
//! to the TCP server address configured in the environment variable
//! IOT_DATA_BRIDGE_URL.
//!

#![warn(rust_2018_idioms)]

use dotenv::dotenv;

use std::env;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time::sleep;

use std::error::Error;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let serverurl = init_server_url();

    let up_and_down = UpAndDown::new(10, 15);
    for value in up_and_down {
        create_value(&serverurl, value).await
    }
    Ok(())
}

// Function to initialize server URL from environment variable.
fn init_server_url() -> String {
    let mut serverurl =
        env::var("IOT_DATA_BRIDGE_URL").expect("env variable IOT_DATA_BRIDGE_URL not set");
    if serverurl.starts_with(':') {
        serverurl = format!("127.0.0.1{}", serverurl);
    }
    serverurl
}

// Refactored create_value function to take server URL as a parameter.
async fn create_value(serverurl: &str, value: i32) {
    sleep(Duration::from_secs(1)).await;
    let value = value as f32;

    // Attempt to connect to the server.
    match TcpStream::connect(serverurl).await {
        Ok(mut stream) => {
            // SMPRIO: Tracing
            println!("Temp: {} degrees", value);

            // Convert and send to server
            let msg = &value.to_be_bytes();
            match stream.write_all(msg).await {
                // SMPRIO: Tracing
                Ok(_) => println!("wrote to stream; len:{}", msg.len()),
                Err(e) => eprintln!("failed to write to stream; error: {}", e),
            }
        }
        Err(_) => {
            eprintln!("connection error to {}", serverurl);
        }
    }
}

struct UpAndDown {
    current: i32,
    max: i32,
    min: i32,
    step: i32,
}

impl UpAndDown {
    fn new(min: i32, max: i32) -> UpAndDown {
        UpAndDown {
            current: min,
            max,
            min,
            step: 1,
        }
    }
}

impl Iterator for UpAndDown {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.current;
        if result == self.max {
            self.step = -1;
        } else if result == self.min {
            self.step = 1;
        }

        if self.step == -1 && self.current == self.min {
            return None;
        }

        self.current += self.step;
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascending_and_descending() {
        let mut iterator = UpAndDown::new(10, 12);
        assert_eq!(iterator.next(), Some(10));
        assert_eq!(iterator.next(), Some(11));
        assert_eq!(iterator.next(), Some(12));
        assert_eq!(iterator.next(), Some(11));
        assert_eq!(iterator.next(), Some(10));
        assert_eq!(iterator.next(), Some(11));
    }
}
