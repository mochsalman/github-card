use serde::{Deserialize, Serialize};

/// Cache LOC (lines of code) hasil hitung per-repo, disimpan di
/// `.github/loc_cache/<owner>/<repo>.json` supaya nggak perlu re-fetch
/// commit yang sudah pernah diproses.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct RepoLocCache {
    pub processed_count: u64,
    pub add: u64,
    pub del: u64,
}

impl RepoLocCache {
    /// Baca cache dari disk (atau default kosong kalau belum ada), sekaligus
    /// mengembalikan path file cache-nya biar bisa dipakai lagi saat `save`.
    pub fn load(owner: &str, repo_name: &str) -> Result<(Self, String), String> {
        let cache_dir = format!(".github/loc_cache/{owner}");
        std::fs::create_dir_all(&cache_dir).map_err(|e| e.to_string())?;
        let cache_path = format!("{cache_dir}/{repo_name}.json");

        let cache = std::fs::read_to_string(&cache_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        Ok((cache, cache_path))
    }

    pub fn save(&self, cache_path: &str) -> Result<(), String> {
        let cache_json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(cache_path, cache_json).map_err(|e| e.to_string())
    }
}
