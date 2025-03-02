use axum::{
    extract::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;

#[derive(Serialize, Deserialize, Clone)]
struct Score {
    player: String,
    score: u32,
}

// Shared state for storing scores
type Leaderboard = Arc<Mutex<Vec<Score>>>;

#[tokio::main]
async fn main() {
    let leaderboard: Leaderboard = Arc::new(Mutex::new(Vec::new()));

    let app = Router::new()
        .route("/leaderboard", get(get_leaderboard))
        .route("/submit", post(submit_score))
        .with_state(leaderboard);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Runnin on {} rn", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .await
        .unwrap();
}

// Submit a score
async fn submit_score(
    state: axum::extract::State<Leaderboard>,
    Json(new_score): Json<Score>,
) -> &'static str {
    let mut scores = state.lock().unwrap();
    scores.push(new_score);
    scores.sort_by(|a, b| b.score.cmp(&a.score));
    scores.dedup_by(|a, b| a.player == b.player);
    "Score submitted!1!!"
}

// Get the leaderboard
async fn get_leaderboard(
    state: axum::extract::State<Leaderboard>,
) -> Json<Vec<Score>> {
    let scores = state.lock().unwrap().clone();
    Json(scores)
}
