use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    HEAD,
    OPTIONS,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::GET => write!(f, "GET"),
            HttpMethod::POST => write!(f, "POST"),
            HttpMethod::PUT => write!(f, "PUT"),
            HttpMethod::PATCH => write!(f, "PATCH"),
            HttpMethod::DELETE => write!(f, "DELETE"),
            HttpMethod::HEAD => write!(f, "HEAD"),
            HttpMethod::OPTIONS => write!(f, "OPTIONS"),
        }
    }
}

impl std::str::FromStr for HttpMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::GET),
            "POST" => Ok(HttpMethod::POST),
            "PUT" => Ok(HttpMethod::PUT),
            "PATCH" => Ok(HttpMethod::PATCH),
            "DELETE" => Ok(HttpMethod::DELETE),
            "HEAD" => Ok(HttpMethod::HEAD),
            "OPTIONS" => Ok(HttpMethod::OPTIONS),
            _ => Err(format!("Método HTTP no válido: {}. Use GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpHeader {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryParameter {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRequest {
    pub method: HttpMethod,
    pub endpoint: String,
    pub headers: Vec<HttpHeader>,
    pub query_params: Vec<QueryParameter>,
    pub body: Option<String>, // JSON string
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuite {
    pub name: String,
    pub base_url: String,
    pub requests: Vec<TestRequest>,
    pub iterations: u32,
    pub concurrent_requests: u32,
    pub wait_time: u64,
    pub output_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_type: TestType,
    pub request_name: String,
    pub iteration: u32,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub duration_ms: u64,
    pub success: bool,
    pub status_code: Option<u16>,
    pub response: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestType {
    Single,
    Suite,
}

impl std::fmt::Display for TestType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestType::Single => write!(f, "Single"),
            TestType::Suite => write!(f, "Suite"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSummary {
    pub test_type: TestType,
    pub request_name: String,
    pub total_requests: u32,
    pub successful_requests: u32,
    pub failed_requests: u32,
    pub total_duration_ms: u64,
    pub average_duration_ms: f64,
    pub min_duration_ms: u64,
    pub max_duration_ms: u64,
    pub success_rate: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>, // Hora fija de cuando se ejecutó el test
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedConfig {
    pub name: String,
    pub base_url: String,
    pub requests: Vec<TestRequest>,
    // Parámetros adicionales de configuración
    pub iterations: u32,
    pub concurrent_requests: u32,
    pub wait_time: u64,
    pub output_dir: String,
    pub auto_generate_report: bool,
    pub auto_upload_report: bool,
    pub remote_folder_path: String,
    pub created_at: chrono::DateTime<chrono::Local>,
    pub description: Option<String>,
}

impl Default for TestRequest {
    fn default() -> Self {
        Self {
            method: HttpMethod::GET,
            endpoint: "/".to_string(),
            headers: Vec::new(),
            query_params: Vec::new(),
            body: None,
            description: "Nueva petición".to_string(),
        }
    }
}

impl Default for TestSuite {
    fn default() -> Self {
        Self {
            name: "Nueva suite de pruebas".to_string(),
            base_url: "http://localhost:8080".to_string(),
            requests: Vec::new(),
            iterations: 10,
            concurrent_requests: 1,
            wait_time: 1,
            output_dir: "./results".to_string(),
        }
    }
} 