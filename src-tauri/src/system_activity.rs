use crate::db::DbState;
use std::sync::atomic::{AtomicBool, Ordering};

static SYSTEM_ACTIVE: AtomicBool = AtomicBool::new(true);

pub fn is_active() -> bool {
    SYSTEM_ACTIVE.load(Ordering::SeqCst)
}

/// Playback is allowed only when the operator left the system on and
/// maintenance (backup/import/reset) is not in progress.
pub fn allows_playback() -> bool {
    is_active() && !crate::maintenance::is_paused()
}

pub fn set_active(active: bool) {
    SYSTEM_ACTIVE.store(active, Ordering::SeqCst);
    tracing::info!(
        target: "system_activity",
        "[system_activity] - changed - active={active}"
    );
    if !active {
        crate::audio::stop_music();
        crate::os_volume::on_protection_released();
    }
}

pub fn load_from_db(db: &DbState) {
    let active = db.get_system_active().unwrap_or(true);
    SYSTEM_ACTIVE.store(active, Ordering::SeqCst);
    if !active {
        crate::audio::stop_music();
        crate::os_volume::on_protection_released();
    }
}
