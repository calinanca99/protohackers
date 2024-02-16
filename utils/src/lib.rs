use std::env;

pub fn addr() -> &'static str {
    match env::var("ENV") {
        Ok(var) if var.to_lowercase() == "proto" => "0.0.0.0:8000",
        Ok(_) | Err(_) => "127.0.0.1:8000",
    }
}
