use crate::model::{Drill, RunResult, Settings};
use serde::{Serialize, de::DeserializeOwned};
use std::{
    env, fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub struct Store {
    pub scenarios: Vec<crate::scenario::Scenario>,
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
        // Keep the original directory name so existing settings and results still load.
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
        let scenarios = match read::<Vec<crate::scenario::Scenario>>(&state.join("scenarios.json"))
        {
            Ok(Some(values))
                if values.len() <= 32 && values.iter().all(|s| s.validate().is_ok()) =>
            {
                values
            }
            Ok(None) => Vec::new(),
            _ => {
                notice = Some("Cannot load saved scenarios: invalid data".into());
                Vec::new()
            }
        };
        Self {
            scenarios,
            config,
            state,
            settings,
            results,
            notice,
        }
    }

    pub fn import(&mut self, path: &Path) -> Result<String, String> {
        let scenario = crate::scenario::load(path)?;
        if self.scenarios.iter().any(|s| s.id == scenario.id) {
            return Ok(scenario.id);
        }
        if self.scenarios.len() >= 32 {
            return Err("Import limit: 32 scenarios".into());
        }
        // Read the current file again; never overwrite a corrupt library with an empty one.
        let mut saved = read::<Vec<crate::scenario::Scenario>>(&self.state.join("scenarios.json"))?
            .unwrap_or_default();
        if saved.len() >= 32 || saved.iter().any(|s| s.validate().is_err()) {
            return Err("Saved scenario library is invalid or full".into());
        }
        let id = scenario.id.clone();
        if !saved.iter().any(|s| s.id == id) {
            saved.push(scenario);
        }
        write(&self.state.join("scenarios.json"), &saved)?;
        self.scenarios = saved;
        Ok(id)
    }

    pub fn best_import(&self, id: &str) -> f64 {
        self.results
            .iter()
            .filter(|r| r.scenario.as_ref().is_some_and(|s| s.id == id))
            .map(|r| r.score)
            .fold(0.0, f64::max)
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
        let dir = env::temp_dir().join(format!("aim-trainer-test-{}", std::process::id()));
        let mut store = Store::from_paths(dir.join("config"), dir.join("state"));
        store.settings.sensitivity = 2.35;
        store.settings.scale = crate::model::SensScale::MarvelRivals;
        store.save_settings();
        store.record(RunResult {
            drill: Drill::Six,
            scenario: None,
            score: 250.0,
            accuracy: 50.0,
            hits: 5,
            shots: 10,
            tracking_seconds: 0.0,
            timestamp: 1,
        });
        let loaded = Store::from_paths(dir.join("config"), dir.join("state"));
        assert_eq!(loaded.settings.sensitivity, 2.35);
        assert_eq!(loaded.settings.scale, crate::model::SensScale::MarvelRivals);
        assert_eq!(loaded.best(Drill::Six), 250.0);
        fs::write(dir.join("config/settings.json"), b"invalid json").expect("write fixture");
        let recovered = Store::from_paths(dir.join("config"), dir.join("state"));
        assert_eq!(recovered.settings.sensitivity, 1.0);
        assert!(recovered.notice.is_some());
        assert_eq!(recovered.results.len(), 1);
        fs::remove_dir_all(dir).expect("remove test data");
    }
}

#[cfg(test)]
mod import_tests {
    use super::*;
    #[test]
    fn import_is_atomic_persistent_and_results_stay_separate() {
        let dir = env::temp_dir().join(format!("aim-import-test-{}", std::process::id()));
        fs::create_dir_all(&dir).expect("directory");
        let source = dir.join("fixture.sce");
        fs::write(&source, include_str!("../tests/fixtures/pasu.sce")).expect("fixture");
        let mut store = Store::from_paths(dir.join("config"), dir.join("state"));
        let id = store.import(&source).expect("import");
        assert_eq!(store.import(&source).expect("duplicate"), id);
        let before = fs::read(dir.join("state/scenarios.json")).expect("saved");
        fs::write(&source, "invalid").expect("bad fixture");
        assert!(store.import(&source).is_err());
        assert_eq!(
            before,
            fs::read(dir.join("state/scenarios.json")).expect("saved")
        );
        let loaded = Store::from_paths(dir.join("config"), dir.join("state"));
        assert_eq!(loaded.scenarios.len(), 1);
        assert_eq!(loaded.scenarios[0].id, id);
        assert_eq!(loaded.scenarios[0].motion.jump, 4.125);
        let mut session = crate::model::Session::imported(loaded.scenarios[0].clone(), false, 12);
        session.phase = crate::model::Phase::Finished;
        session.hits = 4;
        session.shots = 8;
        store.record(session.result(1).expect("result"));
        let loaded = Store::from_paths(dir.join("config"), dir.join("state"));
        assert_eq!(loaded.best(Drill::Six), 0.0);
        assert!((loaded.best_import(&id) - 28.284271247).abs() < 1e-6);
        fs::remove_dir_all(dir).expect("cleanup");
    }
}
