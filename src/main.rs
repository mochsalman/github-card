use axum::{routing::get, Router};
use serde_json::json;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    // sebelum server nyala, kita test dulu koneksi ke GitHub
    match test_github_connection().await {
        Ok(login) => println!("✅ Token valid, login sebagai: {login}"),
        Err(e) => println!("❌ Gagal konek ke GitHub: {e}"),
    }

    let app = Router::new().route("/", get(|| async { "Server jalan!" }));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("Listening on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}

// Fungsi kecil khusus buat ngetes: query paling simpel ke GraphQL GitHub,
// cuma minta "siapa saya" (viewer.login) berdasarkan token yang dipakai.
async fn test_github_connection() -> Result<String, reqwest::Error> {
    let token = std::env::var("GITHUB_PAT").expect("GITHUB_PAT tidak ditemukan di .env");

    let client = reqwest::Client::new();

    let query = json!({
        "query": "{ viewer { login } }"
    });

    let response = client
        .post("https://api.github.com/graphql")
        .bearer_auth(&token)          // ini cara kirim token via header Authorization
        .header("User-Agent", "github-readme-card") // GitHub API wajib ada User-Agent
        .json(&query)
        .send()
        .await?;

    let body: serde_json::Value = response.json().await?;

    // ambil field viewer.login dari hasil JSON
    let login = body["data"]["viewer"]["login"]
        .as_str()
        .unwrap_or("unknown")
        .to_string();

    Ok(login)
}
