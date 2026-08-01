use axum::{
    extract::{Path, State, Query},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};

use moka::future::Cache;
use serde::Deserialize;
use serde_json::json;
use std::time::Duration;
use std::collections::HashMap;

// tipe cache: key = username, value = SVG string yang sudah jadi
type StatsCache = Cache<String, String>;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let cache: StatsCache = Cache::builder()
        .time_to_live(Duration::from_secs(60 * 60)) // 1 jam
        .max_capacity(100) // maksimal 100 entry username berbeda
        .build();

    let app = Router::new()
        .route("/", get(|| async { "Server jalan!" }))
        .route("/api/stats/{username}", get(stats_handler))
        .with_state(cache); // <-- ini yang tadinya hilang

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("Listening on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}

// --- Struct buat mencocokkan bentuk JSON dari GitHub ---
// Nama field di sini HARUS sama persis dengan nama field di query GraphQL di bawah.
#[derive(Deserialize, Debug)]
struct GraphQLResponse {
    data: DataWrapper,
}

#[derive(Deserialize, Debug)]
struct DataWrapper {
    user: UserData,
}

#[derive(Deserialize, Debug)]
struct UserData {
    #[serde(rename = "contributionsCollection")]
    contributions_collection: ContributionsCollection,
    repositories: Repositories,
}

#[derive(Deserialize, Debug)]
struct ContributionsCollection {
    #[serde(rename = "contributionCalendar")]
    contribution_calendar: ContributionCalendar,
}

#[derive(Deserialize, Debug)]
struct ContributionCalendar {
    #[serde(rename = "totalContributions")]
    total_contributions: u32,
}

#[derive(Deserialize, Debug)]
struct Repositories {
    #[serde(rename = "totalCount")]
    total_count: u32,
    nodes: Vec<RepoNode>,
}

#[derive(Deserialize, Debug)]
struct RepoNode {
    stargazers: Stargazers,
}

#[derive(Deserialize, Debug)]
struct Stargazers {
    #[serde(rename = "totalCount")]
    total_count: u32,
}

// --- Handler untuk GET /api/stats/{username} ---
async fn stats_handler(
    Path(username): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    State(cache): State<StatsCache>,
) -> Response {
    if !is_allowed(&username) {
        return (StatusCode::NOT_FOUND, "Username tidak terdaftar").into_response();
    }

    let theme = params.get("theme").map(|s| s.as_str()).unwrap_or("dark");
    let cache_key = format!("{username}:{theme}"); // penting! cache dipisah per tema
    
    // Check cache
    if let Some(cached_svg) = cache.get(&cache_key).await {
        println!("Cache HIT untuk {username}");
        return ([(header::CONTENT_TYPE, "image/svg+xml")], cached_svg).into_response();
    }

    println!("Cache MISS untuk {username}, fetch dari GitHub...");

    match fetch_stats(&username).await {
        Ok((commits, repos, stars)) => {
            let template_path = if theme == "light" {
                "templates/card_light.svg"
            } else {
                "templates/card_dark.svg"  
            };
            
            let template = std::fs::read_to_string(template_path)
                .expect("gagal baca template SVG");

            let svg = template
                .replace("{{username}}", &username)
                .replace("{{repos}}", &repos.to_string())
                .replace("{{stars}}", &stars.to_string())
                .replace("{{commits}}", &commits.to_string());

            cache.insert(username.clone(), svg.clone()).await;

            (
                [(header::CONTENT_TYPE, "image/svg+xml")],
                svg,
            )
                .into_response()
        }
        Err(e) => format!("Error: {e}").into_response(),
    }
}

// Fungsi kecil: cek apakah username ada di ALLOWED_USERS
fn is_allowed(username: &str) -> bool {
    let allowed = std::env::var("ALLOWED_USERS").unwrap_or_default();
    allowed
        .split(',')
        .map(|s| s.trim())
        .any(|allowed_user| allowed_user.eq_ignore_ascii_case(username))
}

// Return (total_commits, total_repos, total_stars)
async fn fetch_stats(username: &str) -> Result<(u32, u32, u32), String> {
    let token = std::env::var("GITHUB_PAT").map_err(|_| "GITHUB_PAT tidak ada di .env".to_string())?;
    let client = reqwest::Client::new();

    let query = json!({
        "query": r#"
            query($login: String!) {
                user(login: $login) {
                    contributionsCollection {
                        contributionCalendar {
                            totalContributions
                        }
                    }
                    repositories(first: 100, ownerAffiliations: OWNER) {
                        totalCount
                        nodes {
                            stargazers { totalCount }
                        }
                    }
                }
            }
        "#,
        "variables": { "login": username }
    });

    let response = client
        .post("https://api.github.com/graphql")
        .bearer_auth(&token)
        .header("User-Agent", "github-readme-card")
        .json(&query)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let parsed: GraphQLResponse = response.json().await.map_err(|e| e.to_string())?;

    let commits = parsed.data.user.contributions_collection.contribution_calendar.total_contributions;
    let repos = parsed.data.user.repositories.total_count;
    let stars: u32 = parsed.data.user.repositories.nodes.iter()
        .map(|r| r.stargazers.total_count)
        .sum();

    Ok((commits, repos, stars))
}
