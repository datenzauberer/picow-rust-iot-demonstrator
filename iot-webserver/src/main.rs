use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use dotenv::dotenv;
use iot_db_accessor::{
    add_sensor_data, get_date_with_default, list_last_values_descending_since, SensorData,
};
use serde::Deserialize;
use sqlx::types::chrono::{self};
use sqlx::SqlitePool;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    // build our application with a route
    let app = Router::new()
        .route("/", get(index))
        .nest(
            "/api",
            Router::new()
                .route("/sensor_values", get(list_sensordata))
                .route("/sensor_values_since", get(list_sensordata_since))
                .route("/add_sensor_value", post(add_sensor_value)),
        )
        .with_state(pool);

    // start the server, listening on confiured port WebServer IpAdress
    let serverurl = &env::var("IOT_WEBSERVER_URL")?;
    let listener = tokio::net::TcpListener::bind(serverurl).await.unwrap();
    println!("URL to IoT Dashboart: http://{}", serverurl);
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

pub struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

async fn index() -> axum::response::Html<String> {
    // use getData (for productive Data)
    let html = std::include_str!("../assets/index.html").replace("= getTestData", "= getData");
    axum::response::Html(html)
}

async fn add_sensor_value(State(pool): State<SqlitePool>, value: String) -> Result<(), AppError> {
    let value = value.trim().to_string().parse::<f64>()?;
    add_sensor_data(&pool, chrono::Utc::now().naive_utc(), value).await?;
    Ok(())
}

pub async fn list_sensordata(
    State(pool): State<SqlitePool>,
) -> Result<axum::Json<Vec<SensorData>>, AppError> {
    iot_db_accessor::list_sensordata(&pool)
        .await
        .map(Json::from)
        .map_err(AppError::from)
}

#[derive(Debug, Deserialize)]
struct ParamsSensordataSince {
    since: Option<chrono::NaiveDateTime>,
    rows: Option<u32>,
}

async fn list_sensordata_since(
    queryparam: Query<ParamsSensordataSince>,
    State(pool): State<SqlitePool>,
) -> Result<axum::Json<Vec<SensorData>>, AppError> {
    let rows = queryparam.rows.unwrap_or(10);
    list_last_values_descending_since(&pool, &get_date_with_default(&queryparam.since), rows)
        .await
        .map(Json::from)
        .map_err(AppError::from)
}
