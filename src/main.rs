mod database;

use std::collections::HashMap;

use axum::{routing::post, Json, Router};
use database::{
    pool::init_pool,
    queries::{get_document_count, get_keywords},
};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

use crate::database::queries::Keyword;

#[derive(Deserialize)]
struct SearchQuery {
    query: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Website {
    pub title: String,
    pub description: String,
    pub url: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct QueryResult {
    execution_seconds: f32,
    results: Vec<Website>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiError {
    error: String,
}

fn lemmanize(text: String) -> Vec<String> {
    text.to_lowercase()
        .chars()
        .filter(|c| !c.is_ascii_punctuation())
        .collect::<String>()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect()
}

async fn query(
    db_pool: Pool<Postgres>,
    body: SearchQuery,
    document_count: i64,
) -> Json<QueryResult> {
    let query_length = body.query.len();
    let lemmanized_query = lemmanize(body.query);
    let keywords = get_keywords(db_pool, &lemmanized_query)
        .await
        .expect("Failed to query keywords"); // TODO: Handle error properly

    let mut website_keywords: HashMap<String, (Vec<Keyword>, Website)> = HashMap::new();
    for keyword in keywords {
        // Should later probably use GROUP BY in the SQL query instead of this, if possible
        website_keywords
            .entry(keyword.url.clone())
            .or_insert((
                Vec::new(),
                Website {
                    url: keyword.url.clone(),
                    title: keyword.title.clone(),
                    description: keyword.description.clone(),
                },
            ))
            .0
            .push(keyword);
    }

    let mut query_term_tfs: HashMap<String, f32> = HashMap::new();
    let mut query_word_occurences: HashMap<String, i32> = HashMap::new();

    for word in &lemmanized_query {
        *query_word_occurences.entry(word.to_string()).or_insert(0) += 1;
    }

    for word in lemmanized_query {
        let tf = query_word_occurences[&word] as f32 / query_length as f32;
        query_term_tfs.insert(word, tf);
    }

    let mut website_similarities: Vec<(f32, Website)> = Vec::new();

    for (_, (keywords, website)) in website_keywords {
        let mut query_vector_sum = 0.;
        let mut document_vector_sum = 0.;

        let mut dot_product = 0.;

        for keyword in keywords {
            let tf = keyword.occurrences as f32 / keyword.word_count as f32;
            let idf = 1. + (document_count as f32 / keyword.documents_containing_word as f32).ln();
            let tf_idf = tf * idf;

            let query_tf_idf = query_term_tfs[&keyword.word] * idf;
            query_vector_sum += query_tf_idf.powi(2);
            document_vector_sum += tf_idf.powi(2);

            dot_product += query_tf_idf * tf_idf;
        }

        let query_vector = query_vector_sum.sqrt();
        let document_vector = document_vector_sum.sqrt();
        let similarity = dot_product / (query_vector * document_vector);
        website_similarities.push((similarity, website));
    }

    website_similarities.sort_by(|(a_sim, _), (b_sim, _)| a_sim.partial_cmp(b_sim).unwrap());
    let results = website_similarities
        .into_iter()
        .map(|(_, website)| website)
        .collect::<Vec<Website>>();

    Json(QueryResult {
        execution_seconds: 0.2,
        results: results,
    })
}

#[tokio::main]
async fn main() {
    dotenv().expect("Failed to initialise .env");

    let pool = init_pool().await.expect("Failed to create pool");
    let document_count = get_document_count(pool.clone())
        .await
        .expect("Failed to get document count")
        .0;

    let router: Router<()> = Router::new().route(
        "/api/query",
        post(move |Json(body): Json<SearchQuery>| async move {
            query(pool, body, document_count).await
        }),
    );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}
