mod router;

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    Ok(router::router().into())
}
