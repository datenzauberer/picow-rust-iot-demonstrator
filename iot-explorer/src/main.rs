use dotenv::dotenv;
use std::env;

use chrono::NaiveDateTime;
use clap::{Parser, Subcommand};
use iot_db_accessor::{
    add_sensor_data, get_date_with_default, list_last_values_descending_since, list_sensordata,
    to_naivedatetime,
};
use sqlx::{Pool, Sqlite, SqlitePool};
use tokio::time::sleep;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a sensor value to the database with timestamp from now
    Add { value: f64 },
    /// List all Sensor values ascending
    All,
    /// List latest Sensor values descending
    Last {
        /// follow up and list new values, does not exit the program
        #[clap(long, short, action)]
        follow: bool,
        /// output the last NUM rows, instead of teht last 10
        #[clap(long, short = 'n', default_value = "10")]
        rows: u32,
        /// show sensor values since the date in format, example: "2024-01-01 00:00:00"
        #[arg(value_parser = parse_duration)]
        since: Option<NaiveDateTime>,
    },
    /// create some test data
    Testdata {},
}

fn parse_duration(arg: &str) -> Result<NaiveDateTime, std::num::ParseIntError> {
    Ok(to_naivedatetime(arg))
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;
    let cli = Cli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Commands::Add { value } => {
            let _id = add_sensor_data(&pool, chrono::Utc::now().naive_utc(), *value).await;
        }
        Commands::All {} => {
            let recs = list_sensordata(&pool).await;
            for rec in recs.unwrap() {
                println!("{:?}", rec);
            }
        }
        Commands::Last {
            follow,
            since,
            rows,
        } => {
            let mut since_latest = get_date_with_default(since);
            println!("Sensor Values");

            loop {
                let recs = list_last_values_descending_since(&pool, &since_latest, *rows).await;
                let sensor_values = recs.unwrap();
                for rec in sensor_values.iter().rev() {
                    println!("{:?}", rec);
                }

                if *follow {
                    if let Some(latest) = sensor_values.first() {
                        since_latest = latest.timestamp
                    }
                    sleep(std::time::Duration::from_secs(1)).await;
                } else {
                    break;
                }
            }
        }
        Commands::Testdata {} => {
            create_test_data(&pool).await;
        }
    }
    Ok(())
}

#[allow(unused_must_use)]
async fn create_test_data(pool: &Pool<Sqlite>) {
    add_sensor_data(pool, to_naivedatetime("2024-01-01 09:00:00"), 10.00).await;
    add_sensor_data(pool, to_naivedatetime("2024-01-01 09:30:00"), 11.00).await;
    add_sensor_data(pool, to_naivedatetime("2024-01-01 09:59:00"), 12.00).await;
    add_sensor_data(pool, to_naivedatetime("2024-01-01 10:00:00"), 13.00).await;
    add_sensor_data(pool, chrono::Utc::now().naive_utc(), 10.00).await;
}
