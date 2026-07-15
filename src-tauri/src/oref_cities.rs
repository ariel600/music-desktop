
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

const CITIES_URL: &str =
    "https://raw.githubusercontent.com/eladnava/pikud-haoref-api/master/cities.json";
const CACHE_FILE: &str = "oref-cities.json";
const CACHE_MAX_AGE: Duration = Duration::from_secs(60 * 60 * 24 * 7);

#[derive(Debug, Deserialize)]
struct RawCity {
    id: i64,
    name: String,
    #[serde(default)]
    name_en: String,
    #[serde(default)]
    zone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrefCity {
    pub name: String,
    pub name_en: String,
    pub zone: String,
}

fn cache_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(CACHE_FILE)
}

fn cache_is_fresh(path: &Path) -> bool {
    let Ok(meta) = fs::metadata(path) else {
        return false;
    };
    let Ok(modified) = meta.modified() else {
        return false;
    };
    SystemTime::now()
        .duration_since(modified)
        .map(|age| age < CACHE_MAX_AGE)
        .unwrap_or(false)
}

fn parse_cities(json: &str) -> Result<Vec<OrefCity>, String> {
    let raw: Vec<RawCity> = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let mut cities: Vec<OrefCity> = raw
        .into_iter()
        .filter(|city| city.id != 0 && city.name.trim() != "בחר הכל")
        .map(|city| OrefCity {
            name: city.name.trim().to_string(),
            name_en: city.name_en.trim().to_string(),
            zone: city.zone.trim().to_string(),
        })
        .filter(|city| !city.name.is_empty())
        .collect();
    cities.sort_by(|a, b| a.name.cmp(&b.name));
    cities.dedup_by(|a, b| a.name == b.name);
    Ok(cities)
}

fn download_cities() -> Result<String, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;
    let response = client
        .get(CITIES_URL)
        .send()
        .map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!("failed to download cities list ({})", response.status()));
    }
    response.text().map_err(|e| e.to_string())
}

pub fn get_oref_cities(app_data_dir: &Path) -> Result<Vec<OrefCity>, String> {
    fs::create_dir_all(app_data_dir).map_err(|e| e.to_string())?;
    let path = cache_path(app_data_dir);

    if path.exists() && cache_is_fresh(&path) {
        let json = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        return parse_cities(&json);
    }

    match download_cities() {
        Ok(json) => {
            let cities = parse_cities(&json)?;
            if let Err(error) = fs::write(&path, &json) {
                tracing::warn!(target: "oref-cities", "cache write failed: {error}");
            } else {
                tracing::info!(target: "oref-cities", "cached {} cities from pikud-haoref-api", cities.len());
            }
            Ok(cities)
        }
        Err(error) => {
            if path.exists() {
                tracing::warn!(target: "oref-cities", "download failed ({error}) — using stale cache");
                let json = fs::read_to_string(&path).map_err(|e| e.to_string())?;
                return parse_cities(&json);
            }
            Err(error)
        }
    }
}

pub fn ensure_cities_cached_async(app_data_dir: PathBuf) {
    std::thread::spawn(move || {
        if let Err(error) = get_oref_cities(&app_data_dir) {
            tracing::warn!(target: "oref-cities", "warmup failed: {error}");
        }
    });
}

pub fn city_matches(alert_city: &str, monitored: &[String]) -> bool {
    let city = alert_city.trim();
    if city.is_empty() || monitored.is_empty() {
        return false;
    }
    monitored.iter().any(|entry| entry.trim() == city)
}
