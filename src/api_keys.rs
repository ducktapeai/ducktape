use crate::security::ApiKey;
use secrecy::ExposeSecret;
use std::sync::Arc;

#[derive(Clone)]
pub struct ApiKeys {
    openai: Option<Arc<ApiKey>>,
    xai: Option<Arc<ApiKey>>,
    deepseek: Option<Arc<ApiKey>>,
    zoom_account: Option<Arc<ApiKey>>,
    zoom_client: Option<Arc<ApiKey>>,
    zoom_secret: Option<Arc<ApiKey>>,
}

impl Default for ApiKeys {
    fn default() -> Self {
        Self::new()
    }
}

impl ApiKeys {
    pub fn new() -> Self {
        Self {
            openai: std::env::var("OPENAI_API_KEY").ok().map(|k| Arc::new(ApiKey::new(k))),
            xai: std::env::var("XAI_API_KEY").ok().map(|k| Arc::new(ApiKey::new(k))),
            deepseek: std::env::var("DEEPSEEK_API_KEY").ok().map(|k| Arc::new(ApiKey::new(k))),
            zoom_account: std::env::var("ZOOM_ACCOUNT_ID").ok().map(|k| Arc::new(ApiKey::new(k))),
            zoom_client: std::env::var("ZOOM_CLIENT_ID").ok().map(|k| Arc::new(ApiKey::new(k))),
            zoom_secret: std::env::var("ZOOM_CLIENT_SECRET").ok().map(|k| Arc::new(ApiKey::new(k))),
        }
    }

    pub fn has_openai(&self) -> bool {
        self.openai.is_some()
    }

    pub fn has_xai(&self) -> bool {
        self.xai.is_some()
    }

    pub fn has_deepseek(&self) -> bool {
        self.deepseek.is_some()
    }

    pub fn has_zoom(&self) -> bool {
        self.zoom_account.is_some() && self.zoom_client.is_some() && self.zoom_secret.is_some()
    }

    pub fn openai(&self) -> Option<&str> {
        self.openai.as_ref().map(|k| k.expose())
    }

    pub fn xai(&self) -> Option<&str> {
        self.xai.as_ref().map(|k| k.expose())
    }

    pub fn deepseek(&self) -> Option<&str> {
        self.deepseek.as_ref().map(|k| k.expose())
    }

    pub fn zoom_credentials(&self) -> Option<(String, String, String)> {
        match (
            self.zoom_account.as_ref(),
            self.zoom_client.as_ref(),
            self.zoom_secret.as_ref(),
        ) {
            (Some(account), Some(client), Some(secret)) => Some((
                account.expose().to_string(),
                client.expose().to_string(),
                secret.expose().to_string(),
            )),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_api_keys() {
        env::set_var("OPENAI_API_KEY", "test_openai_key");
        env::set_var("XAI_API_KEY", "test_xai_key");
        
        let keys = ApiKeys::new();
        assert!(keys.has_openai());
        assert!(keys.has_xai());
        assert!(!keys.has_deepseek());
        assert!(!keys.has_zoom());
        
        assert_eq!(keys.openai(), Some("test_openai_key"));
        assert_eq!(keys.xai(), Some("test_xai_key"));
        assert_eq!(keys.deepseek(), None);
    }
}