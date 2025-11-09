use std::io::{BufReader, prelude::*};
use std::net::{TcpListener, TcpStream};
use std::fs;
use std::collections::HashMap;
use std::path::Path;

use crate::config::AppConfig;
use crate::static_files::{resolve_content_type, StaticFileResolver};

use serde::{Deserialize, Serialize};
use pulldown_cmark::{Parser, Options, html};

pub struct Route {
    
}
pub struct Server {
    pub host: String,
    pub port: String,
    pub address: Option<String>,
    pub listener: Option<TcpListener>,
    pub config: Option<AppConfig>,
    pub resolver: Option<StaticFileResolver>,
}

impl Server {
    pub fn setup_server(&mut self) {
        self.address = Some(format!("{}:{}", self.host, self.port));
        // initialize resolver if config present
        if let Some(cfg) = &self.config {
            match StaticFileResolver::from_config(&cfg.static_cfg) {
                Ok(res) => {
                    self.resolver = Some(res);
                }
                Err(err) => {
                    eprintln!("ERROR initializing resolver: {:?}", err);
                }
            }
        }
        self.setup_listener();
    }

    fn setup_listener(&mut self) {
        match &self.address {
            Some(address) => {
                let listener_result = TcpListener::bind(address);
                match listener_result {
                    Ok(listener) => {
                        println!("Listening to: {}", address);
                        self.listener = Some(listener);
                        self.listen();
                    }
                    Err(error) => {
                        eprintln!("ERROR: {:?}", error);
                    }
                }
            }
            None => {}
        }
    }

    fn listen(&self) {
        match &self.listener {
            Some(listener) => {
                for stream in listener.incoming() {
                    match stream {
                        Ok(tcp_stream) => {
                            self.handle_stream(tcp_stream);
                        }
                        Err(error) => {
                            eprintln!("ERROR listening: {:?}", error);
                        }
                    }
                }
            }
            None => {}
        }
    }

