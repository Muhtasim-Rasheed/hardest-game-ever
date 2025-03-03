mod router;

#[tokio::main]
async fn main() {
    let app = router::router();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Runnin on {} rn", listener.local_addr().unwrap());
    let _ = axum::serve(listener, app)
        .await
        .unwrap();
}
