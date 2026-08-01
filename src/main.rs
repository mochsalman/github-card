use serde::Deserialize;
use serde_json::json;
use std::fs;

#[derive(Deserialize, Debug)]
struct GraphQLResponse { data: DataWrapper }
#[derive(Deserialize, Debug)]
struct DataWrapper { user: UserData }
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
struct RepoNode { stargazers: Stargazers }
#[derive(Deserialize, Debug)]
struct Stargazers {
    #[serde(rename = "totalCount")]
    total_count: u32,
}
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
            Ok((commits, repos, stars)) => {
                for theme in ["dark", "light"] {
                    let template_path = format!(".github/templates/card_{theme}.svg");
                    let template = fs::read_to_string(&template_path)
                        .unwrap_or_else(|_| panic!("gagal baca {template_path}"));

                    let svg = template
                        .replace("{{username}}", username)
                        .replace("{{repos}}", &repos.to_string())
                        .replace("{{stars}}", &stars.to_string())
                        .replace("{{commits}}", &commits.to_string());
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

async fn fetch_stats(username: &str) -> Result<(u32, u32, u32), String> {
    let token = std::env::var("GITHUB_PAT").map_err(|_| "GITHUB_PAT tidak ada".to_string())?;
    let client = reqwest::Client::new();

    let query = json!({
        "query": r#"
            query($login: String!) {
                user(login: $login) {
                    contributionsCollection { contributionCalendar { totalContributions } }
                    repositories(first: 100, ownerAffiliations: OWNER) {
                        totalCount
                        nodes { stargazers { totalCount } }
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

    Ok((commits, repos, stars))
}
