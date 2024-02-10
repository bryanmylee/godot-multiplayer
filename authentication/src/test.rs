use std::sync::{Arc, Mutex};

lazy_static::lazy_static! {
    static ref INITIATED: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
}

#[cfg(test)]
pub fn init() {
    let mut initiated = INITIATED.lock().unwrap();
    if !*initiated {
        dotenvy::dotenv().ok();
        *initiated = true;
    }
}