    fn handle_stream(&self, mut tcp_stream: TcpStream) {
        let mut http_request_lines: Vec<String> = Vec::new();
        let mut bad_request = false;
        let mut error_response = b"HTTP/1.1 400 Bad Request\r\n\r\nInvalid Request".to_vec();
        let mut headers_map: HashMap<String, String> = HashMap::new();
        let mut request_body: Option<Vec<u8>> = None;

        // Read headers and body using BufReader
        {
            let mut buf_reader = BufReader::new(&tcp_stream);
            loop {
                let mut line_buf: Vec<u8> = Vec::new();
                
                // Read bytes until a newline
                match buf_reader.read_until(b'\n', &mut line_buf) {
                    Ok(0) => {
                        // Connection closed, 0 bytes read
                        eprintln!("Connection closed while reading headers");
                        bad_request = true;
                        break;
                    }
                    Ok(_) => {
                        // Try to convert the byte line to UTF-8
                        match std::str::from_utf8(&line_buf) {
                            Ok(line_str) => {
                                let trimmed_line = line_str.trim_end(); // Remove \n or \r\n
                                if trimmed_line.is_empty() {
                                    // Empty line signifies end of headers
                                    // Now read the body if Content-Length is present
                                    let content_length = headers_map
                                        .get("content-length")
                                        .and_then(|s| s.parse::<usize>().ok())
                                        .unwrap_or(0);
                                    
                                    if content_length > 0 {
                                        let mut body = vec![0u8; content_length];
                                        match buf_reader.read_exact(&mut body) {
                                            Ok(_) => {
                                                request_body = Some(body);
                                            }
                                            Err(e) => {
                                                eprintln!("Error reading request body: {:?}", e);
                                                bad_request = true;
                                                break;
                                            }
                                        }
                                    }
                                    break;
                                }
                                // Parse headers (skip request line)
                                if !http_request_lines.is_empty() {
                                    if let Some(colon_pos) = trimmed_line.find(':') {
                                        let key = trimmed_line[..colon_pos].trim().to_lowercase();
                                        let value = trimmed_line[colon_pos + 1..].trim().to_string();
                                        headers_map.insert(key, value);
                                    }
                                }
                                http_request_lines.push(trimmed_line.to_string());
                            }
                            Err(e) => {
                                // Handle non-UTF-8 data (e.g., HTTPS handshake)
                                eprintln!("ERROR: Invalid UTF-8 in request header: {:?}", e);
                                error_response =
                                    b"HTTP/1.1 400 Bad Request\r\n\r\nInvalid UTF-8 in request".to_vec();
                                bad_request = true;
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("ERROR reading from stream: {:?}", e);
                        bad_request = true;
                        break;
                    }
                }
            }
        } // `buf_reader` is dropped here

        // Handle any errors found during reading
        if bad_request {
            tcp_stream.write_all(&error_response).ok(); // Ignore write errors
            return;
        }

        if http_request_lines.is_empty() {
            eprintln!("Received empty request");
            tcp_stream.write_all(&error_response).ok();
            return;
        }

        // --- Start of request parsing ---
        let http_header: Vec<&str> = http_request_lines[0].split(' ').collect();

        // Add a check to prevent panic on malformed header
        if http_header.len() < 2 {
            eprintln!("Malformed request line: {}", http_request_lines[0]);
            let response = b"HTTP/1.1 400 Bad Request\r\n\r\nMalformed request line";
            tcp_stream.write_all(response).ok();
            return;
        }

        let method = http_header[0];
        let route = http_header[1];
        println!("METHOD: {}, ROUTE: {}", method, route);
        // --- End of request parsing ---

        // Handle /api/blogs route
        if route == "/api/blogs" && method == "GET" {
            match self.handle_blogs_list_api() {
                Ok(json_response) => {
                    let response_body = json_response.as_bytes();
                    let headers = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n",
                        response_body.len()
                    );
                    let mut response = headers.into_bytes();
                    response.extend_from_slice(response_body);
                    tcp_stream.write_all(&response).ok();
                    return;
                }
                Err(e) => {
                    eprintln!("Error handling blogs list API: {:?}", e);
                    let error_response = format!(
                        "HTTP/1.1 500 Internal Server Error\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{{\"error\":\"{}\"}}",
                        e.len(),
                        e
                    );
                    tcp_stream.write_all(error_response.as_bytes()).ok();
                    return;
                }
            }
        }

        // Handle /api/blog/:slug route
        if route.starts_with("/api/blog/") && method == "GET" {
            let slug = route.strip_prefix("/api/blog/").unwrap_or("");
            match self.handle_blog_post_api(slug) {
                Ok(json_response) => {
                    let response_body = json_response.as_bytes();
                    let headers = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n",
                        response_body.len()
                    );
                    let mut response = headers.into_bytes();
                    response.extend_from_slice(response_body);
                    tcp_stream.write_all(&response).ok();
                    return;
                }
                Err(e) => {
                    eprintln!("Error handling blog post API: {:?}", e);
                    let error_response = format!(
                        "HTTP/1.1 500 Internal Server Error\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{{\"error\":\"{}\"}}",
                        e.len(),
                        e
                    );
                    tcp_stream.write_all(error_response.as_bytes()).ok();
                    return;
                }
            }
        }

        // Handle /blogs/:slug route - serve individual blog posts as HTML pages
        if route.starts_with("/blogs/") && method == "GET" {
            let slug = route.strip_prefix("/blogs/").unwrap_or("");
            // Don't treat /blogs as a slug (it should be handled by static file resolver)
            if !slug.is_empty() {
                match self.handle_blog_post_page(slug) {
                    Ok(html_response) => {
                        let response_body = html_response.as_bytes();
                        let headers = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n",
                            response_body.len()
                        );
                        let mut response = headers.into_bytes();
                        response.extend_from_slice(response_body);
                        tcp_stream.write_all(&response).ok();
                        return;
                    }
                    Err(e) => {
                        eprintln!("Error handling blog post page: {:?}", e);
                        let error_response = format!(
                            "HTTP/1.1 404 Not Found\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n<html><body><h1>Blog Post Not Found</h1><p>{}</p></body></html>",
                            e.len() + 50,
                            e
                        );
                        tcp_stream.write_all(error_response.as_bytes()).ok();
                        return;
                    }
                }
            }
        }

