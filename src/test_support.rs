use std::sync::{Mutex, MutexGuard, OnceLock};

/// Returns the single global lock that must be used by all tests mutating
/// process environment variables so env-dependent tests stay serialized across
/// modules.
pub fn env_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    match LOCK.get_or_init(|| Mutex::new(())).lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}
