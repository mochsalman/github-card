use axum::{routing::get, Router};
use serde::Deserialize;
use serde_json::json;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let app = Router::new()
        .route("/", get(|| async { "Server jalan!" }))
        .route("/api/stats/{username}", get(stats_handler));

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
    axum::extract::Path(username): axum::extract::Path<String>,
) -> String {
    match fetch_stats(&username).await {
        Ok(stats) => format!(
            "Commits: {}\nRepos: {}\nTotal Stars: {}",
            stats.0, stats.1, stats.2
        ),
        Err(e) => format!("Error: {e}"),
    }
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
