use std::{env, fs, path::PathBuf};

#[derive(Default)]
pub struct AppState {
    pub recent_projects: Vec<PathBuf>,
}

impl AppState {
    pub fn load() -> Self {
        let Some(path) = state_path() else {
            return Self::default();
        };
        let Ok(contents) = fs::read_to_string(path) else {
            return Self::default();
        };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&contents) else {
            return Self::default();
        };
        let recent_projects = value["recent_projects"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(serde_json::Value::as_str)
            .map(PathBuf::from)
            .filter(|path| path.is_dir())
            .collect();
        Self { recent_projects }
    }

    pub fn remember_project(&mut self, path: PathBuf) {
        self.recent_projects.retain(|recent| recent != &path);
        self.recent_projects.insert(0, path);
        self.recent_projects.truncate(12);
        self.save();
    }

    fn save(&self) {
        let Some(path) = state_path() else {
            return;
        };
        let Some(parent) = path.parent() else {
            return;
        };
        if fs::create_dir_all(parent).is_err() {
            return;
        }
        let serialized = serde_json::json!({
            "version": 1,
            "recent_projects": self.recent_projects,
        });
        let Ok(bytes) = serde_json::to_vec_pretty(&serialized) else {
            return;
        };
        let temporary = path.with_extension("json.tmp");
        if fs::write(&temporary, bytes).is_ok() {
            let _ = fs::rename(temporary, path);
        }
    }
}

fn state_path() -> Option<PathBuf> {
    let base = env::var_os("SUBLEMON_DATA_DIR")
        .map(PathBuf::from)
        .or_else(|| env::var_os("XDG_DATA_HOME").map(PathBuf::from))
        .or_else(|| env::var_os("APPDATA").map(PathBuf::from))
        .or_else(|| {
            env::var_os("HOME")
                .or_else(|| env::var_os("USERPROFILE"))
                .map(PathBuf::from)
                .map(|home| home.join(".local").join("share"))
        })?;
    Some(base.join("sublemon").join("state.json"))
}
