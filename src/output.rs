use std::time::SystemTime;

use crate::color::Palette;
use crate::info::{Info, Kind};

pub fn render(info: &Info, explain: bool, pal: &Palette) -> String {
    let mut out = String::new();

    out.push_str(&format!("{}\n", pal.value(&info.name)));
    render_type_lines(info, &mut out, pal);
    out.push('\n');

    match info.kind {
        Kind::Regular => render_regular(info, &mut out, pal, explain),
        Kind::Directory => {}
        Kind::Symlink(_) => {}
        Kind::Fifo | Kind::Socket | Kind::BlockDevice | Kind::CharDevice => {}
    }

    render_permissions(info, &mut out, pal, explain);
    out.push('\n');
    render_times(info, &mut out, pal, explain);

    out
}

fn render_type_lines(info: &Info, out: &mut String, pal: &Palette) {
    let (primary, secondary): (&str, Option<String>) = match &info.kind {
        Kind::Regular => ("Regular File", info.subtype.map(|s| s.to_string())),
        Kind::Directory => ("Directory", None),
        Kind::Symlink(target) => (
            "Symbolic Link",
            Some(format!("Target: {}", target.display())),
        ),
        Kind::Fifo => ("Named Pipe", None),
        Kind::Socket => ("Unix Domain Socket", None),
        Kind::BlockDevice => ("Block Device", None),
        Kind::CharDevice => ("Character Device", None),
    };

    out.push_str(&format!("  {}\n", pal.identity(primary)));
    if let Some(s) = secondary {
        out.push_str(&format!("  {}\n", pal.identity(&s)));
    }
}

const REGULAR_LABELS: [&str; 6] = [
    "Size",
    "Disk Blocks",
    "IO Block",
    "Hard Links",
    "Device",
    "Inode",
];

fn render_regular(info: &Info, out: &mut String, pal: &Palette, explain: bool) {
    let width = label_width(&REGULAR_LABELS);
    row(
        out,
        pal,
        "Size",
        &human_bytes(info.size),
        explain,
        EXPLAIN_SIZE,
        width,
        Palette::stat,
    );
    row(
        out,
        pal,
        "Disk Blocks",
        &format!("{} ({} B each)", info.disk_blocks, 512),
        explain,
        EXPLAIN_DISK_BLOCKS,
        width,
        Palette::stat,
    );
    if info.io_block > 0 {
        row(
            out,
            pal,
            "IO Block",
            &human_bytes(info.io_block),
            explain,
            EXPLAIN_IO_BLOCK,
            width,
            Palette::stat,
        );
    }
    row(
        out,
        pal,
        "Hard Links",
        &info.hard_links.to_string(),
        explain,
        EXPLAIN_HARD_LINKS,
        width,
        Palette::stat,
    );
    row(
        out,
        pal,
        "Device",
        &format_device(info.device),
        explain,
        EXPLAIN_DEVICE,
        width,
        Palette::stat,
    );
    row(
        out,
        pal,
        "Inode",
        &info.inode.to_string(),
        explain,
        EXPLAIN_INODE,
        width,
        Palette::stat,
    );
    out.push('\n');
}

fn label_width(labels: &[&str]) -> usize {
    labels.iter().map(|l| l.chars().count()).max().unwrap_or(0)
}

fn render_permissions(info: &Info, out: &mut String, pal: &Palette, explain: bool) {
    let perm_bits = info.mode & 0o777;
    row(
        out,
        pal,
        "Permissions",
        &format!("{} {:04o}", rwx_string(perm_bits), perm_bits),
        explain,
        EXPLAIN_PERMISSIONS,
        "Permissions".chars().count(),
        Palette::perm,
    );
    for (label, shift) in [("Owner", 6), ("Group", 3), ("Other", 0)] {
        let bits = (perm_bits >> shift) & 0o7;
        out.push_str(&format!(
            "        {}  {}\n",
            pal.label(label),
            pal.perm(&rwx_words(bits))
        ));
    }
}

const TIME_LABELS: [&str; 3] = ["Modified", "Accessed", "Created"];

fn render_times(info: &Info, out: &mut String, pal: &Palette, explain: bool) {
    let width = label_width(&TIME_LABELS);
    row(
        out,
        pal,
        "Modified",
        &natural_time(info.modified),
        explain,
        EXPLAIN_MODIFIED,
        width,
        Palette::time,
    );
    row(
        out,
        pal,
        "Accessed",
        &natural_time(info.accessed),
        explain,
        EXPLAIN_ACCESSED,
        width,
        Palette::time,
    );
    match info.created {
        Some(t) => row(
            out,
            pal,
            " Created",
            &natural_time(t),
            explain,
            EXPLAIN_CREATED,
            width,
            Palette::time,
        ),
        None => row(
            out,
            pal,
            "Created",
            "Not available on this filesystem",
            explain,
            EXPLAIN_CREATED,
            width,
            Palette::time,
        ),
    }
}

