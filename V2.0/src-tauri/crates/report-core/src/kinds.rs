//! What each kind of report holds, its row on the Reports page, and its export layout.

use crate::doc::{Document, Table, Tile, Tone};
use crate::{
    format_bytes, format_count, plural, record_job, Body, Entry, JobInfo, ReportKind, Saved,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use transfer_core::store::RunStore;
use transfer_core::{format_local, ItemStatus, RunState, RunSummary, TransferMode};

/// A finished compare: totals and the missing files.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CompareReport {
    pub source: String,
    pub destination: String,
    pub source_size: u64,
    pub source_files: u64,
    pub destination_size: u64,
    pub destination_files: u64,
    pub missing: u64,
    pub missing_bytes: u64,
    pub different: u64,
    pub extra: u64,
    /// Folders on either side that could not be read completely.
    pub unreadable: u64,
    pub elapsed_ms: u64,
    /// Relative to the source; may be cut short (see [`CompareReport::missing`]).
    pub missing_paths: Vec<String>,
}

/// A Disk Cleanup scan, and what was deleted from it afterwards.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct UsageReport {
    pub path: String,
    /// Space on disk; online-only files are not counted.
    pub size: u64,
    pub files: u64,
    pub dirs: u64,
    pub unreadable: u64,
    pub cloud_files: u64,
    pub cloud_bytes: u64,
    pub drive_total: Option<u64>,
    pub drive_free: Option<u64>,
    pub elapsed_ms: u64,
    pub top_folders: Vec<FolderItem>,
    pub largest_files: Vec<FileItem>,
    pub types: Vec<TypeItem>,
    pub removed: Vec<Removed>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct FolderItem {
    pub name: String,
    pub size: u64,
    pub files: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct FileItem {
    pub path: String,
    pub size: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TypeItem {
    pub extension: String,
    pub size: u64,
    pub files: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Removed {
    pub path: String,
    pub size: u64,
    /// Deleted for good rather than moved to the Recycle Bin.
    pub permanent: bool,
}

/// A quick cleanup run (Temp files, browser caches, ...).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CleanReport {
    pub items: Vec<CleanedItem>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CleanedItem {
    pub title: String,
    pub freed: u64,
    pub removed_files: u64,
    /// Files left because they were in use or too new.
    pub skipped_files: u64,
    pub error: Option<String>,
}

impl CleanReport {
    pub fn freed(&self) -> u64 {
        self.items.iter().map(|item| item.freed).sum()
    }
}

pub(crate) fn entry(saved: &Saved) -> Entry {
    let (title, subject, headline, problem) = match &saved.body {
        Body::Compare(report) => (
            "Compare".to_string(),
            format!("{} → {}", report.source, report.destination),
            if report.missing > 0 {
                format!(
                    "{} missing ({})",
                    plural(report.missing, "file"),
                    format_bytes(report.missing_bytes)
                )
            } else {
                "Nothing missing".to_string()
            },
            report.missing > 0,
        ),
        Body::DiskUsage(report) => {
            let removed: u64 = report.removed.iter().map(|item| item.size).sum();
            let mut headline = format!("{} used", format_bytes(report.size));
            if removed > 0 {
                headline.push_str(&format!(" · {} removed", format_bytes(removed)));
            }
            (
                "Disk usage".to_string(),
                report.path.clone(),
                headline,
                false,
            )
        }
        Body::Cleanup(report) => (
            "Quick cleanup".to_string(),
            report
                .items
                .iter()
                .map(|item| item.title.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            format!("Freed {}", format_bytes(report.freed())),
            false,
        ),
    };
    Entry {
        id: saved.id.clone(),
        kind: saved.body.kind(),
        created_at_ms: saved.created_at_ms,
        title,
        subject,
        headline,
        problem,
        job: saved.job.clone(),
    }
}

fn record_title(run: &RunSummary) -> &'static str {
    match run.settings.mode {
        TransferMode::Copy => "Copy for me",
        TransferMode::Watch => "Watched copy",
    }
}

pub(crate) fn record_entry(run: &RunSummary, job: JobInfo) -> Entry {
    let totals = &run.totals;
    let (headline, problem) = if totals.not_copied > 0 {
        (
            format!(
                "{} not copied ({})",
                plural(totals.not_copied, "file"),
                format_bytes(totals.not_copied_bytes)
            ),
            true,
        )
    } else {
        match run.state {
            RunState::Completed => (
                format!("All {} there", plural(totals.planned_files, "file")),
                false,
            ),
            RunState::Failed => (run.error.clone().unwrap_or_else(|| "Failed".into()), true),
            RunState::Cancelled => ("Cancelled".into(), true),
            _ => ("Not finished".into(), false),
        }
    };
    Entry {
        id: run.id.clone(),
        kind: ReportKind::Record,
        created_at_ms: run.created_at_ms,
        title: record_title(run).into(),
        subject: format!("{} → {}", run.settings.source, run.settings.destination),
        headline,
        problem,
        job,
    }
}

fn meta(job: &JobInfo, at_ms: u64, machine: &str, user: &str) -> Vec<(String, String)> {
    let mut meta = Vec::new();
    for (label, value) in [
        ("Client", &job.client),
        ("Ticket", &job.ticket),
        ("Technician", &job.technician),
    ] {
        if !value.trim().is_empty() {
            meta.push((label.to_string(), value.trim().to_string()));
        }
    }
    meta.push(("Date".into(), format_local(at_ms)));
    meta.push(("Computer".into(), machine.to_string()));
    if job.technician.trim().is_empty() && !user.is_empty() {
        meta.push(("Windows user".into(), user.to_string()));
    }
    meta
}

fn tile(label: &str, value: String, detail: String, tone: Tone) -> Tile {
    Tile {
        label: label.into(),
        value,
        detail,
        tone,
    }
}

fn seconds(ms: u64) -> String {
    format!("{:.1} s", ms as f64 / 1000.0)
}

pub(crate) fn document(saved: &Saved) -> Document {
    let meta = meta(&saved.job, saved.created_at_ms, &saved.machine, &saved.user);
    match &saved.body {
        Body::Compare(report) => {
            let listed = report.missing_paths.len() as u64;
            let mut notes = Vec::new();
            if report.unreadable > 0 {
                notes.push(format!(
                    "{} could not be read completely; sizes there may be low.",
                    plural(report.unreadable, "folder")
                ));
            }
            Document {
                title: "Compare".into(),
                subtitle: format!("{} → {}", report.source, report.destination),
                meta,
                tiles: vec![
                    tile(
                        "Source",
                        format_bytes(report.source_size),
                        plural(report.source_files, "file"),
                        Tone::Plain,
                    ),
                    tile(
                        "Destination",
                        format_bytes(report.destination_size),
                        plural(report.destination_files, "file"),
                        Tone::Plain,
                    ),
                    if report.missing > 0 {
                        tile(
                            "Missing at destination",
                            plural(report.missing, "file"),
                            format!("{} not copied", format_bytes(report.missing_bytes)),
                            Tone::Bad,
                        )
                    } else {
                        tile(
                            "Missing at destination",
                            "None".into(),
                            "Every file is there".into(),
                            Tone::Good,
                        )
                    },
                    tile(
                        "Other differences",
                        plural(report.different, "size change"),
                        format!("{} only at destination", plural(report.extra, "file")),
                        Tone::Plain,
                    ),
                ],
                notes,
                tables: vec![Table {
                    heading: "Missing at destination".into(),
                    columns: vec!["Path (relative to the source)".into()],
                    numeric: vec![false],
                    rows: report
                        .missing_paths
                        .iter()
                        .map(|path| vec![path.clone()])
                        .collect(),
                    bad_rows: true,
                    footnote: if listed < report.missing {
                        format!(
                            "The first {} of {} are listed.",
                            format_count(listed),
                            format_count(report.missing)
                        )
                    } else {
                        format!("Compared in {}.", seconds(report.elapsed_ms))
                    },
                }],
            }
        }
        Body::DiskUsage(report) => {
            let removed: u64 = report.removed.iter().map(|item| item.size).sum();
            let mut notes = Vec::new();
            if report.unreadable > 0 {
                notes.push(format!(
                    "{} could not be read, so the totals may be low.",
                    plural(report.unreadable, "folder")
                ));
            }
            if report.cloud_files > 0 {
                notes.push(format!(
                    "{} ({}) are online-only and take no space on this PC; they are not counted.",
                    plural(report.cloud_files, "cloud file"),
                    format_bytes(report.cloud_bytes)
                ));
            }
            let size_row = |name: &str, size: u64, total: u64| -> Vec<String> {
                let share = if total > 0 {
                    format!("{:.1}%", size as f64 * 100.0 / total as f64)
                } else {
                    String::new()
                };
                vec![name.to_string(), format_bytes(size), share]
            };
            let mut tables = vec![
                Table {
                    heading: "Largest files".into(),
                    columns: vec!["File".into(), "Size".into(), "Share".into()],
                    numeric: vec![false, true, true],
                    rows: report
                        .largest_files
                        .iter()
                        .map(|file| size_row(&file.path, file.size, report.size))
                        .collect(),
                    ..Table::default()
                },
                Table {
                    heading: "Biggest folders".into(),
                    columns: vec!["Folder".into(), "Size".into(), "Share".into()],
                    numeric: vec![false, true, true],
                    rows: report
                        .top_folders
                        .iter()
                        .map(|folder| size_row(&folder.name, folder.size, report.size))
                        .collect(),
                    ..Table::default()
                },
                Table {
                    heading: "File types".into(),
                    columns: vec!["Type".into(), "Size".into(), "Share".into()],
                    numeric: vec![false, true, true],
                    rows: report
                        .types
                        .iter()
                        .map(|kind| {
                            let name = if kind.extension.is_empty() {
                                format!("No extension ({})", plural(kind.files, "file"))
                            } else {
                                format!(".{} ({})", kind.extension, plural(kind.files, "file"))
                            };
                            size_row(&name, kind.size, report.size)
                        })
                        .collect(),
                    ..Table::default()
                },
            ];
            if !report.removed.is_empty() {
                tables.insert(
                    0,
                    Table {
                        heading: "Removed".into(),
                        columns: vec!["Item".into(), "Size".into(), "How".into()],
                        numeric: vec![false, true, false],
                        rows: report
                            .removed
                            .iter()
                            .map(|item| {
                                vec![
                                    item.path.clone(),
                                    format_bytes(item.size),
                                    if item.permanent {
                                        "Deleted".into()
                                    } else {
                                        "Recycle Bin".into()
                                    },
                                ]
                            })
                            .collect(),
                        ..Table::default()
                    },
                );
            }
            let mut tiles = vec![
                tile(
                    "Used",
                    format_bytes(report.size),
                    format!(
                        "{} · {}",
                        plural(report.files, "file"),
                        plural(report.dirs, "folder")
                    ),
                    Tone::Plain,
                ),
                tile(
                    "Removed",
                    format_bytes(removed),
                    plural(report.removed.len() as u64, "item"),
                    if removed > 0 { Tone::Good } else { Tone::Plain },
                ),
            ];
            if let (Some(total), Some(free)) = (report.drive_total, report.drive_free) {
                tiles.insert(
                    1,
                    tile(
                        "Free space at scan",
                        format_bytes(free),
                        format!("of {}", format_bytes(total)),
                        Tone::Plain,
                    ),
                );
            }
            Document {
                title: "Disk usage".into(),
                subtitle: report.path.clone(),
                meta,
                tiles,
                notes,
                tables,
            }
        }
        Body::Cleanup(report) => Document {
            title: "Quick cleanup".into(),
            subtitle: format!("Freed {}", format_bytes(report.freed())),
            meta,
            tiles: vec![tile(
                "Freed",
                format_bytes(report.freed()),
                plural(
                    report.items.iter().map(|item| item.removed_files).sum(),
                    "file",
                ) + " removed",
                Tone::Good,
            )],
            notes: vec!["Files in use or changed in the last day are left in place.".into()],
            tables: vec![Table {
                heading: "Cleaned".into(),
                columns: vec![
                    "Item".into(),
                    "Freed".into(),
                    "Files removed".into(),
                    "Files left".into(),
                ],
                numeric: vec![false, true, true, true],
                rows: report
                    .items
                    .iter()
                    .map(|item| {
                        vec![
                            match &item.error {
                                Some(error) => format!("{} ({error})", item.title),
                                None => item.title.clone(),
                            },
                            format_bytes(item.freed),
                            format_count(item.removed_files),
                            format_count(item.skipped_files),
                        ]
                    })
                    .collect(),
                ..Table::default()
            }],
        },
    }
}

pub(crate) fn record_document(records: &Path, id: &str) -> Result<Document, String> {
    let store = RunStore::open(records, id).map_err(|err| err.to_string())?;
    let run = store.summary().map_err(|err| err.to_string())?;
    let results = store.latest_results().map_err(|err| err.to_string())?;
    let not_copied: Vec<Vec<String>> = results
        .values()
        .filter(|item| item.status == ItemStatus::NotCopied)
        .map(|item| {
            let reason = item
                .reason
                .map(|reason| reason.title())
                .unwrap_or("Other error");
            vec![
                item.relative_path.clone(),
                format_bytes(item.size),
                reason.to_string(),
                item.message.clone().unwrap_or_default(),
            ]
        })
        .collect();
    let totals = &run.totals;
    let job = record_job(&store);
    let arrived = match run.settings.mode {
        TransferMode::Copy => "Copied",
        TransferMode::Watch => "Arrived",
    };
    let mut notes = Vec::new();
    if let Some(error) = &run.error {
        notes.push(format!("The record stopped: {error}"));
    }
    if run.settings.ignore_junk {
        notes.push(
            "System and temp files (Thumbs.db, desktop.ini, ~$ lock files, .tmp) were ignored."
                .into(),
        );
    }
    Ok(Document {
        title: record_title(&run).into(),
        subtitle: format!("{} → {}", run.settings.source, run.settings.destination),
        meta: meta(&job, run.created_at_ms, &run.machine, &run.user),
        tiles: vec![
            tile(
                "Source",
                format_bytes(totals.planned_bytes),
                plural(totals.planned_files, "file"),
                Tone::Plain,
            ),
            tile(
                arrived,
                plural(totals.copied + totals.skipped_identical, "file"),
                format!(
                    "{} · {} already there",
                    format_bytes(totals.copied_bytes),
                    format_count(totals.skipped_identical)
                ),
                Tone::Good,
            ),
            if totals.not_copied > 0 {
                tile(
                    "Not copied",
                    plural(totals.not_copied, "file"),
                    format_bytes(totals.not_copied_bytes),
                    Tone::Bad,
                )
            } else {
                tile(
                    "Not copied",
                    "None".into(),
                    "Every file made it".into(),
                    Tone::Good,
                )
            },
        ],
        notes,
        tables: vec![Table {
            heading: "Not copied".into(),
            columns: vec![
                "Path (relative to the source)".into(),
                "Size".into(),
                "Reason".into(),
                "Details".into(),
            ],
            numeric: vec![false, true, false, false],
            rows: not_copied,
            bad_rows: true,
            footnote: String::new(),
        }],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn saved(body: Body) -> Saved {
        Saved {
            id: "x".into(),
            created_at_ms: 0,
            machine: "PC1".into(),
            user: "sam".into(),
            job: JobInfo {
                client: "Acme".into(),
                ticket: "T-42".into(),
                technician: String::new(),
            },
            body,
        }
    }

    #[test]
    fn compare_rows_flag_missing_files() {
        let report = CompareReport {
            source: r"C:\Data".into(),
            destination: r"E:\Data".into(),
            missing: 3,
            missing_bytes: 2048,
            missing_paths: vec!["a.txt".into(), "b.txt".into()],
            ..CompareReport::default()
        };
        let saved = saved(Body::Compare(report));
        let row = entry(&saved);
        assert!(row.problem);
        assert_eq!(row.headline, "3 files missing (2.00 KB)");
        let doc = document(&saved);
        assert_eq!(doc.tables[0].rows.len(), 2);
        assert_eq!(doc.tables[0].footnote, "The first 2 of 3 are listed.");
        assert_eq!(doc.tiles[2].tone, Tone::Bad);
        assert!(doc.meta.contains(&("Ticket".into(), "T-42".into())));
        assert!(doc.meta.contains(&("Windows user".into(), "sam".into())));
    }

    #[test]
    fn usage_lists_removed_items_first() {
        let report = UsageReport {
            path: r"C:\".into(),
            size: 1000,
            removed: vec![Removed {
                path: r"C:\old.iso".into(),
                size: 400,
                permanent: false,
            }],
            ..UsageReport::default()
        };
        let saved = saved(Body::DiskUsage(report));
        assert_eq!(
            entry(&saved).headline,
            "1000 bytes used · 400 bytes removed"
        );
        let doc = document(&saved);
        assert_eq!(doc.tables[0].heading, "Removed");
        assert_eq!(doc.tables[0].rows[0][2], "Recycle Bin");
    }

    #[test]
    fn cleanup_adds_up_what_was_freed() {
        let report = CleanReport {
            items: vec![
                CleanedItem {
                    title: "Temporary files".into(),
                    freed: 1024,
                    removed_files: 3,
                    ..CleanedItem::default()
                },
                CleanedItem {
                    title: "Browser caches".into(),
                    freed: 1024,
                    ..CleanedItem::default()
                },
            ],
        };
        let saved = saved(Body::Cleanup(report));
        let row = entry(&saved);
        assert_eq!(row.headline, "Freed 2.00 KB");
        assert_eq!(row.subject, "Temporary files, Browser caches");
        assert_eq!(document(&saved).tables[0].rows.len(), 2);
    }
}
