use std::fs;
use std::io::Read;
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::error::NstatError;

pub enum Kind {
    Regular,
    Directory,
    Symlink(PathBuf),
    Fifo,
    Socket,
    BlockDevice,
    CharDevice,
}

pub struct Info {
    pub name: String,
    pub kind: Kind,
    pub subtype: Option<&'static str>,
    pub size: u64,
    pub disk_blocks: u64,
    pub io_block: u64,
    pub hard_links: u64,
    pub device: u64,
    pub inode: u64,
    pub mode: u32,
    pub owner: String,
    pub group: String,
    pub modified: SystemTime,
    pub accessed: SystemTime,
    pub created: Option<SystemTime>,
}

/// Stats a path and classifies it
pub fn gather(path: &Path, no_follow: bool) -> Result<Info, NstatError> {
    let link_meta = fs::symlink_metadata(path).map_err(|e| to_nstat_error(path, e))?;
    let is_symlink = link_meta.file_type().is_symlink();
    let kind = classify(path, &link_meta)?;
    let meta = if is_symlink && !no_follow {
        fs::metadata(path).map_err(|e| to_nstat_error(path, e))?
    } else {
        link_meta
    };
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string());

    let subtype = match &kind {
        Kind::Regular => detect_subtype(path),
        _ => None,
    };

    let uid = meta.uid();
    let gid = meta.gid();

    let owner = unsafe {
        let pwd = libc::getpwuid(uid);
        if !pwd.is_null() {
            std::ffi::CStr::from_ptr((*pwd).pw_name)
                .to_string_lossy()
                .into_owned()
        } else {
            uid.to_string()
        }
    };

    let group = unsafe {
        let grp = libc::getgrgid(gid);
        if !grp.is_null() {
            std::ffi::CStr::from_ptr((*grp).gr_name)
                .to_string_lossy()
                .into_owned()
        } else {
            gid.to_string()
        }
    };

    Ok(Info {
        name,
        kind,
        subtype,
        size: meta.size(),
        disk_blocks: meta.blocks(),
        io_block: meta.blksize(),
        hard_links: meta.nlink(),
        device: meta.dev(),
        inode: meta.ino(),
        mode: meta.mode(),
        owner,
        group,
        modified: meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
        accessed: meta.accessed().unwrap_or(SystemTime::UNIX_EPOCH),
        created: meta.created().ok(),
    })
}

fn classify(path: &Path, meta: &fs::Metadata) -> Result<Kind, NstatError> {
    let ft = meta.file_type();
    if ft.is_symlink() {
        let target =
            fs::read_link(path).map_err(|e| NstatError::ReadLink(path.to_path_buf(), e))?;
        return Ok(Kind::Symlink(target));
    }
    if ft.is_dir() {
        return Ok(Kind::Directory);
    }
    if ft.is_fifo() {
        return Ok(Kind::Fifo);
    }
    if ft.is_socket() {
        return Ok(Kind::Socket);
    }
    if ft.is_block_device() {
        return Ok(Kind::BlockDevice);
    }
    if ft.is_char_device() {
        return Ok(Kind::CharDevice);
    }
    Ok(Kind::Regular)
}

fn to_nstat_error(path: &Path, e: std::io::Error) -> NstatError {
    match e.kind() {
        std::io::ErrorKind::NotFound => NstatError::PathNotFound(path.to_path_buf()),
        std::io::ErrorKind::PermissionDenied => NstatError::PermissionDenied(path.to_path_buf()),
        _ => NstatError::Stat(path.to_path_buf(), e),
    }
}

/// Identifies a regular file's format
fn detect_subtype(path: &Path) -> Option<&'static str> {
    if let Some(magic) = detect_by_magic(path) {
        return Some(magic);
    }
    detect_by_extension(path)
}

