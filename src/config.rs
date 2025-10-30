use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::fmt;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub static_cfg: StaticConfig,
    pub content_types: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone)]
pub struct StaticConfig {
    pub root_dir: String,
    pub index_file: String,
    pub auto_index: bool,
    pub routes: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Parse(String),
    Invalid(String),
}

impl std::error::Error for ConfigError {}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::Io(e) => write!(f, "io error: {}", e),
            ConfigError::Parse(s) => write!(f, "parse error: {}", s),
            ConfigError::Invalid(s) => write!(f, "invalid configuration: {}", s),
        }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::Io(err)
    }
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<AppConfig, ConfigError> {
    let raw = fs::read_to_string(&path)?;
    let cfg = parse_yaml_config(&raw)?;

    // Basic validation
    if cfg.server.host.trim().is_empty() {
        return Err(ConfigError::Invalid("server.host cannot be empty".to_string()));
    }

    if cfg.static_cfg.root_dir.trim().is_empty() {
        return Err(ConfigError::Invalid("static.root_dir cannot be empty".to_string()));
    }

    if cfg.static_cfg.index_file.trim().is_empty() {
        return Err(ConfigError::Invalid("static.index_file cannot be empty".to_string()));
    }

    Ok(cfg)
}

fn parse_yaml_config(content: &str) -> Result<AppConfig, ConfigError> {
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    
    let mut server_config = ServerConfig {
        host: String::new(),
        port: 8080,
    };
    
    let mut static_config = StaticConfig {
        root_dir: String::new(),
        index_file: String::new(),
        auto_index: false,
        routes: HashMap::new(),
    };
    
    let mut content_types = HashMap::new();
    
    while i < lines.len() {
        let line = lines[i].trim();
        
        if line.starts_with("server:") {
            i += 1;
            while i < lines.len() && lines[i].starts_with("  ") {
                let sub_line = lines[i].trim();
                if sub_line.starts_with("host:") {
                    server_config.host = extract_value(sub_line)?;
                } else if sub_line.starts_with("port:") {
                    server_config.port = extract_value(sub_line)?.parse()
                        .map_err(|_| ConfigError::Parse("Invalid port number".to_string()))?;
                }
                i += 1;
            }
        } else if line.starts_with("static:") {
            i += 1;
            while i < lines.len() && lines[i].starts_with("  ") {
                let sub_line = lines[i].trim();
                if sub_line.starts_with("root_dir:") {
                    static_config.root_dir = extract_value(sub_line)?;
                } else if sub_line.starts_with("index_file:") {
                    static_config.index_file = extract_value(sub_line)?;
                } else if sub_line.starts_with("auto_index:") {
                    static_config.auto_index = extract_value(sub_line)?.to_lowercase() == "true";
                } else if sub_line.starts_with("routes:") {
                    i += 1;
                    while i < lines.len() && lines[i].starts_with("    ") {
                        let route_line = lines[i].trim();
                        if let Some((key, value)) = parse_key_value(route_line) {
                            static_config.routes.insert(key, value);
                        }
                        i += 1;
                    }
                    i -= 1; // Adjust for the outer loop increment
                }
                i += 1;
            }
        } else if line.starts_with("content_types:") {
            i += 1;
            while i < lines.len() && lines[i].starts_with("  ") {
                let type_line = lines[i].trim();
                if let Some((key, value)) = parse_key_value(type_line) {
                    content_types.insert(key, value);
                }
                i += 1;
            }
        }
        i += 1;
    }
    
    Ok(AppConfig {
        server: server_config,
        static_cfg: static_config,
        content_types,
    })
}

fn extract_value(line: &str) -> Result<String, ConfigError> {
    if let Some(colon_pos) = line.find(':') {
        let value = line[colon_pos + 1..].trim();
        if value.starts_with('"') && value.ends_with('"') {
            Ok(value[1..value.len()-1].to_string())
        } else {
            Ok(value.to_string())
        }
    } else {
        Err(ConfigError::Parse("Invalid key-value format".to_string()))
    }
}

fn parse_key_value(line: &str) -> Option<(String, String)> {
    if let Some(colon_pos) = line.find(':') {
        let key = line[..colon_pos].trim();
        let value = line[colon_pos + 1..].trim();
        let clean_value = if value.starts_with('"') && value.ends_with('"') {
            value[1..value.len()-1].to_string()
        } else {
            value.to_string()
        };
        Some((key.to_string(), clean_value))
    } else {
        None
    }
}

