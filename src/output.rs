use std::fmt::Write as _;
use std::time::SystemTime;

use crate::color::{Painted, Palette};
use crate::info::{Info, Kind};

struct RenderCtx<'a> {
    out: &'a mut String,
    pal: &'a Palette,
    explain: bool,
}

pub fn render(info: &Info, explain: bool, pal: &Palette) -> String {
    // Preallocate a reasonable chunk
    let mut out = String::with_capacity(1024);

    let _ = writeln!(out, "{}", pal.value(&info.name));
    render_type_lines(info, &mut out, pal);
    out.push('\n');

    let mut ctx = RenderCtx {
        out: &mut out,
        pal,
        explain,
    };

    if let Kind::Regular = info.kind {
        render_regular(info, &mut ctx)
    }

    render_permissions(info, &mut ctx);
    ctx.out.push('\n');
    render_times(info, &mut ctx);

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

    let _ = writeln!(out, "  {}", pal.identity(primary));
    if let Some(s) = secondary {
        let _ = writeln!(out, "  {}", pal.identity(&s));
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

fn render_regular(info: &Info, ctx: &mut RenderCtx) {
    let width = label_width(&REGULAR_LABELS);
    row(
        ctx,
        "Size",
        &human_bytes(info.size),
        EXPLAIN_SIZE,
        width,
        Palette::stat,
    );
    row(
        ctx,
        "Disk Blocks",
        &format!("{} ({} B each)", info.disk_blocks, 512),
        EXPLAIN_DISK_BLOCKS,
        width,
        Palette::stat,
    );
    if info.io_block > 0 {
        row(
            ctx,
            "IO Block",
            &human_bytes(info.io_block),
            EXPLAIN_IO_BLOCK,
            width,
            Palette::stat,
        );
    }
    row(
        ctx,
        "Hard Links",
        &info.hard_links.to_string(),
        EXPLAIN_HARD_LINKS,
        width,
        Palette::stat,
    );
    row(
        ctx,
        "Device",
        &format_device(info.device),
        EXPLAIN_DEVICE,
        width,
        Palette::stat,
    );
    row(
        ctx,
        "Inode",
        &info.inode.to_string(),
        EXPLAIN_INODE,
        width,
        Palette::stat,
    );
    ctx.out.push('\n');
}

fn label_width(labels: &[&str]) -> usize {
    labels.iter().map(|l| l.chars().count()).max().unwrap_or(0)
}

fn render_permissions(info: &Info, ctx: &mut RenderCtx) {
    let perm_bits = info.mode & 0o777;
    let width = "Permissions".chars().count();
    row(
        ctx,
        "Permissions",
        &format!("{} {:04o}", rwx_string(perm_bits), perm_bits),
        EXPLAIN_PERMISSIONS,
        width,
        Palette::perm,
    );

    out.push_str(&format!(
        "        {}  {}\n",
        pal.label("Owner"),
        pal.stat(&info.owner)
    ));

    out.push_str(&format!(
        "        {}  {}\n",
        pal.label("Group"),
        pal.stat(&info.group),
    ));

    for (label, shift) in [("Owner Rights", 6), ("Group Rights", 3), ("Other Rights", 0)] {
        let bits = (perm_bits >> shift) & 0o7;
        let _ = writeln!(
            ctx.out,
            "        {}  {}",
            ctx.pal.label(label),
            ctx.pal.perm(rwx_words(bits))
        );
    }
}

const TIME_LABELS: [&str; 3] = ["Modified", "Accessed", "Created"];

fn render_times(info: &Info, ctx: &mut RenderCtx) {
    let width = label_width(&TIME_LABELS);
    render_time_row(ctx, "Modified", info.modified, EXPLAIN_MODIFIED, width);
    render_time_row(ctx, "Accessed", info.accessed, EXPLAIN_ACCESSED, width);
    render_time_row(ctx, " Created", info.created, EXPLAIN_CREATED, width);
}

fn render_time_row(
    ctx: &mut RenderCtx,
    label: &str,
    value: Option<SystemTime>,
    explanation: &str,
    width: usize,
) {
    match value {
        Some(t) => row(ctx, label, &natural_time(t), explanation, width, Palette::time),
        None => row(
            ctx,
            label.trim_start(),
            "Not available on this filesystem",
            explanation,
            width,
            Palette::time,
        ),
    }
}

/// Renders one padded metric row.
fn row(
    ctx: &mut RenderCtx,
    label: &str,
    value: &str,
    explanation: &str,
    width: usize,
    color: for<'p, 'v> fn(&'p Palette, &'v str) -> Painted<'v>,
) {
    let pad_len = width.saturating_sub(label.chars().count());
    let _ = write!(ctx.out, "  {}", ctx.pal.label(label));
    for _ in 0..pad_len {
        let _ = write!(ctx.out, " ");
    }
    let _ = writeln!(ctx.out, "  {}", color(ctx.pal, value));
    if ctx.explain {
        let _ = writeln!(ctx.out, "      {}", explanation);
    }
}

// funny name
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
    let mut buf = [b'-'; 9];
    for (i, shift) in [6, 3, 0].into_iter().enumerate() {
        let b = (bits >> shift) & 0o7;
        if b & 4 != 0 {
            buf[i * 3] = b'r';
        }
        if b & 2 != 0 {
            buf[i * 3 + 1] = b'w';
        }
        if b & 1 != 0 {
            buf[i * 3 + 2] = b'x';
        }
    }

    buf.iter().map(|&b| b as char).collect()
}

fn rwx_words(bits: u32) -> &'static str {
    match bits & 0o7 {
        0 => "None",
        1 => "Execute",
        2 => "Write",
        3 => "Write / Execute",
        4 => "Read",
        5 => "Read / Execute",
        6 => "Read / Write",
        7 => "Read / Write / Execute",
        _ => unreachable!(),
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
        "In the future".to_string()
    } else if secs < 60 {
        "Just now".to_string()
    } else if secs < 3600 {
        plural(secs / 60, "Minute")
    } else if secs < 86400 {
        plural(secs / 3600, "Hour")
    } else if secs < 86400 * 30 {
        plural(secs / 86400, "Day")
    } else if secs < 86400 * 365 {
        plural(secs / (86400 * 30), "Month")
    } else {
        plural(secs / (86400 * 365), "Year")
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

fn get_local_offset(unix_secs: i64) -> i64 {
    /*
     * localtime_r is thread-safe and does not mutate global state
     * The tm_struct is zero-initialized and lives on the local stack
     * We own the pointer passed to the C function, ensuring no aliasing or lifetime issues
     */
    unsafe {
        let time_val = unix_secs as libc::time_t;
        let mut tm_struct = std::mem::zeroed::<libc::tm>();

        if libc::localtime_r(&time_val, &mut tm_struct).is_null() {
            return 0; // Fallback to 0 if the system call fails
        }

        tm_struct.tm_gmtoff as i64
    }
}

fn absolute_time(t: SystemTime) -> String {
    let secs = t
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let offset = get_local_offset(secs);
    let local_secs = secs + offset;

    let tz_label = if offset == 0 {
        "UTC".to_string()
    } else {
        let sign = if offset >= 0 { '+' } else { '-' };
        let abs_offset = offset.abs();
        let hours = abs_offset / 3600;
        let mins = (abs_offset % 3600) / 60;

        if mins == 0 {
            format!("UTC{}{}", sign, hours)
        } else {
            format!("UTC{}{}:{:02}", sign, hours, mins)
        }
    };

    format!("{} {}", civil_from_unix(local_secs), tz_label)
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

    format!("{y:04}-{mo:02}-{d:02} {h:02}:{m:02}:{s:02}")
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
