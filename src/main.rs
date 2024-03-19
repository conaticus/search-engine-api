use axum::{routing::post, Json, Router};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct SearchQuery {
    query: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct QueryResult {
    execution_seconds: f32,
    results: Vec<String>,
}

async fn query(Json(body): Json<SearchQuery>) -> Json<QueryResult> {
    Json(QueryResult {
        execution_seconds: 0.2,
        results: Vec::new()
    })
}

#[tokio::main]
async fn main() {
    let router: Router<()> = Router::new().route("/api/query", post(query));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}
