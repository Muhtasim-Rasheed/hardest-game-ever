use axum::{
    extract::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
// use tokio::net::TcpListener;

#[derive(Serialize, Deserialize, Clone)]
pub struct Score {
    pub player: String,
    pub score: u32,
}

// Shared state for storing scores
// pub type Leaderboard = Arc<Mutex<Vec<Score>>>;
#[derive(Clone)]
pub struct Leaderboard {
    scores: Arc<Mutex<Vec<Score>>>,
}

pub fn router() -> Router {
    let leaderboard = Leaderboard {
        scores: Arc::new(Mutex::new(Vec::new())),
    };

    Router::new()
        .route("/leaderboard", get(get_leaderboard))
        .route("/submit", post(submit_score))
        .with_state(leaderboard)
}

// Submit a score
async fn submit_score(
    axum::extract::State(state): axum::extract::State<Leaderboard>,
    Json(new_score): Json<Score>,
) -> &'static str {
    let mut scores = state.scores.lock().unwrap();
    scores.push(new_score);
    scores.sort_by(|a, b| b.score.cmp(&a.score));
    scores.dedup_by(|a, b| a.player == b.player);
    "Score submitted!1!!"
}

// Get the leaderboard
async fn get_leaderboard(
    axum::extract::State(state): axum::extract::State<Leaderboard>,
) -> Json<Vec<Score>> {
    let scores = state.scores.lock().unwrap();
    Json(scores.clone())
}
