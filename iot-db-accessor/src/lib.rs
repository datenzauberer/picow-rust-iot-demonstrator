use sqlx::SqlitePool;
// use sqlx::types::chrono::NaiveDateTime;
use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct SensorData {
    pub id: i64,
    pub timestamp: chrono::NaiveDateTime,
    pub value: f64,
}

pub async fn add_sensor_data(
    pool: &SqlitePool,
    timestamp: NaiveDateTime,
    value: f64,
) -> anyhow::Result<i64> {
    let mut conn = pool.acquire().await?;

    // Insert the task, then obtain the ID of this row
    let id = sqlx::query!(
        r#"
    INSERT INTO sensor_values (timestamp, value)
    VALUES ($1, $2)
        "#,
        timestamp,
        value
    )
    .execute(&mut *conn)
    .await?
    .last_insert_rowid();

    Ok(id)
}

pub async fn list_sensordata(pool: &SqlitePool) -> anyhow::Result<Vec<SensorData>> {
    let recs = sqlx::query_as_unchecked!(
        SensorData,
        r#"
    SELECT id, timestamp, value
    FROM sensor_values
    ORDER BY timestamp
    "#
    )
    .fetch_all(pool)
    .await?;
    Ok(recs)
}

pub async fn list_last_values_descending_since(
    pool: &SqlitePool,
    since: &NaiveDateTime,
    rows: u32,
) -> anyhow::Result<Vec<SensorData>> {
    let recs = sqlx::query_as_unchecked!(
        SensorData,
        r#"
    SELECT id, timestamp, value
    FROM sensor_values
    WHERE timestamp > $2
    ORDER BY timestamp DESC
    LIMIT $1
    "#,
        rows,
        since
    )
    .fetch_all(pool)
    .await?;
    Ok(recs)
}

pub fn get_date_with_default(date: &Option<NaiveDateTime>) -> NaiveDateTime {
    date.unwrap_or_else(|| to_naivedatetime("1970-01-01 00:00:00"))
}

pub fn to_naivedatetime(input: &str) -> NaiveDateTime {
    let parse_from_str = NaiveDateTime::parse_from_str;
    const DT_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
    parse_from_str(input, DT_FORMAT).unwrap()
}
