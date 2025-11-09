use std::io::{BufReader, prelude::*};
use std::net::{TcpListener, TcpStream};
use std::fs;
use std::collections::HashMap;

use crate::config::AppConfig;
use crate::static_files::{resolve_content_type, StaticFileResolver};

use serde::{Deserialize, Serialize};

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
            #[serde(rename = "sectionId")]
            section_id: Option<String>,
        }

        let chat_response = ChatResponse {
            response: response_message,
            navigation: ChatNavigation {
                needed: navigation.needed,
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

    fn generate_prompt(message: &str) -> String {
        let html_string = fs::read_to_string("public/index.html").unwrap();
        return format!(r#"You are a helpful assistant for a portfolio website. Respond to the following message: {}
        
Refer to the following HTML string as your reference: {}

IMPORTANT NAVIGATION INSTRUCTIONS:
- The page has the following sections with IDs: home, about, experience, competencies, soft-skills, education, organizations, certificates, awards, contact
- If the user's question requires viewing a specific section of the page, you MUST include a navigation instruction in your response
- Format your response as JSON with two fields:
  1. "response": Your text response to the user
  2. "navigation": An object with "sectionId" (the section ID to navigate to) and "needed" (true/false)
  
Example response format:
{{
  "response": "I can help you with that. Let me navigate to the experience section.",
  "navigation": {{
    "needed": true,
    "sectionId": "experience"
  }}
}}

If navigation is NOT needed, set "needed" to false and "sectionId" to null:
{{
  "response": "Here's the information you requested...",
  "navigation": {{
    "needed": false,
    "sectionId": null
  }}
}}

ALWAYS respond in valid JSON format with both "response" and "navigation" fields."#, message, html_string)
    }
}