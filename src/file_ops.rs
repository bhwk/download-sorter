use crate::mime::{get_mime_type, map_mime_to_folder};
use crate::stability::wait_for_stable_size;
use std::thread;
use std::{fs, path::Path, time::Duration};

const POLL_INTERVAL: Duration = Duration::from_millis(100);
const STABLE_DURATION: Duration = Duration::from_secs(2);

pub fn process_file(file_path: &Path, downloads_dir: &Path) {
    if !file_path.exists() {
        return; // file already moved/removed by another handler or the user
    }

    if !file_path.is_file() {
        return;
    }

    if let Ok(relative_path) = file_path.strip_prefix(downloads_dir) {
        if relative_path.components().count() > 1 {
            return;
        }
    }

    // check if the file is part of the main file or something
    if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
        if matches!(ext, "part" | "tmp" | "crdownload") {
            return;
        }
    }

    // we just wait until the size stops changing
    if wait_for_stable_size(file_path, POLL_INTERVAL, STABLE_DURATION).is_none() {
        eprintln!("[SKIP] {:?} never stabilized, or was removed", file_path);
        return;
    };

    let file_name = match file_path.file_name() {
        Some(name) => name,
        None => return,
    };

    let mime = get_mime_type(file_path).unwrap_or_else(|| "application/octet-stream".to_string());
    let target_subfolder = map_mime_to_folder(&mime);

    let destination_folder = downloads_dir.join(target_subfolder);
    let target_path = destination_folder.join(file_name);

    if let Err(e) = std::fs::create_dir_all(&destination_folder) {
        eprintln!(
            "[ERROR] Could not create directory {:?}: {}",
            destination_folder, e
        )
    };

    println!(
        "[MOVING] {} -> {} ({})",
        file_name.to_string_lossy(),
        target_subfolder,
        mime
    );

    match fs::rename(file_path, &target_path) {
        Ok(_) => println!(
            "[MOVED] {} -> {} ({})",
            file_name.to_string_lossy(),
            target_subfolder,
            mime
        ),
        Err(_) => {
            if let Err(e) = fs::copy(file_path, &target_path) {
                eprintln!("[ERROR] Failed to copy file: {:?}", e)
            } else {
                println!(
                    "[COPY] {} -> {} ({})",
                    file_name.to_string_lossy(),
                    target_subfolder,
                    mime
                );
                println!("[COPY] Removing file at: {:?}", file_path);
                if let Err(e) = fs::remove_file(file_path) {
                    eprintln!("[ERROR] Failed to remove file: {:?}", e)
                } else {
                    println!("[COPY] Removed file at: {:?}", file_path);
                }
            }
        }
    }
}

pub fn sort_existing_files(downloads_dir: &Path) -> std::io::Result<()> {
    let mut handles = vec![];
    for entry in fs::read_dir(downloads_dir)? {
        let entry = entry?;
        let path = entry.path();
        let downloads_dir = downloads_dir.to_path_buf();
        handles.push(thread::spawn(move || {
            process_file(&path, &downloads_dir);
        }));
    }
    for h in handles {
        _ = h.join();
    }
    Ok(())
}
