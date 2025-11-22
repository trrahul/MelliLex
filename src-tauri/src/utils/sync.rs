use crate::errors::AppError;
use std::sync::{Mutex, MutexGuard};

pub fn lock<'a, T>(mutex: &'a Mutex<T>, resource: &'static str) -> Result<MutexGuard<'a, T>, AppError> {
    mutex
        .lock()
        .map_err(|_| AppError::lock_poisoned(resource))
}
