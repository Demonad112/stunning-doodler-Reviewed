//! Drives the bundled disk-usage engine (`native/diskusage`, a WinDirStat fork) from DeepServer:
//! drive listing, folder-ledger snapshots kept in the engine's history folder, and comparisons.
//!
//! The history layout matches the engine's own automatic snapshots, so both see one list:
//! `<root>\<16-hex FNV-1a-64 of the location key>\YYYYMMDD-HHMMSS-mmm.ledger.csv` (UTC) plus a
//! `location.txt`. The root is `%LOCALAPPDATA%\DeepServer\History`, or `DEEPSERVER_HISTORY_DIR`.

use serde::Serialize;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Snapshots kept per location; the engine prunes to the same count.
pub const KEEP_PER_LOCATION: usize = 5;
/// `/compare` only reads two CSV files, so a minute means something is stuck.
pub const COMPARE_TIMEOUT: Duration = Duration::from_secs(60);
/// A headless scan of a very large server volume can take hours; past this it is treated as stuck.
pub const SNAPSHOT_TIMEOUT: Duration = Duration::from_secs(6 * 60 * 60);
pub const HISTORY_DIR_ENV: &str = "DEEPSERVER_HISTORY_DIR";
const LEDGER_SUFFIX: &str = ".ledger.csv";

#[derive(Debug)]
pub enum DiskUsageError {
    Io(io::Error),
    /// The engine exited with a failure; `message` is its `.err` text when it wrote one.
    Engine {
        code: Option<i32>,
        message: String,
    },
    TimedOut,
    InvalidPath(String),
    Parse(String),
}

impl std::fmt::Display for DiskUsageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::Engine { code, message } if message.is_empty() => {
                write!(f, "disk-usage engine failed (exit code {code:?})")
            }
            Self::Engine { message, .. } => write!(f, "{message}"),
            Self::TimedOut => write!(f, "disk-usage engine timed out"),
            Self::InvalidPath(path) => write!(f, "not a folder or drive: {path}"),
            Self::Parse(message) => write!(f, "unreadable comparison: {message}"),
        }
    }
}

impl std::error::Error for DiskUsageError {}

impl From<io::Error> for DiskUsageError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

pub type Result<T> = std::result::Result<T, DiskUsageError>;

// ---------------------------------------------------------------------------------------------
// History folder naming (must match the engine's History::LocationKey / HashKey)
// ---------------------------------------------------------------------------------------------

/// Lower-cased scan root without trailing backslashes, e.g. `C:\` -> `c:`, `D:\Data\` -> `d:\data`.
pub fn location_key(scan_root: &str) -> String {
    scan_root
        .trim()
        .trim_end_matches(['\\', '/'])
        .replace('/', "\\")
        .to_lowercase()
}

/// FNV-1a 64 over the key's UTF-16 code units, as 16 lower-case hex digits.
pub fn hash_key(key: &str) -> String {
    let mut hash: u64 = 14_695_981_039_346_656_037;
    for unit in key.encode_utf16() {
        hash ^= u64::from(unit);
        hash = hash.wrapping_mul(1_099_511_628_211);
    }
    format!("{hash:016x}")
}

pub fn history_root() -> PathBuf {
    if let Some(dir) = std::env::var_os(HISTORY_DIR_ENV).filter(|value| !value.is_empty()) {
        return PathBuf::from(dir);
    }
    let local = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    local.join("DeepServer").join("History")
}

pub fn location_dir(root: &Path, scan_root: &str) -> PathBuf {
    root.join(hash_key(&location_key(scan_root)))
}

/// `YYYYMMDD-HHMMSS-mmm.ledger.csv` in UTC, so names sort chronologically.
pub fn snapshot_file_name(time: SystemTime) -> String {
    let since_epoch = time.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = since_epoch.as_secs();
    let (year, month, day) = civil_from_days((secs / 86_400) as i64);
    let rem = secs % 86_400;
    format!(
        "{year:04}{month:02}{day:02}-{:02}{:02}{:02}-{:03}{LEDGER_SUFFIX}",
        rem / 3600,
        (rem / 60) % 60,
        rem % 60,
        since_epoch.subsec_millis()
    )
}

