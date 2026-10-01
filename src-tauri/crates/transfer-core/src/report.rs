//! Builds the client report of a run (rendered by `report-core`).

use crate::prepare::destination_path;
use crate::store::RunStore;
use crate::{
    format_local, now_ms, FailureReason, ItemStatus, Result, RunState, TransferMode, VerifyLevel,
};
use report_core::{
    format_bytes, ReportField, TransferReport, TransferReportGroup, TransferReportRow,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub fn build_report(root: &Path, run_id: &str) -> Result<TransferReport> {
    let store = RunStore::open(root, run_id)?;
    let summary = store.summary()?;
    let latest = store.latest_results()?;
    let source = PathBuf::from(&summary.settings.source);
    let destination = PathBuf::from(&summary.settings.destination);
    let hashed = summary.settings.verify == VerifyLevel::Hash;

    let mut groups: BTreeMap<FailureReason, TransferReportGroup> = BTreeMap::new();
    let mut rows = Vec::with_capacity(latest.len());
    for item in latest.values() {
        let row = TransferReportRow {
            path: item.relative_path.clone(),
            status: status_name(item.status).to_owned(),
            reason: item.reason.map(|reason| reason.title().to_owned()),
            message: item.message.clone(),
            size: item.size,
            source_path: destination_path(&source, &item.relative_path)
                .display()
                .to_string(),
            destination_path: destination_path(&destination, &item.relative_path)
                .display()
                .to_string(),
            source_hash: item.source_hash.clone(),
            dest_hash: item.dest_hash.clone(),
            inferred: item.inferred,
        };
        if let (ItemStatus::NotCopied, Some(reason)) = (item.status, item.reason) {
            let group = groups.entry(reason).or_insert_with(|| TransferReportGroup {
                title: reason.title().to_owned(),
                explanation: reason.explanation().to_owned(),
                bytes: 0,
                rows: Vec::new(),
            });
            group.bytes += item.size;
            group.rows.push(row.clone());
        }
        rows.push(row);
    }
    let mut groups: Vec<TransferReportGroup> = groups.into_values().collect();
    groups.sort_by_key(|group| std::cmp::Reverse(group.rows.len()));

    let totals = &summary.totals;
    let time = |ms: Option<u64>| ms.map(format_local).unwrap_or_else(|| "—".to_owned());
    let mut fields = vec![
        field("Source", &summary.settings.source),
        field("Destination", &summary.settings.destination),
        field(
            "Mode",
            match summary.settings.mode {
                TransferMode::Copy => "Copied by DeepServer",
                TransferMode::Watch => "Copied by another program, checked by DeepServer",
            },
        ),
        field("Started", &time(summary.started_at_ms)),
        field("Finished", &time(summary.finished_at_ms)),
        field("Result", state_name(summary.state)),
        field("Machine", &summary.machine),
        field("User", &summary.user),
        field(
            "Verification",
            if hashed {
                "Size, date and content hash (BLAKE3)"
            } else {
                "Size and date"
            },
        ),
        field(
            "Files in the source",
            &format!(
                "{} ({})",
                totals.planned_files,
                format_bytes(totals.planned_bytes)
            ),
        ),
        field(
            "Copied",
            &format!("{} ({})", totals.copied, format_bytes(totals.copied_bytes)),
        ),
        field(
            "Already at the destination",
            &totals.skipped_identical.to_string(),
        ),
        field(
            "Not copied",
            &format!(
                "{} ({})",
                totals.not_copied,
                format_bytes(totals.not_copied_bytes)
            ),
        ),
    ];
    if totals.excluded > 0 {
        fields.push(field("Left out by filters", &totals.excluded.to_string()));
    }
    if !summary.settings.include.is_empty() || !summary.settings.exclude.is_empty() {
        fields.push(field(
            "Filters",
            &format!(
                "include: {}; exclude: {}",
                or_none(&summary.settings.include),
                or_none(&summary.settings.exclude)
            ),
        ));
    }
    fields.push(field("Run id", &summary.id));

    Ok(TransferReport {
        title: "DeepServer transfer report".to_owned(),
        generated_at: format_local(now_ms()),
        source: summary.settings.source.clone(),
        destination: summary.settings.destination.clone(),
        fields,
        hashed,
        not_copied: totals.not_copied,
        groups,
        rows,
    })
}

fn field(label: &str, value: &str) -> ReportField {
    ReportField {
        label: label.to_owned(),
        value: value.to_owned(),
    }
}

fn or_none(patterns: &[String]) -> String {
    if patterns.is_empty() {
        "none".to_owned()
    } else {
        patterns.join(" ")
    }
}

fn status_name(status: ItemStatus) -> &'static str {
    match status {
        ItemStatus::Copied => "copied",
        ItemStatus::Arrived => "arrived",
        ItemStatus::SkippedIdentical => "skippedIdentical",
        ItemStatus::NotCopied => "notCopied",
    }
}

fn state_name(state: RunState) -> &'static str {
    match state {
        RunState::Prepared => "Not started",
        RunState::Running => "Running",
        RunState::Watching => "Watching",
        RunState::Completed => "Finished",
        RunState::Cancelled => "Cancelled",
        RunState::Failed => "Stopped on an error",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prepare::prepare;
    use crate::run::run_copy;
    use crate::test_support::TempDir;
    use crate::{ConflictPolicy, NullSink, TransferSettings};
    use job_core::CancellationToken;

    #[test]
    fn report_groups_not_copied_files_and_lists_every_file() {
        let dir = TempDir::new("report");
        dir.write("src/a.txt", b"a");
        dir.write("src/b.txt", b"source b");
        dir.write("dst/b.txt", b"other b");
        let settings = TransferSettings {
            source: dir.path("src").display().to_string(),
            destination: dir.path("dst").display().to_string(),
            mode: TransferMode::Copy,
            include: Vec::new(),
            exclude: Vec::new(),
            verify: VerifyLevel::Hash,
            conflict: ConflictPolicy::Skip,
            include_hidden: true,
        };
        let token = CancellationToken::default();
        let (summary, _) = prepare(&dir.path("runs"), settings, &token, &mut NullSink).unwrap();
        run_copy(&dir.path("runs"), &summary.id, None, &token, &mut NullSink).unwrap();

        let report = build_report(&dir.path("runs"), &summary.id).unwrap();

        assert!(report.hashed);
        assert_eq!(report.not_copied, 1);
        assert_eq!(report.rows.len(), 2);
        assert_eq!(report.groups.len(), 1);
        assert_eq!(
            report.groups[0].title,
            FailureReason::TargetExistsDifferent.title()
        );
        assert_eq!(report.groups[0].rows[0].path, "b.txt");
        assert!(report.rows[0].source_hash.is_some());
        let html = report_core::render_transfer_html_report(&report);
        assert!(html.contains("Copied files with content hash"));
    }
}
