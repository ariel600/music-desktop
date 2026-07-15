use std::sync::atomic::{AtomicBool, Ordering};

static PAUSED: AtomicBool = AtomicBool::new(false);

pub fn pause() {
    PAUSED.store(true, Ordering::SeqCst);
    tracing::info!(target: "maintenance", "[maintenance] - paused - playback and schedulers suspended");
}

pub fn resume() {
    PAUSED.store(false, Ordering::SeqCst);
    tracing::info!(target: "maintenance", "[maintenance] - resumed - normal operation restored");
}

pub fn is_paused() -> bool {
    PAUSED.load(Ordering::SeqCst)
}

pub struct PauseGuard;

impl PauseGuard {
    pub fn enter() -> Self {
        pause();
        Self
    }
}

impl Drop for PauseGuard {
    fn drop(&mut self) {
        resume();
    }
}