// Howard Hinnant's days-to-civil algorithm (proleptic Gregorian, days since 1970-01-01).
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = yoe + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

// ---------------------------------------------------------------------------------------------
// Snapshots
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotInfo {
    pub path: String,
    pub file_name: String,
    /// `YYYY-MM-DD HH:MM:SS` UTC, parsed from the file name.
    pub taken_at_utc: String,
    pub size_bytes: u64,
}

/// Snapshots of one location, newest first.
pub fn list_snapshots(root: &Path, scan_root: &str) -> Result<Vec<SnapshotInfo>> {
    let dir = location_dir(root, scan_root);
    let entries = match fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.into()),
    };

    let mut snapshots = Vec::new();
    for entry in entries.flatten() {
        let file_name = entry.file_name().to_string_lossy().into_owned();
        let Some(taken_at_utc) = parse_snapshot_time(&file_name) else {
            continue;
        };
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        if !metadata.is_file() {
            continue;
        }
        snapshots.push(SnapshotInfo {
            path: entry.path().to_string_lossy().into_owned(),
            file_name,
            taken_at_utc,
            size_bytes: metadata.len(),
        });
    }
    snapshots.sort_by(|a, b| b.file_name.cmp(&a.file_name));
    Ok(snapshots)
}

fn parse_snapshot_time(file_name: &str) -> Option<String> {
    let stem = file_name.strip_suffix(LEDGER_SUFFIX)?;
    let bytes = stem.as_bytes();
    let digits = |range: std::ops::Range<usize>| bytes[range].iter().all(u8::is_ascii_digit);
    if bytes.len() != 19 || bytes[8] != b'-' || bytes[15] != b'-' {
        return None;
    }
    if !(digits(0..8) && digits(9..15) && digits(16..19)) {
        return None;
    }
    Some(format!(
        "{}-{}-{} {}:{}:{}",
        &stem[0..4],
        &stem[4..6],
        &stem[6..8],
        &stem[9..11],
        &stem[11..13],
        &stem[13..15]
    ))
}

/// Deletes all but the newest `keep` snapshots of the location. Returns how many were removed.
pub fn prune_snapshots(root: &Path, scan_root: &str, keep: usize) -> Result<usize> {
    let mut removed = 0;
    for old in list_snapshots(root, scan_root)?.into_iter().skip(keep) {
        if fs::remove_file(&old.path).is_ok() {
            removed += 1;
        }
    }
    Ok(removed)
}

/// Writes `location.txt` (UTF-16 LE with BOM, like the engine) if the folder doesn't have one yet.
fn write_location_file(dir: &Path, key: &str) -> io::Result<()> {
    let path = dir.join("location.txt");
    if path.exists() {
        return Ok(());
    }
    let mut bytes = vec![0xFF, 0xFE];
    for unit in key.encode_utf16() {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }
    fs::write(path, bytes)
}

/// Scans `scan_root` headlessly with the engine and stores the ledger in the history folder.
/// Blocks for the whole scan (up to [`SNAPSHOT_TIMEOUT`]), so call it off the UI thread.
pub fn take_snapshot(engine: &Path, root: &Path, scan_root: &str) -> Result<SnapshotInfo> {
    take_snapshot_with_timeout(engine, root, scan_root, SNAPSHOT_TIMEOUT)
}

