# Download Sorter

Watches your Downloads folder and automatically sorts files into subfolders
based on MIME type. Runs continuously in the background, reacting to new
files as they finish downloading, and can also do a one-time sweep of files
that are already sitting there when it starts.

## Features

- **Live watching** — uses filesystem events (via `notify`) to detect new
  files as soon as they land in Downloads, no polling of the whole directory.
- **Startup sweep** — sorts any files already present in Downloads when the
  program launches, without waiting for a "download finished" signal.
- **Stability check** — before moving a file, waits until its size stops
  changing for a set duration, so in-progress downloads (including
  browser temp-file/rename patterns) aren't moved half-finished.
- **Concurrent processing** — each file is handled on its own thread, with
  in-flight deduplication so the same file isn't processed twice at once
  by overlapping filesystem events.
- **Ignores directories, `.part`/`.tmp`/`.crdownload` temp files, and
  anything already sorted into a subfolder** (so re-running is safe/idempotent).
- **Falls back to `copy` + `remove`** if a direct rename fails (e.g. the
  destination is on a different filesystem).

## MIME Types → Folders

MIME detection first tries [infer](https://crates.io/crates/infer)
(magic-byte sniffing), falling back to
[mime_guess](https://crates.io/crates/mime_guess) (extension-based) if that
fails.

| Folder           | Matches                                                |
| ---------------- | ------------------------------------------------------ |
| `Images`         | `image/*`                                              |
| `Videos`         | `video/*`                                              |
| `Audio`          | `audio/*`                                              |
| `Fonts`          | `font/*`                                               |
| `Documents/Text` | `text/*`                                               |
| `Documents/PDFs` | `application/pdf`                                      |
| `Archive`        | zip, tar, rar, gzip, bzip2, bzip3, 7z, xz              |
| `Books`          | epub, mobi                                             |
| `Documents`      | Word, Excel, PowerPoint (and OpenDocument equivalents) |
| `Executables`    | Windows PE, ELF executables, shared libraries          |
| `ISOs`           | ISO 9660 disc images                                   |
| `Other`          | anything unrecognized                                  |

## Building

```bash
cargo build --release
```

### With Nix

A flake is included for a reproducible build:

```bash
nix build
./result/bin/download_sorter
```

Or build straight from a GitHub repo without cloning first:

```bash
nix build github:yourusername/download-sorter
```

## Running

```bash
cargo run
```

The program locates your OS's default Downloads directory automatically
(via the [dirs](https://crates.io/crates/dirs) crate) and begins watching it.

### As a background service

For always-on sorting, run it as a systemd user service instead of a
foreground terminal process — see `download-sorter.service` (or your Home
Manager config) for an example unit that starts on login and restarts on
failure.

## Crates used

- [notify](https://crates.io/crates/notify) — filesystem event watching
- [infer](https://crates.io/crates/infer) — magic-byte MIME type detection
- [mime_guess](https://crates.io/crates/mime_guess) — extension-based MIME fallback
- [dirs](https://crates.io/crates/dirs) — cross-platform Downloads folder lookup
