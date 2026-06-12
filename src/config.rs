#[derive(Clone)]
pub struct Config {
    pub jwt_secret: String,
    pub google_client_id: String,
    pub frontend_origin: String,   
    pub bind_addr: String,        
}

impl Config {
    pub fn from_env() -> Self {
        Config {
            jwt_secret: required("JWT_SECRET"),
            google_client_id: required("GOOGLE_CLIENT_ID"),

            frontend_origin: std::env::var("FRONTEND_ORIGIN")
                .unwrap_or_else(|_| "http://localhost:5173".to_string()),
            bind_addr: std::env::var("BIND_ADDR")
                .unwrap_or_else(|_| "127.0.0.1:8080".to_string()),
        }
    }
}

fn required(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| panic!("required env var {key} is not set"))
}