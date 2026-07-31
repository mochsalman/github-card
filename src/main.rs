use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok(); // baca file .env

    let app = Router::new().route("/", get(|| async { "Server jalan!" }));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("Listening on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}
