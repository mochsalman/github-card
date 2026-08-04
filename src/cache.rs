use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Cache LOC (lines of code) hasil hitung per-repo.
///
/// Disimpan flat di `.github/loc_cache/<hash>.json`, dengan `<hash>` = SHA-256
/// dari `"{owner}/{repo_name}"` (persis pola yang dipakai Andrew6rant/Andrew6rant
/// buat nyembunyiin nama repo di cache-nya). Efeknya:
/// - Nama repo (termasuk yang private) nggak pernah ketulis plaintext ke disk,
///   jadi nggak ke-commit & ke-push ke git history repo publik ini.
/// - Nggak butuh folder per-owner lagi karena `owner` udah ikut di-hash jadi
///   satu identitas unik per repo.
/// - Cache tetap deterministik: repo yang sama selalu hash ke nama file yang
///   sama, jadi cache hit/miss logic di `fetch_repo_loc` nggak berubah.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct RepoLocCache {
    pub processed_count: u64,
    pub add: u64,
    pub del: u64,
}

impl RepoLocCache {
    /// SHA-256 dari "owner/repo_name", dipakai sebagai nama file cache.
    /// Satu arah aja (nggak perlu, dan nggak bisa, di-reverse) -- yang penting
    /// konsisten, bukan rahasia-tapi-reversible.
    fn hashed_id(owner: &str, repo_name: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!("{owner}/{repo_name}").as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Baca cache dari disk (atau default kosong kalau belum ada), sekaligus
    /// mengembalikan path file cache-nya biar bisa dipakai lagi saat `save`.
    pub fn load(owner: &str, repo_name: &str) -> Result<(Self, String), String> {
        let cache_dir = ".github/loc_cache";
        std::fs::create_dir_all(cache_dir).map_err(|e| e.to_string())?;
        let cache_path = format!("{cache_dir}/{}.json", Self::hashed_id(owner, repo_name));

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