/// Renders one "Label   value" line, padded to `width` column
fn row(
    out: &mut String,
    pal: &Palette,
    label: &str,
    value: &str,
    explain: bool,
    explanation: &str,
    width: usize,
    color: fn(&Palette, &str) -> String,
) {
    let pad = " ".repeat(width.saturating_sub(label.chars().count()));
    out.push_str(&format!(
        "  {}{}  {}\n",
        pal.label(label),
        pad,
        color(pal, value)
    ));
    if explain {
        out.push_str(&format!("      {}\n", explanation));
    }
}

fn human_bytes(n: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut size = n as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{n} B")
    } else {
        format!("{size:.1} {} ({n} B)", UNITS[unit])
    }
}

fn rwx_string(bits: u32) -> String {
    let mut s = String::with_capacity(9);
    for shift in [6, 3, 0] {
        let b = (bits >> shift) & 0o7;
        s.push(if b & 4 != 0 { 'r' } else { '-' });
        s.push(if b & 2 != 0 { 'w' } else { '-' });
        s.push(if b & 1 != 0 { 'x' } else { '-' });
    }
    s
}

fn rwx_words(bits: u32) -> String {
    let mut parts = Vec::new();
    if bits & 4 != 0 {
        parts.push("Read");
    }
    if bits & 2 != 0 {
        parts.push("Write");
    }
    if bits & 1 != 0 {
        parts.push("Execute");
    }
    if parts.is_empty() {
        "None".to_string()
    } else {
        parts.join(" / ")
    }
}

fn format_device(dev: u64) -> String {
    let major = (dev >> 8) & 0xfff;
    let minor = (dev & 0xff) | ((dev >> 12) & 0xfff00);
    format!("{major}:{minor}")
}

fn natural_time(t: SystemTime) -> String {
    let now = SystemTime::now();
    let secs = match now.duration_since(t) {
        Ok(d) => d.as_secs() as i64,
        Err(e) => -(e.duration().as_secs() as i64),
    };

    let phrase = if secs < 0 {
        "in the future".to_string()
    } else if secs < 60 {
        "just now".to_string()
    } else if secs < 3600 {
        plural(secs / 60, "minute")
    } else if secs < 86400 {
        plural(secs / 3600, "hour")
    } else if secs < 86400 * 30 {
        plural(secs / 86400, "day")
    } else if secs < 86400 * 365 {
        plural(secs / (86400 * 30), "month")
    } else {
        plural(secs / (86400 * 365), "year")
    };

    format!("{} | {phrase}", absolute_time(t))
}

fn plural(n: i64, unit: &str) -> String {
    if n == 1 {
        format!("1 {unit} ago")
    } else {
        format!("{n} {unit}s ago")
    }
}

/// Formats a SystemTime as an ISO-ish local independnt timestamp
fn absolute_time(t: SystemTime) -> String {
    let secs = t
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    civil_from_unix(secs)
}

fn civil_from_unix(unix_secs: i64) -> String {
    let days = unix_secs.div_euclid(86400);
    let rem = unix_secs.rem_euclid(86400);
    let (h, m, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);

    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if mo <= 2 { y + 1 } else { y };

    format!("{y:04}-{mo:02}-{d:02} {h:02}:{m:02}:{s:02} UTC")
}

const EXPLAIN_SIZE: &str = "The number of bytes of actual content in the file.";
const EXPLAIN_DISK_BLOCKS: &str =
    "The number of 512-byte blocks allocated on disk. Can be smaller than size for sparse files.";
const EXPLAIN_IO_BLOCK: &str =
    "The filesystem's preferred block size for efficient I/O on this file.";
const EXPLAIN_HARD_LINKS: &str = "The number of directory entries pointing at this same inode.";
const EXPLAIN_DEVICE: &str = "The major:minor number of the device holding this filesystem.";
const EXPLAIN_INODE: &str =
    "The filesystem's internal identifier for this file's metadata and data.";
const EXPLAIN_PERMISSIONS: &str =
    "Read, write, and execute bits for the owner, group, and everyone else.";
const EXPLAIN_MODIFIED: &str = "When the file's contents were last changed.";
const EXPLAIN_ACCESSED: &str =
    "When the file was last read. Many systems no longer update this by default.";
const EXPLAIN_CREATED: &str = "When the file was created, if the filesystem tracks it.";
