mod cache;
mod config;
mod format;
mod github;
mod template;

use config::UserConfig;
use std::fs;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok(); // tetap berguna buat testing lokal

    let allowed = std::env::var("ALLOWED_USERS").unwrap_or_default();
    let usernames: Vec<&str> = allowed
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    // config sama untuk semua user, jadi cukup dibaca sekali di luar loop
    let config = UserConfig::load();

    for username in usernames {
        println!("Generating stats untuk {username}...");

        match github::fetch_stats(username).await {
            Ok(stats) => {
                for theme in ["dark", "light"] {
                    let template_path = format!(".github/templates/card_{theme}.svg");
                    let template = fs::read_to_string(&template_path)
                        .unwrap_or_else(|_| panic!("gagal baca {template_path}"));

                    let svg = template::render_svg(&template, username, &stats, &config);

                    let out_path = format!(".github/{username}_{theme}.svg");
                    fs::write(&out_path, svg).expect("gagal tulis file SVG");
                    println!("  -> {out_path} tersimpan");
                }
            }
            Err(e) => eprintln!("  Gagal fetch {username}: {e}"),
        }
    }
}
