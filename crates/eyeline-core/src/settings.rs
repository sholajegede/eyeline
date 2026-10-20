use serde::{Deserialize, Serialize};
use std::{fs, io, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Settings {
    pub scripts_dir: Option<String>,
    pub mode: String,
    pub font_size: f64,
    pub speed: f64,
    pub mirror_x: bool,
    pub mirror_y: bool,
    pub text_color: String,
    pub width: f64,
    pub height: f64,
    pub hide_from_capture: bool,
    pub countdown_secs: u32,
    pub dim_read: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            scripts_dir: None,
            mode: "notch".into(),
            font_size: 30.0,
            speed: 60.0,
            mirror_x: false,
            mirror_y: false,
            text_color: "#f5f5f0".into(),
            width: 640.0,
            height: 168.0,
            hide_from_capture: true,
            countdown_secs: 3,
            dim_read: true,
        }
    }
}

impl Settings {
    pub fn load(path: &Path) -> Self {
        fs::read_to_string(path)
            .ok()
            .and_then(|raw| serde_json::from_str::<Settings>(&raw).ok())
            .map(Settings::sanitized)
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let tmp = path.with_extension("json.tmp");
        let body = serde_json::to_vec_pretty(self).map_err(io::Error::other)?;
        fs::write(&tmp, body)?;
        fs::rename(tmp, path)
    }

    pub fn sanitized(mut self) -> Self {
        if self.mode != "notch" && self.mode != "floating" {
            self.mode = "notch".into();
        }
        self.font_size = self.font_size.clamp(14.0, 96.0);
        self.speed = self.speed.clamp(4.0, 400.0);
        self.width = self.width.clamp(280.0, 2400.0);
        self.height = self.height.clamp(80.0, 1400.0);
        self.countdown_secs = self.countdown_secs.min(10);
        if !is_hex_color(&self.text_color) {
            self.text_color = Settings::default().text_color;
        }
        self
    }
}

fn is_hex_color(value: &str) -> bool {
    let digits = value.strip_prefix('#').unwrap_or("");
    matches!(digits.len(), 3 | 6) && digits.chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_out_of_range_values() {
        let s = Settings {
            font_size: 900.0,
            speed: -4.0,
            countdown_secs: 99,
            mode: "weird".into(),
            text_color: "red; background:url(x)".into(),
            ..Settings::default()
        }
        .sanitized();
        assert_eq!(s.font_size, 96.0);
        assert_eq!(s.speed, 4.0);
        assert_eq!(s.countdown_secs, 10);
        assert_eq!(s.mode, "notch");
        assert_eq!(s.text_color, "#f5f5f0");
    }

    #[test]
    fn missing_fields_fall_back_to_defaults() {
        let s: Settings = serde_json::from_str(r#"{"speed": 80}"#).unwrap();
        assert_eq!(s.speed, 80.0);
        assert_eq!(s.font_size, Settings::default().font_size);
    }

    #[test]
    fn round_trips_through_disk() {
        let dir = std::env::temp_dir().join(format!("eyeline-settings-{}", std::process::id()));
        let path = dir.join("settings.json");
        let s = Settings {
            speed: 120.0,
            ..Settings::default()
        };
        s.save(&path).unwrap();
        assert_eq!(Settings::load(&path).speed, 120.0);
        let _ = fs::remove_dir_all(dir);
    }
}
