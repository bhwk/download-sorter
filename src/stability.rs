use std::{
    fs,
    path::Path,
    thread,
    time::{Duration, Instant},
};

pub fn wait_for_stable_size(
    path: &Path,
    poll_interval: Duration,
    required_stable_duration: Duration,
) -> Option<u64> {
    let mut last_size = fs::metadata(path).ok()?.len();
    let mut last_changed_at = Instant::now();

    loop {
        thread::sleep(poll_interval);

        let size = match fs::metadata(path) {
            Ok(meta) => meta.len(),
            Err(_) => return None, // file vanished or became unreadable — give up
        };

        if size != last_size {
            last_size = size;
            last_changed_at = Instant::now();
            continue;
        }

        // size hasn't moved since last_changed_at — check how long it's been quiet
        if size > 0 && last_changed_at.elapsed() >= required_stable_duration {
            return Some(size);
        }
    }
}
