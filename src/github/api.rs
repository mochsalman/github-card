use serde_json::json;
use std::collections::HashMap;

use super::types::*;
use crate::cache::RepoLocCache;

/// Ringkasan statistik GitHub yang sudah dihitung, siap dipakai buat render SVG.
#[derive(Debug, Default)]
pub struct Stats {
    pub commits: u32,
    pub repos: u32,
    pub stars: u32,
    pub top_languages: String,
    pub contributed: u32,
    pub followers: u32,
    pub loc_add: u64,
    pub loc_del: u64,
}

impl Stats {
    /// Net lines of code = additions - deletions.
    pub fn loc_net(&self) -> u64 {
        self.loc_add.saturating_sub(self.loc_del)
    }
}

/// Ambil semua statistik profil (commits, repos, stars, bahasa, LOC, dst)
/// untuk satu username GitHub.
pub async fn fetch_stats(username: &str) -> Result<Stats, String> {
    let token = std::env::var("GITHUB_PAT").map_err(|_| "GITHUB_PAT tidak ada".to_string())?;
    let client = reqwest::Client::new();

    let query = json!({
        "query": r#"
            query($login: String!) {
                user(login: $login) {
                    contributionsCollection {
                        contributionCalendar { totalContributions }
                        totalRepositoriesWithContributedCommits
                    }
                    followers { totalCount }
                    repositories(first: 100, ownerAffiliations: OWNER) {
                        totalCount
                        nodes {
                            name
                            stargazers { totalCount }
                            languages(first: 10, orderBy: {field: SIZE, direction: DESC}) {
                                edges {
                                    size
                                    node { name }
                                }
                            }
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
    let stars: u32 = parsed.data.user.repositories.nodes.iter().map(|r| r.stargazers.total_count).sum();
    let contributed = parsed.data.user.contributions_collection.total_repositories_with_contributed_commits;
    let followers = parsed.data.user.followers.total_count;
    let top_languages = aggregate_top_languages(&parsed.data.user.repositories.nodes);

    let mut loc_add: u64 = 0;
    let mut loc_del: u64 = 0;
    for repo in &parsed.data.user.repositories.nodes {
        println!(" Menghitung LOC untuk repo: {}", repo.name);
        match fetch_repo_loc(&client, &token, username, &repo.name, username).await {
            Ok((add, del)) => {
                loc_add += add;
                loc_del += del;
            }
            Err(e) => eprintln!(" Gagal hitung loc repo {}: {e}", repo.name),
        }
    }

    Ok(Stats { commits, repos, stars, top_languages, contributed, followers, loc_add, loc_del })
}

/// Agregasi ukuran bahasa dari semua repo, ambil 5 teratas (urut terbesar).
fn aggregate_top_languages(repos: &[RepoNode]) -> String {
    let mut lang_totals: HashMap<String, u64> = HashMap::new();
    for repo in repos {
        for edge in &repo.languages.edges {
            *lang_totals.entry(edge.node.name.clone()).or_insert(0) += edge.size;
        }
    }

    let mut lang_vec: Vec<(String, u64)> = lang_totals.into_iter().collect();
    lang_vec.sort_by(|a, b| b.1.cmp(&a.1));

    lang_vec.into_iter().take(5).map(|(name, _)| name).collect::<Vec<_>>().join(", ")
}

/// Hitung total additions & deletions untuk 1 repo, khusus commit dari `username`.
/// Pakai cache di disk supaya commit lama nggak perlu di-fetch ulang.
async fn fetch_repo_loc(
    client: &reqwest::Client,
    token: &str,
    owner: &str,
    repo_name: &str,
    username: &str,
) -> Result<(u64, u64), String> {
    let (mut cache, cache_path) = RepoLocCache::load(owner, repo_name)?;

    let current_total = get_repo_commit_count(client, token, owner, repo_name).await?;

    if current_total == cache.processed_count {
        println!("    (cache hit, tidak ada commit baru)");
        return Ok((cache.add, cache.del));
    }

    let new_commits_count = current_total.saturating_sub(cache.processed_count);
    println!("    ({new_commits_count} commit baru, fetch detailnya...)");

    let mut fetched: u64 = 0;
    let mut cursor: Option<String> = None;

    'paging: loop {
        let query = json!({
            "query": r#"
                query($owner: String!, $name: String!, $cursor: String) {
                    repository(owner: $owner, name: $name) {
                        defaultBranchRef {
                            target {
                                ... on Commit {
                                    history(first: 100, after: $cursor) {
                                        pageInfo { hasNextPage endCursor }
                                        edges { node { additions deletions author { user { login } } } }
                                    }
                                }
                            }
                        }
                    }
                }
            "#,
            "variables": { "owner": owner, "name": repo_name, "cursor": cursor }
        });

        let response = client
            .post("https://api.github.com/graphql")
            .bearer_auth(token)
            .header("User-Agent", "github-readme-card")
            .json(&query)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let parsed: RepoLocResponse = response.json().await.map_err(|e| e.to_string())?;

        let Some(repo) = parsed.data.repository else { break };
        let Some(branch) = repo.default_branch_ref else { break };
        let Some(target) = branch.target else { break };
        let history = target.history;

        for edge in &history.edges {
            if fetched >= new_commits_count {
                break 'paging;
            }
            let is_mine = edge
                .node
                .author
                .user
                .as_ref()
                .map(|u| u.login.eq_ignore_ascii_case(username))
                .unwrap_or(false);
            if is_mine {
                cache.add += edge.node.additions;
                cache.del += edge.node.deletions;
            }
            fetched += 1;
        }

        if !history.page_info.has_next_page || fetched >= new_commits_count {
            break;
        }
        cursor = history.page_info.end_cursor;
    }

    cache.processed_count = current_total;
    cache.save(&cache_path)?;

    Ok((cache.add, cache.del))
}

/// Ambil total jumlah commit di default branch, dipakai untuk cek apakah
/// cache LOC sudah basi (ada commit baru sejak terakhir dihitung).
async fn get_repo_commit_count(
    client: &reqwest::Client,
    token: &str,
    owner: &str,
    repo_name: &str,
) -> Result<u64, String> {
    let query = json!({
        "query": r#"
            query($owner: String!, $name: String!) {
                repository(owner: $owner, name: $name) {
                    defaultBranchRef {
                        target {
                            ... on Commit { history { totalCount } }
                        }
                    }
                }
            }
        "#,
        "variables": { "owner": owner, "name": repo_name }
    });

    let response = client
        .post("https://api.github.com/graphql")
        .bearer_auth(token)
        .header("User-Agent", "github-readme-card")
        .json(&query)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let parsed: RepoCountResponse = response.json().await.map_err(|e| e.to_string())?;

    Ok(parsed
        .data
        .repository
        .and_then(|r| r.default_branch_ref)
        .and_then(|b| b.target)
        .map(|t| t.history.total_count)
        .unwrap_or(0))
}
