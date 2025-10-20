use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JsonShortcut {
    pub name: String,
    pub icon_path: String,
    pub command: String,
}

pub struct ShortcutStore {
    pub shortcuts: Vec<JsonShortcut>,
}

impl ShortcutStore {
    pub fn load_or_default(path: &str) -> Self {
        if let Ok(json) = std::fs::read_to_string(path) {
            if let Ok(data) = serde_json::from_str(&json) {
                return Self { shortcuts: data };
            }
        }
        Self { shortcuts: vec![] }
    }

    pub fn add(&mut self, shortcut: JsonShortcut) {
        self.shortcuts.push(shortcut);
    }

    pub fn save(&self, path: &str) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(&self.shortcuts)?;
        std::fs::write(path, json)
    }
}