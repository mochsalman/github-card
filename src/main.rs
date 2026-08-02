use chrono::{Datelike, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs;

#[derive(Deserialize, Debug)]
struct GraphQLResponse { data: DataWrapper }
#[derive(Deserialize, Debug)]
struct DataWrapper { user: UserData }

#[derive(Deserialize, Debug)]
struct UserData {
    #[serde(rename = "contributionsCollection")]
    contributions_collection: ContributionsCollection,
    followers: Followers,
    repositories: Repositories,
}
#[derive(Deserialize, Debug)]
struct ContributionsCollection {
    #[serde(rename = "contributionCalendar")]
    contribution_calendar: ContributionCalendar,
    #[serde(rename = "totalRepositoriesWithContributedCommits")]
    total_repositories_with_contributed_commits: u32,
}
#[derive(Deserialize, Debug)]
struct Followers {
    #[serde(rename = "totalCount")]
    total_count: u32,
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
struct Stargazers {
    #[serde(rename = "totalCount")]
    total_count: u32, } #[derive(Deserialize, Debug)]
struct UserConfig {
    host: HostConfig,
    languages: LanguagesConfig,
    skills: SkillsConfig,
    contact: ContactConfig,
}
#[derive(Deserialize, Debug)]
struct HostConfig {
    os: String,
    uptime: String,
    host: String,
    kernel: String,
    ide: String,
}
#[derive(Deserialize, Debug)]
struct LanguagesConfig {
    secondary: String,
    native: String,
}
#[derive(Deserialize, Debug)]
struct SkillsConfig {
    softskill: String,
    hardskill: String,
}
#[derive(Deserialize, Debug)]
struct ContactConfig {
    email: EmailConfig,
    #[serde(rename = "linkedIn")]
    linked_in: String,
    discord: String,
}
#[derive(Deserialize, Debug)]
struct EmailConfig {
    personal: String,
    work: String,
}
#[derive(Deserialize, Debug)]
struct RepoNode {
    name: String,
    stargazers: Stargazers,
    languages: LanguageConnection,
}
#[derive(Deserialize, Debug)]
struct LanguageConnection {
    edges: Vec<LanguageEdge>,
}
#[derive(Deserialize, Debug)]
struct LanguageEdge {
    size: u64,
    node: LanguageNode,
}
#[derive(Deserialize, Debug)]
struct LanguageNode {
    name: String,
}
#[derive(Deserialize, Debug)]
struct RepoLocResponse {
    data: RepoLocData,
}
#[derive(Deserialize, Debug)]
struct RepoLocData {
    repository: Option<RepositoryHistory>,
}
#[derive(Deserialize, Debug)]
struct RepositoryHistory {
    #[serde(rename = "defaultBranchRef")]
    default_branch_ref: Option<DefaultBranchRef>,
}
#[derive(Deserialize, Debug)]
struct DefaultBranchRef {
    target: Option<CommitTarget>,
}
#[derive(Deserialize, Debug)]
struct CommitTarget {
    history: CommitHistory,
}
#[derive(Deserialize, Debug)]
struct CommitHistory {
    #[serde(rename = "pageInfo")]
    page_info: PageInfo,
    edges: Vec<CommitEdge>,
}
#[derive(Deserialize, Debug)]
struct PageInfo {
    #[serde(rename = "hasNextPage")]
    has_next_page: bool,
    #[serde(rename = "endCursor")]
    end_cursor: Option<String>,
}
#[derive(Deserialize, Debug)]
struct CommitEdge {
    node: CommitNode,
}
#[derive(Deserialize, Debug)]
struct CommitNode {
    additions: u64,
    deletions: u64,
    author: CommitAuthor,
}
#[derive(Deserialize, Debug)]
struct CommitAuthor {
    user: Option<CommitAuthorUser>, // bisa null kalau akun sudah dihapus, dsb
}
#[derive(Deserialize, Debug)]
struct CommitAuthorUser {
    login: String,
}
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
struct RepoLocCache {
    processed_count: u64,
    add: u64,
    del: u64,
}
#[derive(Deserialize, Debug)]
struct RepoCountResponse { data: RepoCountData }
#[derive(Deserialize, Debug)]
struct RepoCountData { repository: Option<RepoCountRepository> }
#[derive(Deserialize, Debug)]
struct RepoCountRepository { #[serde(rename = "defaultBranchRef")] default_branch_ref: Option<RepoCountBranch> }
#[derive(Deserialize, Debug)]
struct RepoCountBranch { target: Option<RepoCountTarget> }
#[derive(Deserialize, Debug)]
struct RepoCountTarget { history: RepoCountHistory }
#[derive(Deserialize, Debug)]
struct RepoCountHistory { #[serde(rename = "totalCount")] total_count: u64 }


#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok(); // tetap berguna buat testing lokal

    let allowed = std::env::var("ALLOWED_USERS").unwrap_or_default();
    let usernames: Vec<&str> = allowed.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();

    // pastikan folder output ada
    // fs::create_dir_all("generated").expect("gagal bikin folder generated/"); // pindah ke .github/

    for username in usernames {
        println!("Generating stats untuk {username}...");

        // load config TOML khusus user ini
        let config_path = format!(".github/preferences.toml");
        let config_str = fs::read_to_string(&config_path)
            .unwrap_or_else(|_| panic!("gagal baca {config_path} — pastikan file config-nya ada"));
        let config: UserConfig = toml::from_str(&config_str)
            .unwrap_or_else(|e| panic!("format TOML salah di {config_path}: {e}"));

        match fetch_stats(username).await {
            Ok((commits, repos, stars, top_languages, contributed, followers, loc_add, loc_del)) => {
                let loc_data = loc_add.saturating_sub(loc_del); // ner lines = tambah - hapus
                for theme in ["dark", "light"] {
                    let template_path = format!(".github/templates/card_{theme}.svg");
                    let template = fs::read_to_string(&template_path)
                        .unwrap_or_else(|_| panic!("gagal baca {template_path}"));

                    let uptime_display = if config.host.uptime == "-" {
                        "-".to_string()
                    } else {
                        calculate_uptime(&config.host.uptime)
                    };

                    let svg = template
                        .replace("{{username}}", username)
                        .replace("{{repos}}", &repos.to_string())
                        .replace("{{stars}}", &stars.to_string())
                        .replace("{{commits}}", &commits.to_string())
                        .replace("{{lang_programming}}", &top_languages)
                        .replace("{{contributed}}", &contributed.to_string())
                        .replace("{{follower}}", &followers.to_string())
                        .replace("{{loc_data}}", &loc_data.to_string())
                        .replace("{{loc_add}}", &loc_add.to_string())
                        .replace("{{loc_del}}", &loc_del.to_string())
                        .replace("{{uptime}}", &uptime_display)
                        // field baru
                        .replace("{{os}}", &config.host.os)
                        .replace("{{uptime}}", &config.host.uptime)
                        .replace("{{host}}", &config.host.host)
                        .replace("{{kernel}}", &config.host.kernel)
                        .replace("{{ide}}", &config.host.ide)
                        .replace("{{lang_secondary}}", &config.languages.secondary)
                        .replace("{{lang_native}}", &config.languages.native)
                        .replace("{{softskill}}", &config.skills.softskill)
                        .replace("{{hardskill}}", &config.skills.hardskill)
                        .replace("{{email_personal}}", &config.contact.email.personal)
                        .replace("{{email_work}}", &config.contact.email.work)
                        .replace("{{linkedin}}", &config.contact.linked_in)
                        .replace("{{discord}}", &config.contact.discord);

                    let out_path = format!(".github/{username}_{theme}.svg");
                    fs::write(&out_path, svg).expect("gagal tulis file SVG");
                    println!("  -> {out_path} tersimpan");
                }
            }
            Err(e) => eprintln!("  Gagal fetch {username}: {e}"),
        }
    }
}



// Hitung total additions & deletions untuk 1 repo, khusus commit dari `username`
async fn fetch_repo_loc(
    client: &reqwest::Client,
    token: &str,
    owner: &str,
    repo_name: &str,
    username: &str,
) -> Result<(u64, u64), String> {
    let cache_dir = format!(".github/loc_cache/{owner}");
    std::fs::create_dir_all(&cache_dir).map_err(|e| e.to_string())?;
    let cache_path = format!("{cache_dir}/{repo_name}.json");

    let mut cache: RepoLocCache = std::fs::read_to_string(&cache_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

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

        let response = client.post("https://api.github.com/graphql")
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
            if fetched >= new_commits_count { break 'paging; }
            let is_mine = edge.node.author.user.as_ref()
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
    let cache_json = serde_json::to_string_pretty(&cache).map_err(|e| e.to_string())?;
    std::fs::write(&cache_path, cache_json).map_err(|e| e.to_string())?;

    Ok((cache.add, cache.del))
}

async fn fetch_stats(username: &str) -> Result<(u32, u32, u32, String, u32, u32, u64, u64), String> {
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

    // --- agregasi bahasa dari semua repo ---
    let mut lang_totals: HashMap<String, u64> = HashMap::new();
    for repo in &parsed.data.user.repositories.nodes {
        for edge in &repo.languages.edges {
            *lang_totals.entry(edge.node.name.clone()).or_insert(0) += edge.size;
        }
    }

    let mut lang_vec: Vec<(String, u64)> = lang_totals.into_iter().collect();
    lang_vec.sort_by(|a, b| b.1.cmp(&a.1)); // urut dari terbesar

    let top_languages: String = lang_vec
        .into_iter()
        .take(5)
        .map(|(name, _)| name)
        .collect::<Vec<_>>()
        .join(", ");

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


    Ok((commits, repos, stars, top_languages, contributed, followers, loc_add, loc_del))
}

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

    let response = client.post("https://api.github.com/graphql")
        .bearer_auth(token)
        .header("User-Agent", "github-readme-card")
        .json(&query)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let parsed: RepoCountResponse = response.json().await.map_err(|e| e.to_string())?;

    Ok(parsed.data.repository
        .and_then(|r| r.default_branch_ref)
        .and_then(|b| b.target)
        .map(|t| t.history.total_count)
        .unwrap_or(0))
}

// Hitung selisih kalender (tahun, bulan, hari) dari tanggal `dd/mm/yyyy` sampai hari ini
fn calculate_uptime(date_str: &str) -> String {
    let birth = match NaiveDate::parse_from_str(date_str, "%d/%m/%Y") {
        Ok(d) => d,
        Err(_) => return "-".to_string(), // format tanggal salah, fallback aman
    };

    let today = Utc::now().date_naive();

    if birth > today {
        return "-".to_string(); // tanggal di masa depan, nggak masuk akal dihitung
    }

    let mut years = today.year() - birth.year();
    let mut months = today.month() as i32 - birth.month() as i32;
    let mut days = today.day() as i32 - birth.day() as i32;

    if days < 0 {
        months -= 1;
        let (prev_year, prev_month) = if today.month() == 1 {
            (today.year() - 1, 12)
        } else {
            (today.year(), today.month() - 1)
        };
        days += days_in_month(prev_year, prev_month) as i32;
    }

    if months < 0 {
        years -= 1;
        months += 12;
    }

    format!(
        "{} year{}, {} month{}, {} day{}",
        years, if years != 1 { "s" } else { "" },
        months, if months != 1 { "s" } else { "" },
        days, if days != 1 { "s" } else { "" },
    )
}

// Helper: jumlah hari dalam bulan tertentu (buat "pinjam" hari saat days < 0)
fn days_in_month(year: i32, month: u32) -> u32 {
    let next_month = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }.unwrap();
    let this_month = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    (next_month - this_month).num_days() as u32
}
