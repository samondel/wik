use std::sync::{Arc, Mutex};

pub type Shared<T> = Arc<Mutex<T>>;
