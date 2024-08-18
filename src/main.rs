use std::{net::SocketAddr, time::SystemTime};

use axum::{extract::Path, response::IntoResponse, routing::get, Router};
use serde_json::json;

use aws_sdk_dynamodb::types::AttributeValue;

const HEADERS: [(&'static str, &'static str); 4] = [
    ("x-powered-by", "benchmark"),
    ("content-type", "application/json"),
    ("connection", "keep-alive"),
    ("keep-alive", "timeout=5"),
];

async fn handle_request(Path(name): Path<String>) -> impl IntoResponse {
    dotenv::dotenv().ok();

    let config = aws_config::from_env().load().await;
    let client = aws_sdk_dynamodb::Client::new(&config);

    let now = SystemTime::now();

    let response = client
        .get_item()
        .table_name("terms-of-use")
        .key("platform", AttributeValue::S("IAP Online".to_string()))
        .send()
        .await
        .unwrap();

    println!("{:?}", now.elapsed().unwrap().as_millis());

    let documents = response.item.unwrap();

    let terms = documents.get("terms").unwrap().as_s().ok().unwrap();

    return (
        HEADERS,
        json!({
            "name": name,
            "terms": terms
        })
        .to_string(),
    );
}

async fn handle_basic() -> impl IntoResponse {
    (
        HEADERS,
        json!({
            "message": "success",
        })
        .to_string(),
    )
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/bench/:name", get(handle_request))
        .route("/", get(handle_basic));
    let port_number: u16 = str::parse("3000").unwrap();

    let addr = SocketAddr::from(([0, 0, 0, 0], port_number));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