        // Handle /api/chat route
        if route == "/api/chat" && method == "POST" {
            if let Some(body) = request_body {
                println!("Content Length: {}", body.len());
                println!("Body: {:?}", body);
                match self.handle_chat_api(&body) {
                    Ok(json_response) => {
                        let response_body = json_response.as_bytes();
                        let headers = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n",
                            response_body.len()
                        );
                        let mut response = headers.into_bytes();
                        response.extend_from_slice(response_body);
                        tcp_stream.write_all(&response).ok();
                        return;
                    }
                    Err(e) => {
                        eprintln!("Error handling chat API: {:?}", e);
                        let error_response = format!(
                            "HTTP/1.1 500 Internal Server Error\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{{\"error\":\"{}\"}}",
                            e.len(),
                            e
                        );
                        tcp_stream.write_all(error_response.as_bytes()).ok();
                        return;
                    }
                }
            } else {
                let response = b"HTTP/1.1 400 Bad Request\r\n\r\nMissing request body";
                tcp_stream.write_all(response).ok();
                return;
            }
        }

        // --- Start of your original file-serving logic ---
        
        // Default response
        let mut response: Vec<u8> =
            b"HTTP/1.1 404 ERROR\r\nContent-Length: 13\r\n\r\nFile Not Found".to_vec();

        if let Some(cfg) = &self.config {
            if let Some(resolver) = &self.resolver {
                match resolver.resolve(route) {
                    Ok(path) => {
                        match fs::read(&path) {
                            Ok(bytes) => {
                                // File found, build 200 OK response
                                let content_type = resolve_content_type(&path, &cfg.content_types);
                                let headers = format!(
                                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: {}\r\n\r\n",
                                    bytes.len(),
                                    content_type
                                );
                                let mut buf = headers.into_bytes();
                                buf.extend_from_slice(&bytes);
                                response = buf;
                            }
                            Err(_e) => {
                                // File read error, default 404 response is already set
                                eprintln!("File not found or unreadable: {:?}", path);
                            }
                        }
                    }
                    Err(_e) => {
                        // Route not resolved, default 404 response is already set
                        eprintln!("Route not resolved: {}", route);
                    }
                }
            }
        }

        // Write the final response back to the stream
        match tcp_stream.write_all(&response) {
            Ok(_result) => {}
            Err(error) => {
                eprintln!("ERROR writing response: {:?}", error)
            }
        };        
    }

    fn handle_chat_api(&self, body: &[u8]) -> Result<String, String> {
        println!("Handling chat API");
        println!("Body: {:?}", body);
        // Parse request JSON
        #[derive(Deserialize)]
        struct ChatRequest {
            message: String,
        }

        let request: ChatRequest = serde_json::from_slice(body)
            .map_err(|e| format!("Failed to parse request JSON: {}", e))?;

        // Get API key from environment variable
        let api_key = std::env::var("GEMINI_API_KEY")
            .map_err(|_| "GEMINI_API_KEY environment variable not set".to_string())?;

        // Prepare Gemini API request
        #[derive(Serialize, Debug)]
        struct GeminiRequest {
            contents: Vec<GeminiContent>,
        }

        #[derive(Serialize, Debug)]
        struct GeminiContent {
            parts: Vec<GeminiPart>,
        }

        #[derive(Serialize, Debug)]
        struct GeminiPart {
            text: String,
        }

        let gemini_request = GeminiRequest {
            contents: vec![GeminiContent {
                parts: vec![GeminiPart {
                    text: Server::generate_prompt(&request.message).to_string(),
                }],
            }],
        };

        println!("Gemini Request: {:?}", gemini_request);

        // Make request to Google Gemini API
        println!("Making request to Google Gemini API");
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash-lite:generateContent?key={}",
            api_key
        );

        let response = ureq::post(&url)
            .set("Content-Type", "application/json")
            .send_json(&gemini_request)
            .map_err(|e| format!("Failed to call Gemini API: {}", e))?;

        println!("Gemini Response: {:?}", response);

        // Parse Gemini response
        #[derive(Deserialize, Debug)]
        struct GeminiResponse {
            candidates: Vec<GeminiCandidate>,
        }

        #[derive(Deserialize, Debug)]
        struct GeminiCandidate {
            content: GeminiContentResponse,
        }

        #[derive(Deserialize, Debug)]
        struct GeminiContentResponse {
            parts: Vec<GeminiPartResponse>,
        }

        #[derive(Deserialize, Debug)]
        struct GeminiPartResponse {
            text: String,
        }

        let gemini_response: GeminiResponse = response
            .into_json()
            .map_err(|e| format!("Failed to parse Gemini response: {}", e))?;

        println!("Gemini Response: {:?}", gemini_response);

        // Extract the text response
        let response_text = gemini_response
            .candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone())
            .ok_or_else(|| "No response text in Gemini API response".to_string())?;

        // Parse the JSON response from Gemini
        #[derive(Deserialize)]
        struct GeminiParsedResponse {
            response: String,
            navigation: NavigationData,
        }

        #[derive(Deserialize)]
        struct NavigationData {
            needed: bool,
            #[serde(rename = "page")]
            page: Option<String>,
            #[serde(rename = "sectionId")]
            section_id: Option<String>,
        }

        // Extract JSON from markdown code blocks if present
        let json_text = Server::extract_json_from_markdown(&response_text);
        
        // Try to parse as JSON, fallback to plain text if parsing fails
        let parsed: Result<GeminiParsedResponse, _> = serde_json::from_str(&json_text);
        
        let (response_message, navigation) = match parsed {
            Ok(parsed_response) => {
                // Successfully parsed JSON with navigation data
                (parsed_response.response, parsed_response.navigation)
            }
            Err(e) => {
                // Fallback: treat as plain text response with no navigation
                eprintln!("Failed to parse JSON response: {:?}, text: {}", e, json_text);
                (response_text, NavigationData {
                    needed: false,
                    page: None,
                    section_id: None,
                })
            }
        };

        // Return JSON response for chatbar.js
        #[derive(Serialize)]
        struct ChatResponse {
            response: String,
            navigation: ChatNavigation,
        }

        #[derive(Serialize)]
        struct ChatNavigation {
            needed: bool,
            #[serde(rename = "page")]
            page: Option<String>,
            #[serde(rename = "sectionId")]
            section_id: Option<String>,
        }

        let chat_response = ChatResponse {
            response: response_message,
            navigation: ChatNavigation {
                needed: navigation.needed,
                page: navigation.page,
                section_id: navigation.section_id,
            },
        };

        serde_json::to_string(&chat_response)
            .map_err(|e| format!("Failed to serialize response: {}", e))
    }

    fn extract_json_from_markdown(text: &str) -> String {
        let trimmed = text.trim();
        
        // Check if wrapped in markdown code blocks (```json or just ```)
        if trimmed.starts_with("```") {
            // Find the first newline after ```
            if let Some(start_idx) = trimmed.find('\n') {
                let after_lang = &trimmed[start_idx + 1..];
                // Find the closing ``` (search from the end)
                if let Some(end_idx) = after_lang.rfind("```") {
                    return after_lang[..end_idx].trim().to_string();
                }
            }
        }
        
        // If no code blocks found, return as-is
        trimmed.to_string()
    }

    fn handle_blogs_list_api(&self) -> Result<String, String> {
        let blogs_dir = Path::new("public/blogs");
        
        if !blogs_dir.exists() {
            return Ok(r#"{"blogs":[]}"#.to_string());
        }

        let entries = fs::read_dir(blogs_dir)
            .map_err(|e| format!("Failed to read blogs directory: {}", e))?;

        let mut blogs: Vec<BlogMetadata> = Vec::new();

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();
            
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                if let Some(file_name) = path.file_stem().and_then(|s| s.to_str()) {
                    let slug = file_name.to_string();
                    let content = fs::read_to_string(&path)
                        .map_err(|e| format!("Failed to read blog file: {}", e))?;
                    
                    // Extract title and date from markdown (first line is usually title)
                    let title = content.lines()
                        .next()
                        .unwrap_or(&slug)
                        .trim_start_matches('#')
                        .trim()
                        .to_string();
                    
                    // Try to extract date from content (look for **Published:** pattern)
                    let published_date = content.lines()
                        .find(|line| line.contains("**Published:**") || line.contains("Published:"))
                        .and_then(|line| {
                            line.split("Published:").nth(1)
                                .or_else(|| line.split("**Published:**").nth(1))
                                .map(|s| s.trim().trim_matches('*').trim().to_string())
                        })
                        .unwrap_or_else(|| "Unknown".to_string());

                    blogs.push(BlogMetadata {
                        slug,
                        title,
                        published_date,
                    });
                }
            }
        }

        // Sort by published date (most recent first)
        blogs.sort_by(|a, b| b.published_date.cmp(&a.published_date));

        #[derive(Serialize)]
        struct BlogListResponse {
            blogs: Vec<BlogMetadata>,
        }

        #[derive(Serialize)]
        struct BlogMetadata {
            slug: String,
            title: String,
            published_date: String,
        }

        let response = BlogListResponse { blogs };
        serde_json::to_string(&response)
            .map_err(|e| format!("Failed to serialize blog list: {}", e))
    }

    fn handle_blog_post_api(&self, slug: &str) -> Result<String, String> {
        let blog_path = Path::new("public/blogs").join(format!("{}.md", slug));
        
        if !blog_path.exists() {
            return Err(format!("Blog post '{}' not found", slug));
        }

        let markdown_content = fs::read_to_string(&blog_path)
            .map_err(|e| format!("Failed to read blog file: {}", e))?;

        // Parse markdown to HTML
        let parser = Parser::new_ext(&markdown_content, Options::all());
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        // Extract metadata
        let title = markdown_content.lines()
            .next()
            .unwrap_or(slug)
            .trim_start_matches('#')
            .trim()
            .to_string();
        
        let published_date = markdown_content.lines()
            .find(|line| line.contains("**Published:**") || line.contains("Published:"))
            .and_then(|line| {
                line.split("Published:").nth(1)
                    .or_else(|| line.split("**Published:**").nth(1))
                    .map(|s| s.trim().trim_matches('*').trim().to_string())
            })
            .unwrap_or_else(|| "Unknown".to_string());

        #[derive(Serialize)]
        struct BlogPostResponse {
            slug: String,
            title: String,
            published_date: String,
            content: String,
        }

        let response = BlogPostResponse {
            slug: slug.to_string(),
            title,
            published_date,
            content: html_output,
        };

        serde_json::to_string(&response)
            .map_err(|e| format!("Failed to serialize blog post: {}", e))
    }

    fn handle_blog_post_page(&self, slug: &str) -> Result<String, String> {
        let blog_path = Path::new("public/blogs").join(format!("{}.md", slug));
        
        if !blog_path.exists() {
            return Err(format!("Blog post '{}' not found", slug));
        }

        let markdown_content = fs::read_to_string(&blog_path)
            .map_err(|e| format!("Failed to read blog file: {}", e))?;

        // Parse markdown to HTML
        let parser = Parser::new_ext(&markdown_content, Options::all());
        let mut html_content = String::new();
        html::push_html(&mut html_content, parser);

        // Extract metadata
        let title = markdown_content.lines()
            .next()
            .unwrap_or(slug)
            .trim_start_matches('#')
            .trim()
            .to_string();
        
        let published_date = markdown_content.lines()
            .find(|line| line.contains("**Published:**") || line.contains("Published:"))
            .and_then(|line| {
                line.split("Published:").nth(1)
                    .or_else(|| line.split("**Published:**").nth(1))
                    .map(|s| s.trim().trim_matches('*').trim().to_string())
            })
            .unwrap_or_else(|| "Unknown".to_string());

        // Generate HTML page
        let html_page = format!(r#"<!DOCTYPE html>
<html lang="en">
    <head>
        <meta charset="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <title>{} — Blogs — Will Vincent Parrone</title>
        <link rel="stylesheet" href="/index.css" />
        <link rel="stylesheet" href="/chatbar.css" />
    </head>
    <body>
        <header class="site-header">
            <nav class="nav" aria-label="Primary">
                <a class="brand" href="/">WVP</a>
                <button class="nav-toggle" aria-expanded="false" aria-controls="nav-menu">Menu</button>
                <ul id="nav-menu" class="nav-menu">
                    <li><a href="/">Home</a></li>
                    <li><a href="/blogs">Blogs</a></li>
                </ul>
            </nav>
        </header>

        <main>
            <div class="container" style="padding: 3rem 0;">
                <div class="blog-post">
                    <button id="blog-back" class="blog-back" aria-label="Back to blogs">← Back to Blogs</button>
                    <article class="blog-content">
                        <header class="blog-header">
                            <h1>{}</h1>
                            <div class="blog-meta">Published: {}</div>
                        </header>
                        <div class="blog-body">{}</div>
                    </article>
                </div>
            </div>
        </main>

        <footer class="site-footer">
            <div class="container">
                <small>© <span id="year"></span> Will Vincent Parrone</small>
            </div>
        </footer>

        <!-- Chatbar Component -->
        <div id="chatbar" class="chatbar">
            <button class="chatbar-toggle" aria-label="Toggle chat" aria-expanded="false">
                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
                </svg>
            </button>
            <div class="chatbar-panel">
                <div class="chatbar-header">
                    <h3>Chat</h3>
                    <button class="chatbar-close" aria-label="Close chat">
                        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <line x1="18" y1="6" x2="6" y2="18"/>
                            <line x1="6" y1="6" x2="18" y2="18"/>
                        </svg>
                    </button>
                </div>
                <div class="chatbar-messages" id="chatbar-messages">
                    <div class="chatbar-message chatbar-message-system">
                        <p>Hello! How can I help you today?</p>
                    </div>
                </div>
                <div class="chatbar-input-container">
                    <form id="chatbar-form" class="chatbar-form">
                        <input 
                            type="text" 
                            id="chatbar-input" 
                            class="chatbar-input" 
                            placeholder="Type your message..." 
                            autocomplete="off"
                            aria-label="Message input"
                        />
                        <button type="submit" class="chatbar-send" aria-label="Send message">
                            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                <line x1="22" y1="2" x2="11" y2="13"/>
                                <polygon points="22 2 15 22 11 13 2 9 22 2"/>
                            </svg>
                        </button>
                    </form>
                </div>
            </div>
        </div>

        <script src="/chatbar.js"></script>
        <script>
            const yearEl = document.getElementById('year');
            if (yearEl) {{ yearEl.textContent = new Date().getFullYear(); }}
            const toggle = document.querySelector('.nav-toggle');
            const menu = document.getElementById('nav-menu');
            if (toggle && menu) {{
                toggle.addEventListener('click', () => {{
                    const open = menu.classList.toggle('open');
                    toggle.setAttribute('aria-expanded', String(open));
                }});
            }}

            const blogBackBtn = document.getElementById('blog-back');
            if (blogBackBtn) {{
                blogBackBtn.addEventListener('click', () => {{
                    window.location.href = '/blogs';
                }});
            }}
        </script>
    </body>
</html>"#, title, title, published_date, html_content);

        Ok(html_page)
    }

    fn generate_prompt(message: &str) -> String {
        // Read pages.json for page and section summaries
        let pages_json = fs::read_to_string("pages.json")
            .unwrap_or_else(|_| r#"{"pages":{}}"#.to_string());
        
        return format!(r#"You are a helpful assistant for a portfolio website. Respond to the following message: {}
        
Refer to the following pages and sections summary as your reference: {}

IMPORTANT NAVIGATION INSTRUCTIONS:
- Available pages: "index" (home page at /), "blogs" (blogs page at /blogs)
- The index page has the following sections with IDs: home, about, experience, competencies, soft-skills, education, organizations, certificates, awards, contact
- The blogs page has a listing section and individual blog posts
- Individual blog posts are accessible at /blogs/:slug (e.g., /blogs/welcome-to-my-blog, /blogs/getting-started-with-rust)
- If the user's question requires viewing a specific page, section, or blog post, you MUST include a navigation instruction in your response
- Format your response as JSON with two fields:
  1. "response": Your text response to the user
  2. "navigation": An object with "page" (the page to navigate to: "index", "blogs", or a blog post URL like "/blogs/welcome-to-my-blog"), "sectionId" (the section ID to navigate to, if applicable), and "needed" (true/false)
  
Example response formats:
{{
  "response": "I can help you with that. Let me navigate to the experience section.",
  "navigation": {{
    "needed": true,
    "page": "index",
    "sectionId": "experience"
  }}
}}

{{
  "response": "Let me show you the blogs page.",
  "navigation": {{
    "needed": true,
    "page": "blogs",
    "sectionId": null
  }}
}}

{{
  "response": "I'll navigate to the contact section for you.",
  "navigation": {{
    "needed": true,
    "page": "/",
    "sectionId": "contact"
  }}
}}

{{
  "response": "Let me show you the blog post about getting started with Rust.",
  "navigation": {{
    "needed": true,
    "page": "/blogs/getting-started-with-rust",
    "sectionId": null
  }}
}}

If navigation is NOT needed, set "needed" to false and both "page" and "sectionId" to null:
{{
  "response": "Here's the information you requested...",
  "navigation": {{
    "needed": false,
    "page": null,
    "sectionId": null
  }}
}}

ALWAYS respond in valid JSON format with both "response" and "navigation" fields."#, message, pages_json)
    }
}