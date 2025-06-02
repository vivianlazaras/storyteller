/*!
### Command to generate example config.
```
cargo build
./set_env.sh --generate-config
```
*/

use rocket_oidc::OIDCConfig;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    backend: String,
    name: String,
    user: String,
    host: String,
    port: u16,
    passwordFile: Option<PathBuf>,
}

impl Default for DatabaseConfig {
    fn default() -> DatabaseConfig {
        DatabaseConfig {
            backend: "postgres".to_string(),
            host: "localhost".to_string(),
            user: "storyteller".to_string(),
            port: 5432,
            name: "storyteller".to_string(),
            passwordFile: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    port: u16,
    listen: String,
    url: String,
    ssl: bool,
    certFile: Option<PathBuf>,
    keyFile: Option<PathBuf>,
    oidc: OIDCConfig,
}

impl ServerConfig {
    pub fn endpoint(&self) -> &str {
        &self.url
    }
}

impl Default for ServerConfig {
    fn default() -> ServerConfig {
        ServerConfig {
            port: 8000,
            listen: "localhost".to_string(),
            url: "localhost".to_string(),
            ssl: false,
            certFile: None,
            keyFile: None,
            oidc: OIDCConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APISettings {
    auto_create_users: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIConfig {
    server: ServerConfig,
    db: DatabaseConfig,
}

impl APIConfig {
    pub fn endpoint(&self) -> &str {
        &self.server.endpoint()
    }
}

impl Default for APIConfig {
    fn default() -> APIConfig {
        let mut server = ServerConfig::default();
        server.port = 8442;
        APIConfig {
            server,
            db: DatabaseConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    server: ServerConfig,
    api: APIConfig,
    /// the directory in which to store image files
    images: PathBuf,
}

impl Config {
    pub fn port(&self) -> u16 {
        self.server.port
    }

    pub fn listen(&self) -> &str {
        &self.server.listen
    }

    pub fn api_endpoint(&self) -> &str {
        &self.api.endpoint()
    }

    pub fn oidc(&self) -> &rocket_oidc::OIDCConfig {
        &self.server.oidc
    }
}

impl Default for Config {
    fn default() -> Config {
        let mut server = ServerConfig::default();
        server.port = 8440;
        let mut images = PathBuf::new();
        images.push("/images");
        Config {
            server,
            api: APIConfig::default(),
            images,
        }
    }
}
