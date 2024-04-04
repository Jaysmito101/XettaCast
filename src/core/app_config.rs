use std::str::FromStr;

pub enum AppConfigItem {
    None,
    Monitor(String),
    Trigger(global_hotkey::hotkey::HotKey),
}

impl Default for AppConfigItem {
    fn default() -> Self {
        Self::None
    }
}

impl crate::ConfigItem for AppConfigItem {
    fn load(key: &str, value: &serde_yaml::Value) -> Result<Self, String> {
        match key {
            "monitor" => {
                let monitor = value.as_str().ok_or("Failed to get monitor as string")?;
                Ok(Self::Monitor(monitor.to_string()))
            },
            "trigger" => {
                let trigger = global_hotkey::hotkey::HotKey::from_str(value.as_str().ok_or("Failed to get trigger as string")?).map_err(|e| format!("Failed to parse trigger: {}", e))?;
                Ok(Self::Trigger(trigger))
            },
            _ => Err(format!("Unknown key: {}", key)),
        }
    }

    fn save(&self) -> (&str, serde_yaml::Value) {
        match self {
            Self::Monitor(monitor) => ("monitor", serde_yaml::Value::String(monitor.to_string())),
            Self::Trigger(_trigger) => ("trigger", serde_yaml::Value::String("cmd+alt+space".to_string())), // currently we cannot save hotkey
            Self::None => panic!("Cannot save None"),
        }
    }
}
 