fn take_snapshot_with_timeout(
    engine: &Path,
    root: &Path,
    scan_root: &str,
    timeout: Duration,
) -> Result<SnapshotInfo> {
    let scan_path = Path::new(scan_root.trim());
    if scan_root.trim().is_empty() || !scan_path.is_dir() {
        return Err(DiskUsageError::InvalidPath(scan_root.to_owned()));
    }

    let dir = location_dir(root, scan_root);
    fs::create_dir_all(&dir)?;
    write_location_file(&dir, &location_key(scan_root))?;

    // The engine writes in place, so scan into a temp file and move it in when complete; a
    // half-written ledger must never show up as the newest snapshot.
    let name = snapshot_file_name(SystemTime::now());
    let temp = dir.join(format!(".partial-{name}"));
    // `/saveto` reports failure only through its exit code; it writes no `.err` note.
    let status = run_with_timeout(
        Command::new(engine)
            .arg("/noelevate")
            .arg("/saveto")
            .arg(&temp)
            .arg(scan_path),
        timeout,
    );
    match status {
        Ok(status) if status.success() && temp.is_file() => {}
        Ok(status) => {
            let _ = fs::remove_file(&temp);
            return Err(DiskUsageError::Engine {
                code: status.code(),
                message: String::new(),
            });
        }
        Err(error) => {
            let _ = fs::remove_file(&temp);
            return Err(error);
        }
    }

    let final_path = dir.join(&name);
    fs::rename(&temp, &final_path)?;
    prune_snapshots(root, scan_root, KEEP_PER_LOCATION)?;

    Ok(SnapshotInfo {
        path: final_path.to_string_lossy().into_owned(),
        taken_at_utc: parse_snapshot_time(&name).unwrap_or_default(),
        size_bytes: fs::metadata(&final_path)?.len(),
        file_name: name,
    })
}

// ---------------------------------------------------------------------------------------------
// Comparison
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ChangeKind {
    Added,
    Removed,
    Grown,
    Shrunk,
    FilesChanged,
    Unchanged,
}

