use notify_rust::Urgency::{Critical, Low};
use notify_rust::{Notification, Urgency};

use crate::mime::{get_mime_type, map_mime_to_folder};
use crate::stability::wait_for_stable_size;
use std::path::PathBuf;
use std::thread;
use std::{fs, path::Path, time::Duration};

const POLL_INTERVAL: Duration = Duration::from_millis(100);
const STABLE_DURATION: Duration = Duration::from_secs(2);

fn notify_message(summary: &str, message: &str, icon: &str, urgency: Urgency) {
    if let Err(e) = Notification::new()
        .summary(summary)
        .body(message)
        .icon(icon)
        .urgency(urgency)
        .show()
    {
        eprintln!("[ERROR] Failed to send notification: {}", e)
    }
}

fn unique_destination(target_path: &Path) -> PathBuf {
    if !target_path.exists() {
        return target_path.to_path_buf();
    }

    let parent = target_path.parent().unwrap_or_else(|| Path::new(""));
    let stem = target_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file");
    let ext = target_path.extension().and_then(|e| e.to_str());

    let mut counter = 1;
    loop {
        let candidate_name = match ext {
            Some(ext) => format!("{}({}).{}", stem, counter, ext),
            None => format!("{}({})", stem, counter),
        };
        let candidate = parent.join(candidate_name);
        if !candidate.exists() {
            return candidate;
        }
        counter += 1;
    }
}

pub fn process_file(file_path: &Path, downloads_dir: &Path) {
    if !file_path.exists() || !file_path.is_file() {
        return; // file already moved/removed by another handler or the user
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
    let mut target_path = destination_folder.join(file_name);

    if let Err(e) = fs::create_dir_all(&destination_folder) {
        eprintln!(
            "[ERROR] Could not create directory {:?}: {}",
            destination_folder, e
        );
        notify_message(
            "File sort failed",
            &format!(
                "Could not create folder {}: {}",
                destination_folder.display(),
                e
            ),
            "dialog-error",
            Critical,
        );
        return;
    }

    target_path = unique_destination(&target_path);

    println!(
        "[MOVING] {} -> {} ({})",
        file_name.to_string_lossy(),
        target_subfolder,
        mime
    );

    match fs::rename(file_path, &target_path) {
        Ok(_) => {
            println!(
                "[MOVED] {} -> {} ({})",
                file_name.to_string_lossy(),
                target_subfolder,
                mime
            );

            notify_message(
                "File sorted",
                &format!(
                    "{} moved to {}",
                    file_name.to_string_lossy(),
                    target_subfolder
                ),
                "folder",
                Low,
            );
        }
        Err(_) => {
            if let Err(e) = fs::copy(file_path, &target_path) {
                eprintln!("[ERROR] Failed to copy file: {:?}", e);

                notify_message(
                    "File sort failed",
                    &format!("Could not move {}: {}", file_name.to_string_lossy(), e),
                    "dialog-error",
                    Critical,
                );
            } else {
                println!(
                    "[COPY] {} -> {} ({})",
                    file_name.to_string_lossy(),
                    target_subfolder,
                    mime
                );
                println!("[COPY] Removing file at: {:?}", file_path);
                if let Err(e) = fs::remove_file(file_path) {
                    eprintln!("[ERROR] Failed to remove file: {:?}", e);

                    notify_message(
                        "File sort failed",
                        &format!(
                            "Copied {} but failed to remove original: {}",
                            file_name.to_string_lossy(),
                            e
                        ),
                        "dialog-error",
                        Critical,
                    );
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
