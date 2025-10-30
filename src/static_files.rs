use std::path::{Path, PathBuf};
use std::fmt;

use crate::config::StaticConfig;

#[derive(Debug)]
pub struct StaticFileResolver {
    root_dir: PathBuf,
    index_file: String,
    auto_index: bool,
    routes: std::collections::HashMap<String, String>,
}

#[derive(Debug)]
pub enum ResolveError {
    Forbidden,
    NotFound,
}

impl std::error::Error for ResolveError {}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ResolveError::Forbidden => write!(f, "forbidden"),
            ResolveError::NotFound => write!(f, "not found"),
        }
    }
}

impl StaticFileResolver {
    pub fn from_config(cfg: &StaticConfig) -> Result<Self, std::io::Error> {
        let root_dir = Path::new(&cfg.root_dir).to_path_buf();
        Ok(Self {
            root_dir,
            index_file: cfg.index_file.clone(),
            auto_index: cfg.auto_index,
            routes: cfg.routes.clone(),
        })
    }

    pub fn resolve(&self, url_path: &str) -> Result<PathBuf, ResolveError> {
        // Normalize url path
        let mut normalized = if url_path.is_empty() { "/".to_string() } else { url_path.to_string() };
        if !normalized.starts_with('/') { normalized = format!("/{}", normalized); }

        // Route override
        if let Some(rel) = self.routes.get(&normalized) {
            let candidate = self.root_dir.join(rel);
            return self.validate_within_root(candidate);
        }

        // Filesystem mapping
        let joined = self.root_dir.join(normalized.trim_start_matches('/'));
        // If it's a directory or ends with '/':
        let candidate = if normalized.ends_with('/') || joined.is_dir() {
            if self.auto_index {
                joined.join(&self.index_file)
            } else {
                joined
            }
        } else {
            joined
        };

        self.validate_within_root(candidate)
    }

    fn validate_within_root(&self, candidate: PathBuf) -> Result<PathBuf, ResolveError> {
        let root_canon = match std::fs::canonicalize(&self.root_dir) {
            Ok(p) => p,
            Err(_) => return Err(ResolveError::NotFound),
        };
        let cand_canon = match std::fs::canonicalize(&candidate) {
            Ok(p) => p,
            Err(_) => return Err(ResolveError::NotFound),
        };
        if !cand_canon.starts_with(&root_canon) {
            return Err(ResolveError::Forbidden);
        }
        Ok(cand_canon)
    }
}

pub fn resolve_content_type(path: &Path, overrides: &std::collections::HashMap<String, String>) -> String {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        if let Some(ct) = overrides.get(&format!(".{}", ext)) {
            return ct.clone();
        }
        match ext {
            "html" => "text/html",
            "css" => "text/css",
            "js" => "application/javascript",
            "json" => "application/json",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "svg" => "image/svg+xml",
            "ico" => "image/x-icon",
            _ => "application/octet-stream",
        }.to_string()
    } else {
        "application/octet-stream".to_string()
    }
}

