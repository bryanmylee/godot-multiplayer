use std::env;

#[allow(dead_code)]
fn get_secret_text_or_file(var: &str) -> Option<String> {
    let secret_text = env::var(var);

    if let Ok(secret_text) = secret_text {
        Some(secret_text)
    } else {
        use std::fs;
        let Ok(secret_file) = env::var(format!("{var}_FILE")) else {
            return None;
        };
        fs::read_to_string(secret_file).ok()
    }
}

fn get_required_secret_text_or_file(var: &str) -> String {
    let secret_text = env::var(var);

    if let Ok(secret_text) = secret_text {
        secret_text
    } else {
        use std::fs;
        let secret_file = env::var(format!("{var}_FILE"))
            .expect(format!("Expected either {var} or {var}_FILE to be set").as_str());
        fs::read_to_string(secret_file)
            .expect(format!("The file at {var}_FILE should contain the secret").as_str())
    }
}

lazy_static::lazy_static! {
    pub static ref POSTGRES_URL: String = get_required_secret_text_or_file("POSTGRES_URL");
    pub static ref GAME_SERVER_MANAGER_SERVICE_KEY: String = get_required_secret_text_or_file("GAME_SERVER_MANAGER_SERVICE_KEY");
}
