use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::storage::Store;

pub struct AppState {
    pub store: Arc<Mutex<Store>>,
    pub suppress: Arc<Mutex<Option<String>>>,
    pub media_dir: PathBuf,
}
