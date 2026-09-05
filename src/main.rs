mod file_ops;
mod mime;
mod stability;

use notify::{
    event::ModifyKind, Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use rapidhash::RapidHashSet;
use std::sync::{Arc, Mutex};
use std::{sync::mpsc::channel, thread};

use crate::file_ops::{process_file, sort_existing_files};

type Error = Box<dyn std::error::Error>;

fn main() -> Result<(), Error> {
    let downloads_dir = dirs::download_dir().ok_or("Could not locate downloads directory")?;

    let in_progress: Arc<Mutex<RapidHashSet<std::path::PathBuf>>> =
        Arc::new(Mutex::new(RapidHashSet::default()));

    println!("Watching downloads folder: {:?}", downloads_dir);

    if let Err(e) = sort_existing_files(&downloads_dir) {
        eprintln!("[ERROR] Initial sort failed: {}", e)
    }

    let (tx, rx) = channel();

    let mut watcher = RecommendedWatcher::new(
        move |res| {
            if let Ok(event) = res {
                _ = tx.send(event);
            }
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
                let downloads_dir = downloads_dir.clone();
                let in_progress = Arc::clone(&in_progress);

                {
                    let mut guard = in_progress
                        .lock()
                        .map_err(|e| format!("Failed to acquire lock: {:?}", e))?;
                    if guard.contains(&path) {
                        continue;
                    }
                    guard.insert(path.clone());
                }

                thread::spawn(move || {
                    process_file(&path, &downloads_dir);
                    in_progress
                        .lock()
                        .expect("Failed to acquire lock")
                        .remove(&path)
                });
            }
        }
    }

    Ok(())
}
