pub trait ConfigItem {
    fn load(key: &str, value: &serde_yaml::Value) -> Result<Self, String> where Self: Sized;
    fn save(&self) -> (&str, serde_yaml::Value);
}

pub struct ConfigStore {
    data            : Option<serde_yaml::Value>,
    path            : String,
    default         : Option<String>,
    changed         : bool,
}

impl ConfigStore
{

    pub async fn new(path: String, default: Option<String>) -> Result<Self, String> {
        let mut value = Self {
            data            : None,
            path            : path,
            default         : default,
            changed         : false,
        };

        value.reload()?;

        Ok(value)
    }

    pub fn set_raw(&mut self, key: &str, value: String) -> Result<(), String> {
        self.data.as_mut().ok_or("Failed to get config data!")?[key] = serde_yaml::Value::String(value);
        self.changed = true;
        Ok(())
    }

    pub fn set<T : ConfigItem>(&mut self, data: T) -> Result<(), String> {
        let (key, data) = data.save();
        self.data.as_mut().ok_or("Failed to get config data!")?[key] = data;
        self.changed = true;
        Ok(())
    }

    pub fn get_raw(&self, key: &str) -> Result<String, String> {
        let value = self.data.as_ref().ok_or("Failed to get config data!")?[key].as_str().ok_or("Failed to get value as string")?;
        Ok(value.to_string())
    }

    // I am not very fond of using str as key, but I am not sure how to make it more generic
    pub fn get<T : ConfigItem>(&self, key: &str) -> Result<T, String> {
        let value = self.data.as_ref().ok_or("Failed to get config data!")?[key].clone();
        let value = T::load(key, &value)?;
        Ok(value)
    }

    pub fn reload(&mut self) -> Result<(), String> {

        let path = self.path.clone();

        if !std::path::Path::new(&path).exists() {
            return self.reset();
        }

        log::info!("Loading config from: {}", path);
        let data = std::fs::read_to_string(&path).map_err(|e| format!("Failed to read config: {}", e))?;
        let value = serde_yaml::from_str(&data).map_err(|e| format!("Failed to parse config: {}", e))?;
        self.data = Some(value);
        
        self.changed = false;

        Ok(())
    }

    pub fn reset(&mut self) -> Result<(), String> {
        let default = self.default.clone().ok_or("No default config provided")?;
        let value = serde_yaml::from_str(&default).map_err(|e| format!("Failed to parse default config: {}", e))?;
        self.data = Some(value);
        self.changed = false;
        self.save()
    }

    pub fn save(&mut self) -> Result<(), String> {
        let data = self.data.clone().ok_or("No data to save")?;
        let path = self.path.clone();
        let data = serde_yaml::to_string(&data).map_err(|e| format!("Failed to serialize data: {}", e))?;

        let dir = std::path::Path::new(&path).parent().ok_or("Failed to get parent directory")?;

        if !dir.exists() {
            log::info!("Creating directory: {}", dir.display());
            std::fs::create_dir_all(dir).map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        log::info!("Saving config to: {}", path);
        std::fs::write(&path, data).map_err(|e| format!("Failed to write config: {}", e))?;
        self.changed = false;

        Ok(())     
    }

}



