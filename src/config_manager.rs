use std::str::FromStr;

pub struct ConfigManager {
    global_config: serde_yaml::Value,
    requires_restart: bool,
}

impl ConfigManager {

    pub async fn new() -> Self {

        let global_config = Self::load_global_config().unwrap();
        let requires_restart = false;

        Self {
            global_config,
            requires_restart,
        }
    }

    pub fn get_global(&self) -> &serde_yaml::Value {
        &self.global_config
    }
    
    pub fn save_global(&self) -> Result<(), String> {
        Self::save_global_config(&self.global_config)
    }
    
    pub fn update_global_config(&mut self, key: &str, value: String) {
        self.global_config[key] = serde_yaml::Value::String(value);
        self.requires_restart = true;
    }

    pub fn get_global_config(&self, key: &str) -> Option<&serde_yaml::Value> {
        self.global_config.get(key)
    }

    pub fn get_trigger(&self) -> Result<global_hotkey::hotkey::HotKey, String> {
        let trigger = self.global_config.get("trigger")
            .and_then(|value| value.as_str())
            .ok_or("Failed to get trigger")?;

        Ok(global_hotkey::hotkey::HotKey::from_str(trigger.trim()).map_err(|e| format!("Failed to parse trigger: {}", e))?)
    }
    
    pub fn requires_restart(&self) -> bool {
        self.requires_restart
    }

    fn load_global_config() -> Result<serde_yaml::Value, String> {
        // get path to home
        let home = match dirs::home_dir() {
            Some(home) => home,
            None => return Err("Failed to get home directory".to_string()),
        };

        // get path to config.yaml
        let config_path = home.join(".xettacast/global_config.yaml");

        // check if the directory exists
        if !home.join(".xettacast").exists() {
            log::info!("Failed to find ~/.xettacast, creating new one");
            // create directory
            match std::fs::create_dir_all(home.join(".xettacast")) {
                Ok(_) => (),
                Err(e) => return Err(format!("Failed to create .xettacast: {}", e)),
            }
        }

        // check if config.yaml exists
        if !config_path.exists() || true /* Force recreate config for debug */ {
            log::info!("Failed to find ~/.xettacast/global_config.yaml, creating new one");
            // create config.yaml
            match std::fs::write(&config_path, include_str!("./assets/config/default_global.yaml")) {
                Ok(_) => (),
                Err(e) => return Err(format!("Failed to create config.yaml: {}", e)),
            }
        }

        // read config.yaml
        let content = match std::fs::read_to_string(&config_path) {
            Ok(content) => content,
            Err(e) => return Err(format!("Failed to read config.yaml: {}", e)),
        };

        // parse config.yaml
        match serde_yaml::from_str(&content) {
            Ok(value) => Ok(value),
            Err(e) => Err(format!("Failed to parse config.yaml: {}", e)),
        }
    }


    fn save_global_config(value: &serde_yaml::Value) -> Result<(), String> {
        // get path to home
        let home = match dirs::home_dir() {
            Some(home) => home,
            None => return Err("Failed to get home directory".to_string()),
        };

        // get path to config.yaml
        let config_path = home.join(".xettacast/global_config.yaml");

        // serialize config.yaml
        let content = match serde_yaml::to_string(value) {
            Ok(content) => content,
            Err(e) => return Err(format!("Failed to serialize config.yaml: {}", e)),
        };

        // write config.yaml
        match std::fs::write(&config_path, content) {
            Ok(_) =>  {
                log::info!("Saved ~/.xettacast/global_config.yaml");
                Ok(())
            },
            Err(e) => Err(format!("Failed to write config.yaml: {}", e)),
        }
    }

}



