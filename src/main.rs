mod file_ops;
mod mime;
mod stability;

use notify::{
    event::ModifyKind, Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use rapidhash::RapidHashSet;
use std::panic;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::{sync::mpsc::channel, thread};

use crate::file_ops::{process_file, sort_existing_files};

const WORKERS: usize = 4;

type Error = Box<dyn std::error::Error>;
type InProgress = Arc<Mutex<RapidHashSet<PathBuf>>>;

fn handle_path(path: PathBuf, downloads_dir: &Path, in_progress: &InProgress) {
    let result = panic::catch_unwind(|| process_file(&path, downloads_dir));

    if let Err(e) = result {
        eprintln!("[ERROR] process_file panicked for {:?}: {:?}", path, e);
    }

    in_progress
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .remove(&path);
}

fn main() -> Result<(), Error> {
    let downloads_dir: Arc<PathBuf> =
        Arc::new(dirs::download_dir().ok_or("Could not locate downloads directory")?);
    let in_progress: InProgress = Arc::new(Mutex::new(RapidHashSet::default()));

    println!("Watching downloads folder: {:?}", downloads_dir);

    if let Err(e) = sort_existing_files(&downloads_dir) {
        eprintln!("[ERROR] Initial sort failed: {}", e)
    }

    // bounded pool instead of spawning new thread per file
    let (work_tx, work_rx) = channel::<PathBuf>();
    let work_rx = Arc::new(Mutex::new(work_rx));

    for _ in 0..WORKERS {
        let work_rx = Arc::clone(&work_rx);
        let downloads_dir = Arc::clone(&downloads_dir);
        let in_progress = Arc::clone(&in_progress);
        thread::spawn(move || loop {
            let next = { work_rx.lock().unwrap().recv() };
            match next {
                Ok(path) => handle_path(path, &downloads_dir, &in_progress),
                Err(_) => break, // sender dropped, shut down
            }
        });
    }

    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(
        move |res| match res {
            Ok(event) => {
                let _ = tx.send(event);
            }
            Err(e) => eprintln!("[ERROR] watch error: {e}"),
        },
        Config::default(),
    )?;
    watcher.watch(&downloads_dir, RecursiveMode::NonRecursive)?;

    for event in rx {
        if let Event {
            kind: EventKind::Create(_) | EventKind::Modify(ModifyKind::Name(_)),
            paths,
            ..
        } = event
        {
            for path in paths {
                {
                    let mut guard = in_progress
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                    if guard.contains(&path) {
                        continue;
                    }
                    guard.insert(path.clone());
                }

                if work_tx.send(path).is_err() {
                    eprintln!("[ERROR] worker pool is gone, dropping event");
                }
            }
        }
    }

    Ok(())
}
