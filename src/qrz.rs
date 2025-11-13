use crate::config::QrzConfig;
use anyhow::{Context, Result};
use qrz_xml::{ApiVersion, QrzXmlClient};
use tracing::{debug, info};

pub struct QrzClient {
    client: QrzXmlClient,
}

#[derive(Debug, Clone)]
pub struct CallsignInfo {
    pub fname: Option<String>,
    pub name: Option<String>,
    pub nickname: Option<String>,
}

impl QrzClient {
    /// Create a new QRZ client and authenticate with credentials
    pub async fn new(config: &QrzConfig) -> Result<Self> {
        info!("Initializing QRZ XML API client");

        let client = QrzXmlClient::new(&config.username, &config.password, ApiVersion::Current)
            .context("Failed to create QRZ client and authenticate")?;

        info!("Successfully authenticated with QRZ.com");

        Ok(Self { client })
    }

    /// Lookup a callsign and retrieve name information
    pub async fn lookup_callsign(&self, callsign: &str) -> Result<CallsignInfo> {
        debug!("Looking up callsign: {}", callsign);

        let record = self
            .client
            .lookup_callsign(callsign)
            .await
            .context("Failed to lookup callsign")?;

        let info = CallsignInfo {
            fname: record.fname,
            name: record.name,
            nickname: record.nickname,
        };

        debug!("QRZ lookup result for {}: {:?}", callsign, info);

        Ok(info)
    }

    /// Get the best display name from QRZ data
    /// Prioritizes: nickname > fname > name
    pub fn get_display_name(info: &CallsignInfo) -> Option<String> {
        if let Some(nickname) = &info.nickname {
            if !nickname.is_empty() {
                return Some(nickname.clone());
            }
        }

        if let Some(fname) = &info.fname {
            if !fname.is_empty() {
                return Some(fname.clone());
            }
        }

        if let Some(name) = &info.name {
            if !name.is_empty() {
                return Some(name.clone());
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_name_nickname_priority() {
        let info = CallsignInfo {
            fname: Some("John".to_string()),
            name: Some("Smith".to_string()),
            nickname: Some("Jay".to_string()),
        };
        assert_eq!(QrzClient::get_display_name(&info), Some("Jay".to_string()));
    }

    #[test]
    fn test_display_name_fname_fallback() {
        let info = CallsignInfo {
            fname: Some("John".to_string()),
            name: Some("Smith".to_string()),
            nickname: None,
        };
        assert_eq!(QrzClient::get_display_name(&info), Some("John".to_string()));
    }

    #[test]
    fn test_display_name_name_fallback() {
        let info = CallsignInfo {
            fname: None,
            name: Some("Smith".to_string()),
            nickname: None,
        };
        assert_eq!(
            QrzClient::get_display_name(&info),
            Some("Smith".to_string())
        );
    }

    #[test]
    fn test_display_name_empty() {
        let info = CallsignInfo {
            fname: None,
            name: None,
            nickname: None,
        };
        assert_eq!(QrzClient::get_display_name(&info), None);
    }

    #[test]
    fn test_display_name_empty_strings() {
        let info = CallsignInfo {
            fname: Some("".to_string()),
            name: Some("".to_string()),
            nickname: Some("".to_string()),
        };
        assert_eq!(QrzClient::get_display_name(&info), None);
    }
}