fn detect_by_magic(path: &Path) -> Option<&'static str> {
    let mut f = match fs::File::open(path) {
        Ok(file) => file,
        Err(_) => return None, // Deliberate fallback to extension checking on read failure
    };
    let mut buf = [0u8; 265];
    let n = match f.read(&mut buf) {
        Ok(bytes) => bytes,
        Err(_) => return None, // Deliberate fallback to extension checking on read failure.
    };
    let b = &buf[..n];

    if b.starts_with(b"Signature: 8a477f597d28d172789f06886806bc55") {
        return Some("Cache Directory Tag");
    }
    if b.starts_with(b"\x7fELF") {
        return Some("ELF Executable");
    }
    if b.starts_with(b"\xfe\xed\xfa\xce")
        || b.starts_with(b"\xfe\xed\xfa\xcf")
        || b.starts_with(b"\xce\xfa\xed\xfe")
        || b.starts_with(b"\xcf\xfa\xed\xfe")
    {
        return Some("Mach-O Executable");
    }
    if b.starts_with(b"\xca\xfe\xba\xbe") {
        return Some("Java Class File");
    }
    if b.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Some("PNG Image");
    }
    if b.starts_with(b"\xff\xd8\xff") {
        return Some("JPEG Image");
    }
    if b.starts_with(b"GIF87a") || b.starts_with(b"GIF89a") {
        return Some("GIF Image");
    }
    if b.starts_with(b"BM") {
        return Some("BMP Image");
    }
    if b.len() >= 12 && &b[0..4] == b"RIFF" {
        if &b[8..12] == b"WEBP" {
            return Some("WebP Image");
        }
        if &b[8..12] == b"WAVE" {
            return Some("WAV Audio");
        }
    }
    if b.starts_with(b"OggS") {
        return Some("Ogg Media");
    }
    if b.starts_with(b"ID3") || b.starts_with(b"\xff\xfb") {
        return Some("MP3 Audio");
    }
    if b.starts_with(b"fLaC") {
        return Some("FLAC Audio");
    }
    if b.starts_with(b"%PDF-") {
        return Some("PDF Document");
    }
    if b.starts_with(b"SQLite format 3\x00") {
        return Some("SQLite Database");
    }
    if b.len() >= 262 && &b[257..262] == b"ustar" {
        return Some("Tar Archive");
    }
    if b.starts_with(b"\x1f\x8b") {
        return Some("Gzip Compressed Archive");
    }
    if b.starts_with(b"BZh") {
        return Some("Bzip2 Compressed Archive");
    }
    if b.starts_with(b"\xfd7zXZ\x00") {
        return Some("XZ Compressed Archive");
    }
    if b.starts_with(b"7z\xbc\xaf\x27\x1c") {
        return Some("7-Zip Archive");
    }
    if b.starts_with(b"PK\x03\x04") || b.starts_with(b"PK\x05\x06") {
        return Some("ZIP Archive");
    }
    if b.starts_with(b"\x00asm") {
        return Some("WebAssembly Binary");
    }
    if b.starts_with(b"#!") {
        return detect_shebang(b);
    }
    None
}

fn detect_shebang(first_bytes: &[u8]) -> Option<&'static str> {
    let line = std::str::from_utf8(first_bytes).ok()?;
    let line = line.lines().next()?;
    if line.contains("python") {
        Some("Python Script")
    } else if line.contains("bash") || line.contains("/sh") {
        Some("Shell Script")
    } else if line.contains("node") {
        Some("Node.js Script")
    } else if line.contains("lua") {
        Some("Lua Script")
    } else {
        None
    }
}

fn detect_by_extension(path: &Path) -> Option<&'static str> {
    let name = path.file_name()?.to_str()?;

    // Filenames
    match name {
        "Cargo.toml" | "Cargo.lock" | "pyproject.toml" => return Some("TOML Configuration"),
        "package.json" | "package-lock.json" | "tsconfig.json" | "composer.json" => {
            return Some("JSON Document");
        }
        "Dockerfile" => return Some("Dockerfile"),
        "Makefile" | "makefile" | "GNUmakefile" => return Some("Makefile"),
        ".gitignore" | ".dockerignore" | ".npmignore" => return Some("Ignore Pattern List"),
        ".gitattributes" => return Some("Git Attributes File"),
        ".editorconfig" => return Some("EditorConfig File"),
        ".env" => return Some("Environment Variable File"),
        _ => {}
    }

    let ext = path.extension()?.to_str()?.to_ascii_lowercase();

    Some(match ext.as_str() {
        "md" | "markdown" => "Markdown Document",
        "toml" => "TOML Configuration",
        "json" => "JSON Document",
        "yaml" | "yml" => "YAML Configuration",
        "xml" => "XML Document",
        "ini" | "cfg" | "conf" => "INI Configuration",
        "csv" => "CSV Document",
        "sql" => "SQL Script",
        "html" | "htm" => "HTML Document",
        "css" => "CSS Stylesheet",
        "rs" => "Rust Source",
        "c" => "C Source",
        "h" => "C Header",
        "cpp" | "cc" | "cxx" => "C++ Source",
        "hpp" => "C++ Header",
        "py" => "Python Source",
        "sh" | "bash" | "zsh" => "Shell Script",
        "lua" => "Lua Source",
        "go" => "Go Source",
        "java" => "Java Source",
        "kt" | "kts" => "Kotlin Source",
        "swift" => "Swift Source",
        "rb" => "Ruby Source",
        "php" => "PHP Source",
        "cs" => "C# Source",
        "ts" => "TypeScript Source",
        "tsx" => "TypeScript JSX Source",
        "js" | "mjs" | "cjs" => "JavaScript Source",
        "jsx" => "JavaScript JSX Source",
        "zig" => "Zig Source",
        "nim" => "Nim Source",
        "hs" => "Haskell Source",
        "ml" | "mli" => "OCaml Source",
        "ex" | "exs" => "Elixir Source",
        "erl" => "Erlang Source",
        "clj" | "cljs" => "Clojure Source",
        "vue" => "Vue Component",
        "svelte" => "Svelte Component",
        "proto" => "Protocol Buffers Schema",
        "txt" => "Text Document",
        "mp4" => "MPEG-4 Video",
        "mkv" => "Matroska Video",
        "webm" => "WebM Video",
        "flac" => "FLAC Audio",
        "mp3" => "MP3 Audio",
        "svg" => "SVG Image",
        _ => return None,
    })
}
