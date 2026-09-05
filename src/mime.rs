use std::path::Path;

pub fn get_mime_type(path: &Path) -> Option<String> {
    if let Ok(Some(kind)) = infer::get_from_path(path) {
        return Some(kind.mime_type().to_string());
    }

    if let Some(guess) = mime_guess::from_path(path).first() {
        return Some(guess.to_string());
    }
    None
}

pub fn map_mime_to_folder(mime: &str) -> &'static str {
    match mime {
        m if m.starts_with("image/") => "Images",
        m if m.starts_with("video/") => "Videos",
        m if m.starts_with("audio/") => "Audio",
        m if m.starts_with("font/") => "Fonts",
        m if m.starts_with("text/") => "Documents/Text",

        "application/pdf" => "Documents/PDFs",
        "application/zip"
        | "application/x-tar"
        | "application/vnd.rar"
        | "application/gzip"
        | "application/x-bzip2"
        | "application/vnd.bzip3"
        | "application/x-7z-compressed"
        | "application/x-xz" => "Archive",

        "application/epub+zip" | "application/x-mobipocket-ebook" => "Books",

        "application/msword"
        | "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        | "application/vnd.ms-excel"
        | "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
        | "application/vnd.ms-powerpoint"
        | "application/vnd.openxmlformats-officedocument.presentationml.presentation"
        | "application/vnd.oasis.opendocument.text"
        | "application/vnd.oasis.opendocument.spreadsheet"
        | "application/vnd.oasis.opendocument.presentation" => "Documents",

        "application/vnd.microsoft.portable-executable"
        | "application/x-executable"
        | "application/x-sharedlib" => "Executables",

        "application/x-iso9660-image" => "ISOs",

        _ => "Other",
    }
}
