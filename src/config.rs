use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Debug)]
pub struct UserConfig {
    pub host: HostConfig,
    pub languages: LanguagesConfig,
    pub skills: SkillsConfig,
    pub contact: ContactConfig,
}

#[derive(Deserialize, Debug)]
pub struct HostConfig {
    pub os: String,
    pub uptime: String,
    pub host: String,
    pub kernel: String,
    pub ide: String,
}

#[derive(Deserialize, Debug)]
pub struct LanguagesConfig {
    pub secondary: String,
    pub native: String,
}

#[derive(Deserialize, Debug)]
pub struct SkillsConfig {
    pub softskill: String,
    pub hardskill: String,
}

#[derive(Deserialize, Debug)]
pub struct ContactConfig {
    pub email: EmailConfig,
    #[serde(rename = "linkedIn")]
    pub linked_in: String,
    pub discord: String,
}

#[derive(Deserialize, Debug)]
pub struct EmailConfig {
    pub personal: String,
    pub work: String,
}

impl UserConfig {
    /// Baca & parse `.github/preferences.toml`. Sama untuk semua user, jadi
    /// cukup dipanggil sekali di `main`.
    pub fn load() -> Self {
        let config_path = ".github/preferences.toml";
        let config_str = fs::read_to_string(config_path)
            .unwrap_or_else(|_| panic!("gagal baca {config_path} — pastikan file config-nya ada"));
        toml::from_str(&config_str)
            .unwrap_or_else(|e| panic!("format TOML salah di {config_path}: {e}"))
    }
}
