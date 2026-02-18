use rust_embed::Embed;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Embed)]
#[folder = "locales/"]
struct Asset;

pub struct TranslationService {
    // lang_code -> (key -> message)
    data: HashMap<String, HashMap<String, String>>,
    default_locale: String,
    fallback_locale: String,
}

use crate::core::services::I18nService;

impl I18nService for TranslationService {
    fn get(&self, key: &str, lang: &str) -> String {
        self.get(key, lang)
    }

    fn supported_languages(&self) -> Vec<String> {
        self.supported_languages()
    }
}

impl TranslationService {
    pub fn new(default_locale: String, fallback_locale: String) -> Self {
        let mut data = HashMap::new();
        
        // Load all locale files
        let locales = vec!["en", "hi", "es", "fr"];
        
        for locale in locales {
            let filename = format!("{}.json", locale);
            if let Some(file) = Asset::get(&filename) {
                if let Ok(content) = std::str::from_utf8(file.data.as_ref()) {
                    if let Ok(json) = serde_json::from_str::<Value>(content) {
                        let mut flat_map = HashMap::new();
                        flatten_json("", &json, &mut flat_map);
                        data.insert(locale.to_string(), flat_map);
                    }
                }
            }
        }
        
        Self { data, default_locale, fallback_locale }
    }

    /// Get a translated message by key and language
    /// Keys can be nested like "errors.not_found" or "auth.login_success"
    pub fn get(&self, key: &str, lang: &str) -> String {
        // Try requested language first
        if let Some(messages) = self.data.get(lang) {
            if let Some(msg) = messages.get(key) {
                return msg.clone();
            }
        }
        // Fallback 1: Configured default locale
        if lang != self.default_locale {
            if let Some(messages) = self.data.get(&self.default_locale) {
                if let Some(msg) = messages.get(key) {
                    return msg.clone();
                }
            }
        }
        // Fallback 2: Configured fallback locale
        if lang != self.fallback_locale && self.default_locale != self.fallback_locale {
             if let Some(messages) = self.data.get(&self.fallback_locale) {
                if let Some(msg) = messages.get(key) {
                    return msg.clone();
                }
            }
        }
        
        // Last resort: Key itself
        key.to_string()
    }
    
    /// Get list of supported languages
    pub fn supported_languages(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }
}

/// Flatten nested JSON into dot-notation keys
/// Example: {"errors": {"not_found": "..."}} becomes {"errors.not_found": "..."}
fn flatten_json(prefix: &str, value: &Value, output: &mut HashMap<String, String>) {
    match value {
        Value::Object(map) => {
            for (key, val) in map {
                let new_prefix = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", prefix, key)
                };
                flatten_json(&new_prefix, val, output);
            }
        }
        Value::String(s) => {
            output.insert(prefix.to_string(), s.clone());
        }
        _ => {}
    }
}
