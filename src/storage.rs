use crate::model::{Drill, RunResult, Settings};
use serde::{Serialize, de::DeserializeOwned};
use std::{
    env, fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub struct Store {
    config: PathBuf,
    state: PathBuf,
    pub settings: Settings,
    pub results: Vec<RunResult>,
    pub notice: Option<String>,
}

pub fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(1, |d| d.as_secs())
}

fn xdg_path(variable: &str, fallback: &str) -> PathBuf {
    env::var_os(variable)
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .unwrap_or_else(|| {
            env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(env::temp_dir)
                .join(fallback)
        })
        .join("aim-room")
}

fn read<T: DeserializeOwned>(path: &Path) -> Result<Option<T>, String> {
    match fs::read(path) {
        Ok(bytes) => serde_json::from_slice(&bytes)
            .map(Some)
            .map_err(|e| format!("Cannot read {}: {e}", path.display())),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("Cannot read {}: {e}", path.display())),
    }
}

fn write<T: Serialize + ?Sized>(path: &Path, value: &T) -> Result<(), String> {
    let parent = path.parent().ok_or("Missing save directory")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    fs::write(&temporary, bytes).map_err(|e| e.to_string())?;
    fs::rename(&temporary, path).map_err(|e| e.to_string())
}

impl Store {
    pub fn load() -> Self {
        Self::from_paths(
            xdg_path("XDG_CONFIG_HOME", ".config"),
            xdg_path("XDG_STATE_HOME", ".local/state"),
        )
    }

    pub fn from_paths(config: PathBuf, state: PathBuf) -> Self {
        let mut notice = None;
        let mut settings = match read::<Settings>(&config.join("settings.json")) {
            Ok(value) => value.unwrap_or_default(),
            Err(e) => {
                notice = Some(e);
                Settings::default()
            }
        };
        settings.validate();
        let mut results = match read::<Vec<RunResult>>(&state.join("results.json")) {
            Ok(value) => value.unwrap_or_default(),
            Err(e) => {
                notice = Some(e);
                Vec::new()
            }
        };
        results.retain(RunResult::valid);
        if results.len() > 100 {
            results.drain(..results.len() - 100);
        }
        Self {
            config,
            state,
            settings,
            results,
            notice,
        }
    }

    pub fn save_settings(&mut self) {
        self.settings.validate();
        if let Err(e) = write(&self.config.join("settings.json"), &self.settings) {
            self.notice = Some(format!("Settings were not saved: {e}"));
        }
    }

    pub fn record(&mut self, result: RunResult) {
        if !result.valid() {
            self.notice = Some("Invalid result was not saved".into());
            return;
        }
        self.results.push(result);
        if self.results.len() > 100 {
            self.results.remove(0);
        }
        if let Err(e) = write(&self.state.join("results.json"), &self.results) {
            self.notice = Some(format!("Result was not saved: {e}"));
        }
    }

    pub fn best(&self, drill: Drill) -> f64 {
        self.results
            .iter()
            .filter(|r| r.drill == drill)
            .map(|r| r.score)
            .fold(0.0, f64::max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_and_results_survive_restart_and_bad_data_does_not_crash() {
        let dir = env::temp_dir().join(format!("aim-room-test-{}", std::process::id()));
        let mut store = Store::from_paths(dir.join("config"), dir.join("state"));
        store.settings.sensitivity = 2.35;
        store.save_settings();
        store.record(RunResult {
            drill: Drill::Six,
            score: 250.0,
            accuracy: 50.0,
            hits: 5,
            shots: 10,
            tracking_seconds: 0.0,
            timestamp: 1,
        });
        let loaded = Store::from_paths(dir.join("config"), dir.join("state"));
        assert_eq!(loaded.settings.sensitivity, 2.35);
        assert_eq!(loaded.best(Drill::Six), 250.0);
        fs::write(dir.join("config/settings.json"), b"invalid json").expect("write fixture");
        let recovered = Store::from_paths(dir.join("config"), dir.join("state"));
        assert_eq!(recovered.settings.sensitivity, 1.0);
        assert!(recovered.notice.is_some());
        assert_eq!(recovered.results.len(), 1);
        fs::remove_dir_all(dir).expect("remove test data");
    }
}
