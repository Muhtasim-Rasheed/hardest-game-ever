use axum::{
    extract::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
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
        .route("/world", get(world))
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

#[derive(Serialize, Deserialize)]
struct World {
    objects: Vec<serde_json::Value>,
    poly_objects: Vec<serde_json::Value>,
    moving_objects: Vec<serde_json::Value>,
    speed_increases: Vec<serde_json::Value>,
}

impl ToString for World {
    fn to_string(&self) -> String {
        json!({
            "objects": self.objects,
            "poly_objects": self.poly_objects,
            "moving_objects": self.moving_objects,
            "speed_increases": self.speed_increases,
        })
        .to_string()
    }
}

async fn world() -> Json<String> {
    // "{\"moving_objects\":[{\"from\":{\"x\":1100.0,\"y\":-225.0},\"height\":100.0,\"speed\":3.0,\"to\":{\"x\":1200.0,\"y\":150.0},\"width\":50.0},{\"from\":{\"x\":1300.0,\"y\":150.0},\"height\":100.0,\"speed\":3.0,\"to\":{\"x\":1400.0,\"y\":-225.0},\"width\":50.0},{\"from\":{\"x\":1500.0,\"y\":0.0},\"height\":50.0,\"speed\":3.0,\"to\":{\"x\":1600.0,\"y\":0.0},\"width\":100.0}],\"objects\":[{\"height\":50.0,\"width\":5000.0,\"x\":0.0,\"y\":250.0},{\"height\":50.0,\"width\":5000.0,\"x\":0.0,\"y\":-300.0},{\"height\":235.0,\"width\":50.0,\"x\":500.0,\"y\":15.0},{\"height\":235.0,\"width\":50.0,\"x\":625.0,\"y\":-250.0}],\"poly_objects\":[{\"points\":[{\"x\":775.0,\"y\":-80.0},{\"x\":775.0,\"y\":250.0},{\"x\":1075.0,\"y\":250.0},{\"x\":1075.0,\"y\":0.0},{\"x\":975.0,\"y\":-80.0}]}],\"speed_increases\":[{\"speed_change\":2.0,\"x\":1075.0,\"y\":-200.0}]}"
    
    let world = World {
        objects: vec![
            json!({
                "x": 0.0,
                "y": 250.0,
                "width": 5000.0,
                "height": 50.0,
            }),
            json!({
                "x": 0.0,
                "y": -300.0,
                "width": 5000.0,
                "height": 50.0,
            }),
            json!({
                "x": 500.0,
                "y": 15.0,
                "width": 50.0,
                "height": 235.0,
            }),
            json!({
                "x": 625.0,
                "y": -250.0,
                "width": 50.0,
                "height": 235.0,
            }),
        ],
        poly_objects: vec![json!({
            "points": [
                {"x": 775.0, "y": -80.0},
                {"x": 775.0, "y": 250.0},
                {"x": 1075.0, "y": 250.0},
                {"x": 1075.0, "y": 0.0},
                {"x": 975.0, "y": -80.0},
            ],
        })],
        moving_objects: vec![
            json!({
                "from": {"x": 1100.0, "y": -225.0},
                "to": {"x": 1200.0, "y": 150.0},
                "width": 50.0,
                "height": 100.0,
                "speed": 3.0,
            }),
            json!({
                "from": {"x": 1300.0, "y": 150.0},
                "to": {"x": 1400.0, "y": -225.0},
                "width": 50.0,
                "height": 100.0,
                "speed": 3.0,
            }),
            json!({
                "from": {"x": 1500.0, "y": 0.0},
                "to": {"x": 1600.0, "y": 0.0},
                "width": 100.0,
                "height":
                50.0,
                "speed": 3.0,
            }),
        ],
        speed_increases: vec![json!({
            "x": 1075.0,
            "y": -200.0,
            "speed_change": 2.0,
        })],
    };
    Json(world.to_string())
}
