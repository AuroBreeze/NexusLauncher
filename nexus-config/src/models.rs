use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// The structure representing the launcher's persistent settings.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct UserConfig {
    /// The username of the current user
    pub user_profile: UserProfiles,

    /// A mapping from username to UUID
    pub username: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct UserProfiles {
    pub offline: UserProfile,
    pub online: UserProfile,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct UserProfile {
    pub username: String,
    pub uuid: String,
}

/// The structure representing the launcher's persistent settings.
#[derive(Debug, Serialize, Deserialize)]
pub struct LaunchConfig {
    /// A mapping from Java major version to its executable path
    /// e.g., 17 = "/usr/lib/jvm/java-17-openjdk/bin/java"
    pub java_paths: HashMap<u32, PathBuf>,

    /// whether to launch the game in offline mode
    pub offline: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_user_config_default() {
        let cfg = UserConfig::default();
        assert!(cfg.user_profile.offline.username.is_empty());
        assert!(cfg.user_profile.offline.uuid.is_empty());
        assert!(cfg.user_profile.online.username.is_empty());
        assert!(cfg.user_profile.online.uuid.is_empty());
        assert!(cfg.username.is_empty());
    }

    #[test]
    fn test_user_config_toml_roundtrip() {
        let cfg = UserConfig {
            user_profile: UserProfiles {
                offline: UserProfile {
                    username: "Player".to_string(),
                    uuid: "abc123".to_string(),
                },
                online: UserProfile {
                    username: "OnlinePlayer".to_string(),
                    uuid: "def456".to_string(),
                },
            },
            username: HashMap::from([("Player".to_string(), "abc123".to_string())]),
        };

        let toml_str = toml::to_string_pretty(&cfg).unwrap();
        let restored: UserConfig = toml::from_str(&toml_str).unwrap();

        assert_eq!(restored.user_profile.offline.username, "Player");
        assert_eq!(restored.user_profile.offline.uuid, "abc123");
        assert_eq!(restored.user_profile.online.username, "OnlinePlayer");
        assert_eq!(restored.user_profile.online.uuid, "def456");
        assert_eq!(restored.username.get("Player").unwrap(), "abc123");
    }

    #[test]
    fn test_launch_config_default() {
        let cfg = LaunchConfig::default();
        assert!(cfg.offline);
        assert!(cfg.java_paths.is_empty());
    }

    #[test]
    fn test_launch_config_toml_roundtrip() {
        let cfg = LaunchConfig {
            offline: false,
            java_paths: HashMap::from([
                (17, PathBuf::from("/usr/lib/jvm/java-17/bin/java")),
                (21, PathBuf::from("/usr/lib/jvm/java-21/bin/java")),
            ]),
        };

        let toml_str = toml::to_string_pretty(&cfg).unwrap();
        let restored: LaunchConfig = toml::from_str(&toml_str).unwrap();

        assert!(!restored.offline);
        assert_eq!(restored.java_paths.len(), 2);
        assert_eq!(
            restored.java_paths.get(&17).unwrap(),
            &PathBuf::from("/usr/lib/jvm/java-17/bin/java")
        );
        assert_eq!(
            restored.java_paths.get(&21).unwrap(),
            &PathBuf::from("/usr/lib/jvm/java-21/bin/java")
        );
    }

    #[test]
    fn test_launch_config_empty_roundtrip() {
        let cfg = LaunchConfig {
            offline: true,
            java_paths: HashMap::new(),
        };
        let toml_str = toml::to_string_pretty(&cfg).unwrap();
        let restored: LaunchConfig = toml::from_str(&toml_str).unwrap();
        assert!(restored.offline);
        assert!(restored.java_paths.is_empty());
    }
}
