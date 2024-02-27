use anyhow::Result;
use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::SqlitePool;

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
) -> Result<i64> {
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

pub async fn list_sensordata(pool: &SqlitePool) -> Result<Vec<SensorData>> {
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
) -> Result<Vec<SensorData>> {
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

#[cfg(test)]
// https://docs.rs/sqlx/latest/sqlx/attr.test.html#automatic-test-database-management-requires-migrate-feature
mod test {
    use sqlx::SqlitePool;

    use crate::{add_sensor_data, list_sensordata, to_naivedatetime};

    #[sqlx::test]
    async fn test_add_and_list(pool: SqlitePool) -> sqlx::Result<()> {
        assert_eq!(list_sensordata(&pool).await.unwrap().len(), 0);
        let id = add_sensor_data(&pool, to_naivedatetime("2024-01-01 09:00:00"), 10.00).await;
        assert_eq!(id.unwrap(), 1);
        let entries = list_sensordata(&pool).await.unwrap();
        assert_eq!(entries.len(), 1);
        let sensor_data = &entries[0];
        assert_eq!(sensor_data.id, 1);
        assert_eq!(
            sensor_data.timestamp,
            to_naivedatetime("2024-01-01 09:00:00")
        );
        assert_eq!(sensor_data.value, 10.);

        Ok(())
    }
}