impl ChangeKind {
    fn parse(code: &str) -> Option<Self> {
        Some(match code {
            "Added" => Self::Added,
            "Removed" => Self::Removed,
            "Grown" => Self::Grown,
            "Shrunk" => Self::Shrunk,
            "FilesChanged" => Self::FilesChanged,
            "Unchanged" => Self::Unchanged,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeRow {
    /// Folder path relative to the scan root (`.` is the root itself).
    pub folder: String,
    pub change: ChangeKind,
    pub baseline_size: Option<u64>,
    pub current_size: Option<u64>,
    pub size_change: i64,
    pub baseline_files: Option<u64>,
    pub current_files: Option<u64>,
    pub files_change: i64,
}

const COMPARISON_HEADER: &str = "Folder,Change,Baseline Size (bytes),Current Size (bytes),\
Size Change (bytes),Baseline Files,Current Files,Files Change";

/// Parses the engine's `/compare` output (UTF-8 with BOM, CRLF, quoted text fields).
pub fn parse_comparison_csv(text: &str) -> Result<Vec<ChangeRow>> {
    let text = text.strip_prefix('\u{FEFF}').unwrap_or(text);
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    match lines.next() {
        Some(header) if header.trim_end() == COMPARISON_HEADER => {}
        other => {
            return Err(DiskUsageError::Parse(format!(
                "unexpected header {:?}",
                other.unwrap_or_default()
            )))
        }
    }

    lines
        .enumerate()
        .map(|(index, line)| {
            parse_comparison_row(line)
                .ok_or_else(|| DiskUsageError::Parse(format!("row {}: {line}", index + 1)))
        })
        .collect()
}

fn parse_comparison_row(line: &str) -> Option<ChangeRow> {
    let fields = split_csv_line(line);
    if fields.len() != 8 {
        return None;
    }
    let optional = |value: &str| -> Option<Option<u64>> {
        if value.is_empty() {
            Some(None)
        } else {
            value.parse().ok().map(Some)
        }
    };
    Some(ChangeRow {
        folder: fields[0].clone(),
        change: ChangeKind::parse(&fields[1])?,
        baseline_size: optional(&fields[2])?,
        current_size: optional(&fields[3])?,
        size_change: fields[4].parse().ok()?,
        baseline_files: optional(&fields[5])?,
        current_files: optional(&fields[6])?,
        files_change: fields[7].parse().ok()?,
    })
}

// Same rules as the engine's SplitCsv: quoted fields, doubled quotes inside them.
fn split_csv_line(line: &str) -> Vec<String> {
    let mut fields = vec![String::new()];
    let mut quoted = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        let field = fields.last_mut().expect("fields is never empty");
        if quoted {
            if c == '"' && chars.peek() == Some(&'"') {
                field.push('"');
                chars.next();
            } else if c == '"' {
                quoted = false;
            } else {
                field.push(c);
            }
        } else if c == '"' {
            quoted = true;
        } else if c == ',' {
            fields.push(String::new());
        } else {
            field.push(c);
        }
    }
    fields
}

/// Result of `/compare`: the changed folders, plus the engine's `<out>.warn` note when the result is
/// valid but may mislead (the exclusion filters differ between the two snapshots).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Comparison {
    pub rows: Vec<ChangeRow>,
    pub warning: Option<String>,
}

/// Runs the engine's `/compare`. `all` lists every changed folder instead of only significant ones.
pub fn compare_snapshots(
    engine: &Path,
    baseline: &Path,
    current: &Path,
    all: bool,
) -> Result<Comparison> {
    let out = std::env::temp_dir().join(format!(
        "deepserver-compare-{}-{}.csv",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    let err = PathBuf::from(format!("{}.err", out.display()));
    let warn = PathBuf::from(format!("{}.warn", out.display()));

    let mut command = Command::new(engine);
    command.arg("/compare").arg(baseline).arg(current).arg(&out);
    if all {
        command.arg("/all");
    }
    let result = run_with_timeout(&mut command, COMPARE_TIMEOUT).and_then(|status| {
        if status.success() {
            let rows = parse_comparison_csv(&fs::read_to_string(&out)?)?;
            let warning = fs::read_to_string(&warn)
                .ok()
                .map(|text| text.trim_start_matches('\u{FEFF}').trim().to_owned())
                .filter(|text| !text.is_empty());
            Ok(Comparison { rows, warning })
        } else {
            Err(DiskUsageError::Engine {
                code: status.code(),
                message: fs::read_to_string(&err)
                    .unwrap_or_default()
                    .trim()
                    .to_owned(),
            })
        }
    });
    let _ = fs::remove_file(&out);
    let _ = fs::remove_file(&err);
    let _ = fs::remove_file(&warn);
    result
}

fn run_with_timeout(command: &mut Command, timeout: Duration) -> Result<ExitStatus> {
    let mut child = command.spawn()?;
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(DiskUsageError::TimedOut);
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// Opens the engine's treemap window on `path` (it may ask to elevate, which is fine for the GUI).
pub fn open_in_engine(engine: &Path, path: &str) -> Result<()> {
    let target = Path::new(path.trim());
    if path.trim().is_empty() || !target.is_dir() {
        return Err(DiskUsageError::InvalidPath(path.to_owned()));
    }
    Command::new(engine).arg(target).spawn()?;
    Ok(())
}

// ---------------------------------------------------------------------------------------------
// Drives
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DriveKind {
    Fixed,
    Removable,
    Network,
    Optical,
    RamDisk,
    Unknown,
}

impl DriveKind {
    /// Maps `GetDriveTypeW` results.
    pub fn from_drive_type(value: u32) -> Self {
        match value {
            2 => Self::Removable,
            3 => Self::Fixed,
            4 => Self::Network,
            5 => Self::Optical,
            6 => Self::RamDisk,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveInfo {
    /// `C:\`
    pub root: String,
    pub label: String,
    pub kind: DriveKind,
    /// `None` when the drive isn't ready (empty card reader, disconnected share).
    pub total_bytes: Option<u64>,
    pub free_bytes: Option<u64>,
}

#[cfg(windows)]
mod win32 {
    #[link(name = "kernel32")]
    extern "system" {
        pub fn GetLogicalDrives() -> u32;
        pub fn GetDriveTypeW(root: *const u16) -> u32;
        pub fn GetDiskFreeSpaceExW(
            directory: *const u16,
            free_to_caller: *mut u64,
            total: *mut u64,
            total_free: *mut u64,
        ) -> i32;
        pub fn GetVolumeInformationW(
            root: *const u16,
            volume_name: *mut u16,
            volume_name_size: u32,
            serial: *mut u32,
            max_component: *mut u32,
            flags: *mut u32,
            fs_name: *mut u16,
            fs_name_size: u32,
        ) -> i32;
        pub fn SetErrorMode(mode: u32) -> u32;
    }
}

/// Lettered drives with their type, label and space.
#[cfg(windows)]
pub fn list_drives() -> Vec<DriveInfo> {
    use std::ptr::null_mut;

    // Don't pop "insert a disk" dialogs for empty removable drives.
    const SEM_FAILCRITICALERRORS: u32 = 0x0001;
    // SAFETY: plain Win32 calls with NUL-terminated buffers owned by this function.
    unsafe {
        let previous_mode = win32::SetErrorMode(SEM_FAILCRITICALERRORS);
        let mask = win32::GetLogicalDrives();
        let mut drives = Vec::new();
        for index in 0..26u8 {
            if mask & (1 << index) == 0 {
                continue;
            }
            let root = format!("{}:\\", char::from(b'A' + index));
            let wide: Vec<u16> = root.encode_utf16().chain(Some(0)).collect();

            let kind = DriveKind::from_drive_type(win32::GetDriveTypeW(wide.as_ptr()));
            let (mut free, mut total, mut total_free) = (0u64, 0u64, 0u64);
            let ready =
                win32::GetDiskFreeSpaceExW(wide.as_ptr(), &mut free, &mut total, &mut total_free)
                    != 0;

            let mut name = [0u16; 261];
            let label = if win32::GetVolumeInformationW(
                wide.as_ptr(),
                name.as_mut_ptr(),
                name.len() as u32,
                null_mut(),
                null_mut(),
                null_mut(),
                null_mut(),
                0,
            ) != 0
            {
                let len = name.iter().position(|&c| c == 0).unwrap_or(name.len());
                String::from_utf16_lossy(&name[..len])
            } else {
                String::new()
            };

            drives.push(DriveInfo {
                root,
                label,
                kind,
                total_bytes: ready.then_some(total),
                free_bytes: ready.then_some(free),
            });
        }
        win32::SetErrorMode(previous_mode);
        drives
    }
}

#[cfg(not(windows))]
pub fn list_drives() -> Vec<DriveInfo> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn location_key_matches_engine_rules() {
        assert_eq!(location_key(r"C:\"), "c:");
        assert_eq!(location_key(r"D:\Data\Projects\"), r"d:\data\projects");
        assert_eq!(location_key(r"\\Server\Share\"), r"\\server\share");
        assert_eq!(location_key("E:/Media/"), r"e:\media");
    }

    #[test]
    fn hash_key_is_fnv1a64_over_utf16_units() {
        // FNV-1a 64 offset basis for the empty key.
        assert_eq!(hash_key(""), "cbf29ce484222325");
        // One unit: (basis ^ 'a') * prime, the same as byte-wise FNV-1a for ASCII input.
        assert_eq!(hash_key("a"), "af63dc4c8601ec8c");
        assert_eq!(hash_key("c:").len(), 16);
        assert_ne!(hash_key("c:"), hash_key("d:"));
    }

    #[test]
    fn snapshot_names_are_utc_and_sortable() {
        let time = UNIX_EPOCH + Duration::from_millis(1_790_000_000_123);
        let name = snapshot_file_name(time);
        assert_eq!(name, "20260921-141320-123.ledger.csv");
        assert_eq!(
            parse_snapshot_time(&name).as_deref(),
            Some("2026-09-21 14:13:20")
        );
        assert_eq!(
            snapshot_file_name(UNIX_EPOCH),
            "19700101-000000-000.ledger.csv"
        );
        assert_eq!(
            snapshot_file_name(UNIX_EPOCH + Duration::from_secs(951_782_400)),
            "20000229-000000-000.ledger.csv"
        );
    }

    #[test]
    fn ignores_files_that_are_not_snapshots() {
        assert_eq!(parse_snapshot_time("location.txt"), None);
        assert_eq!(
            parse_snapshot_time(".partial-20260921-122000-123.ledger.csv"),
            None
        );
        assert_eq!(parse_snapshot_time("2026092-122000-123.ledger.csv"), None);
    }

    #[test]
    fn lists_newest_first_and_prunes_old_snapshots() {
        let root = std::env::temp_dir().join(format!("diskusage-core-test-{}", std::process::id()));
        let dir = location_dir(&root, r"D:\Data");
        fs::create_dir_all(&dir).unwrap();
        for day in 1..=7 {
            fs::write(
                dir.join(format!("202609{day:02}-080000-000.ledger.csv")),
                "x",
            )
            .unwrap();
        }
        fs::write(dir.join("location.txt"), "d:\\data").unwrap();
        fs::write(dir.join(".partial-20260930-080000-000.ledger.csv"), "x").unwrap();

        let listed = list_snapshots(&root, r"d:\data\").unwrap();
        assert_eq!(listed.len(), 7);
        assert_eq!(listed[0].file_name, "20260907-080000-000.ledger.csv");
        assert_eq!(listed[0].taken_at_utc, "2026-09-07 08:00:00");

        assert_eq!(
            prune_snapshots(&root, r"D:\Data", KEEP_PER_LOCATION).unwrap(),
            2
        );
        let kept = list_snapshots(&root, r"D:\Data").unwrap();
        assert_eq!(kept.len(), KEEP_PER_LOCATION);
        assert_eq!(
            kept.last().unwrap().file_name,
            "20260903-080000-000.ledger.csv"
        );
        assert!(dir.join("location.txt").exists());

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn missing_location_has_no_snapshots() {
        let root = std::env::temp_dir().join("diskusage-core-test-missing");
        assert!(list_snapshots(&root, r"Z:\Nothing").unwrap().is_empty());
    }

    #[test]
    fn parses_engine_comparison_csv() {
        let csv = "\u{FEFF}Folder,Change,Baseline Size (bytes),Current Size (bytes),Size Change (bytes),Baseline Files,Current Files,Files Change\r\n\
\"a\\b\\c\",\"Grown\",100,2097252,2097152,1,3,2\r\n\
\"new\",\"Added\",,4096,4096,,2,2\r\n\
\"gone \"\"old\"\", stuff\",\"Removed\",5000,,-5000,4,,-4\r\n";
        let rows = parse_comparison_csv(csv).unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].folder, r"a\b\c");
        assert_eq!(rows[0].change, ChangeKind::Grown);
        assert_eq!(rows[0].size_change, 2_097_152);
        assert_eq!(rows[1].baseline_size, None);
        assert_eq!(rows[1].current_files, Some(2));
        assert_eq!(rows[2].folder, "gone \"old\", stuff");
        assert_eq!(rows[2].change, ChangeKind::Removed);
        assert_eq!(rows[2].current_size, None);
        assert_eq!(rows[2].size_change, -5000);
        assert_eq!(rows[2].files_change, -4);
    }

    #[test]
    fn rejects_localized_or_malformed_comparisons() {
        let header = "Folder,Change,Baseline Size (bytes),Current Size (bytes),Size Change (bytes),Baseline Files,Current Files,Files Change";
        assert!(
            parse_comparison_csv(&format!("{header}\r\n\"a\",\"Hinzugefügt\",,1,1,,1,1")).is_err()
        );
        assert!(parse_comparison_csv(&format!("{header}\r\n\"a\",\"Added\",,x,1,,1,1")).is_err());
        assert!(parse_comparison_csv("Path,Relative Path\r\n").is_err());
        assert!(parse_comparison_csv(header).unwrap().is_empty());
    }

    #[test]
    fn maps_drive_types() {
        assert_eq!(DriveKind::from_drive_type(3), DriveKind::Fixed);
        assert_eq!(DriveKind::from_drive_type(4), DriveKind::Network);
        assert_eq!(DriveKind::from_drive_type(2), DriveKind::Removable);
        assert_eq!(DriveKind::from_drive_type(0), DriveKind::Unknown);
    }

    #[test]
    fn rejects_snapshot_of_missing_folder() {
        let error = take_snapshot(
            Path::new("unused.exe"),
            &std::env::temp_dir(),
            r"Z:\definitely\not\here",
        )
        .unwrap_err();
        assert!(matches!(error, DiskUsageError::InvalidPath(_)));
    }

    /// Writes a batch file that stands in for the engine; `body` runs with the engine's arguments.
    #[cfg(windows)]
    fn fake_engine(dir: &Path, body: &str) -> PathBuf {
        fs::create_dir_all(dir).unwrap();
        let path = dir.join("fake-engine.cmd");
        fs::write(&path, format!("@echo off\r\n{body}\r\n")).unwrap();
        path
    }

    #[cfg(windows)]
    const FAKE_HEADER: &str = "echo Folder,Change,Baseline Size (bytes),Current Size (bytes),Size Change (bytes),Baseline Files,Current Files,Files Change> \"%~4\"";

    #[cfg(windows)]
    fn leftover_compare_files() -> usize {
        let prefix = format!("deepserver-compare-{}-", std::process::id());
        fs::read_dir(std::env::temp_dir())
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_name().to_string_lossy().starts_with(&prefix))
            .count()
    }

    #[cfg(windows)]
    #[test]
    fn compare_returns_the_filter_warning_and_cleans_up() {
        let dir = std::env::temp_dir().join(format!("diskusage-core-warn-{}", std::process::id()));
        let engine = fake_engine(
            &dir,
            &format!(
                // Redirect first: a trailing `2>>` would redirect stderr instead.
                "{FAKE_HEADER}\r\n>> \"%~4\" echo \"new\",\"Added\",,4096,4096,,2,2\r\n\
                 > \"%~4.warn\" echo The exclusion filters changed.\r\nexit /b 0"
            ),
        );
        let before = leftover_compare_files();

        let comparison = compare_snapshots(
            &engine,
            Path::new("a.ledger.csv"),
            Path::new("b.ledger.csv"),
            false,
        )
        .unwrap();
        assert_eq!(comparison.rows.len(), 1);
        assert_eq!(comparison.rows[0].change, ChangeKind::Added);
        assert_eq!(
            comparison.warning.as_deref(),
            Some("The exclusion filters changed.")
        );
        assert_eq!(leftover_compare_files(), before);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn compare_without_a_warning_note_has_none() {
        let dir =
            std::env::temp_dir().join(format!("diskusage-core-nowarn-{}", std::process::id()));
        let engine = fake_engine(&dir, &format!("{FAKE_HEADER}\r\nexit /b 0"));

        let comparison = compare_snapshots(
            &engine,
            Path::new("a.ledger.csv"),
            Path::new("b.ledger.csv"),
            true,
        )
        .unwrap();
        assert!(comparison.rows.is_empty());
        assert_eq!(comparison.warning, None);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn snapshot_that_hangs_times_out_and_leaves_no_partial_file() {
        let dir = std::env::temp_dir().join(format!("diskusage-core-hang-{}", std::process::id()));
        // Writes a partial ledger (argument 3 is the /saveto target), then hangs.
        let engine = fake_engine(
            &dir.join("engine"),
            "echo x> \"%~3\"\r\nping -n 30 127.0.0.1 > nul",
        );
        let scanned = dir.join("scanned");
        fs::create_dir_all(&scanned).unwrap();
        let root = dir.join("history");
        let scan_root = scanned.to_string_lossy().into_owned();

        let error =
            take_snapshot_with_timeout(&engine, &root, &scan_root, Duration::from_millis(1500))
                .unwrap_err();
        assert!(matches!(error, DiskUsageError::TimedOut));
        let names: Vec<String> = fs::read_dir(location_dir(&root, &scan_root))
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, vec!["location.txt".to_owned()]);

        let _ = fs::remove_dir_all(&dir);
    }

    #[cfg(windows)]
    #[test]
    fn lists_at_least_the_system_drive() {
        let drives = list_drives();
        let system = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".to_owned());
        assert!(drives
            .iter()
            .any(|drive| drive.root.eq_ignore_ascii_case(&format!("{system}\\"))));
    }
}
