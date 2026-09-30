//! Client-ready report for a Transfer Monitor run: a summary, then one table per reason a file
//! was not copied. The CSV has every file.

use crate::{
    escape_csv, escape_html, ReportKind, ReportMetadata, ReportRow, ReportRowStatus, ReportSection,
    ReportSectionKind, UnifiedReport,
};
use serde::{Deserialize, Serialize};

/// The HTML lists at most this many copied files (with hashing on); the CSV has them all.
pub const MAX_HTML_COPIED_ROWS: usize = 5000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportField {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferReportRow {
    pub path: String,
    /// `copied`, `arrived`, `skippedIdentical` or `notCopied`.
    pub status: String,
    pub reason: Option<String>,
    pub message: Option<String>,
    pub size: u64,
    pub source_path: String,
    pub destination_path: String,
    pub source_hash: Option<String>,
    pub dest_hash: Option<String>,
    /// Watch mode: the reason is a best guess.
    pub inferred: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferReportGroup {
    pub title: String,
    pub explanation: String,
    pub bytes: u64,
    pub rows: Vec<TransferReportRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferReport {
    pub title: String,
    pub generated_at: String,
    pub source: String,
    pub destination: String,
    /// Summary lines in display order (times, machine, user, verify level, totals…).
    pub fields: Vec<ReportField>,
    pub hashed: bool,
    pub not_copied: u64,
    /// Not-copied files grouped by reason, largest group first.
    pub groups: Vec<TransferReportGroup>,
    /// Every file, for the CSV and the hash list.
    pub rows: Vec<TransferReportRow>,
}

pub fn render_transfer_html_report(report: &TransferReport) -> String {
    let mut html = String::with_capacity(16 * 1024);
    html.push_str("<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">");
    html.push_str(
        "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>",
    );
    html.push_str(&escape_html(&report.title));
    html.push_str("</title><style>");
    html.push_str(
        "body{font-family:'Segoe UI',system-ui,sans-serif;margin:32px auto;max-width:1100px;padding:0 16px;color:#1f2937;line-height:1.45}\
         h1{font-size:24px;margin:0 0 4px}h2{font-size:18px;margin:28px 0 4px}\
         .muted{color:#6b7280}.banner{margin:20px 0;padding:12px 16px;border-radius:6px;font-weight:600}\
         .ok{background:#ecfdf5;color:#065f46;border:1px solid #a7f3d0}.warn{background:#fef2f2;color:#991b1b;border:1px solid #fecaca}\
         dl{display:grid;grid-template-columns:max-content 1fr;gap:4px 24px;margin:16px 0}dt{color:#6b7280}dd{margin:0;word-break:break-all}\
         table{width:100%;border-collapse:collapse;margin-top:8px;font-size:13px}\
         th,td{border-bottom:1px solid #e5e7eb;padding:6px 8px;text-align:left;vertical-align:top}\
         th{background:#f9fafb;font-weight:600}td.num{text-align:right;white-space:nowrap}td.path,td.hash{word-break:break-all}\
         td.hash{font-family:Consolas,monospace;font-size:12px}\
         @media print{body{margin:0}.banner{border:1px solid #999}}",
    );
    html.push_str("</style></head><body><h1>");
    html.push_str(&escape_html(&report.title));
    html.push_str("</h1><div class=\"muted\">Generated ");
    html.push_str(&escape_html(&report.generated_at));
    html.push_str("</div>");

    if report.not_copied == 0 {
        html.push_str("<div class=\"banner ok\">Every file was copied");
        html.push_str(if report.hashed {
            " and its content checked."
        } else {
            " and checked by size and date."
        });
        html.push_str("</div>");
    } else {
        html.push_str("<div class=\"banner warn\">");
        html.push_str(&report.not_copied.to_string());
        html.push_str(if report.not_copied == 1 {
            " file or folder was not copied. It is listed below with the reason."
        } else {
            " files or folders were not copied. They are listed below by reason."
        });
        html.push_str("</div>");
    }

    html.push_str("<dl>");
    for field in &report.fields {
        html.push_str("<dt>");
        html.push_str(&escape_html(&field.label));
        html.push_str("</dt><dd>");
        html.push_str(&escape_html(&field.value));
        html.push_str("</dd>");
    }
    html.push_str("</dl>");

    for group in &report.groups {
        html.push_str("<section><h2>");
        html.push_str(&escape_html(&group.title));
        html.push_str(" (");
        html.push_str(&group.rows.len().to_string());
        html.push_str(", ");
        html.push_str(&format_bytes(group.bytes));
        html.push_str(")</h2><div class=\"muted\">");
        html.push_str(&escape_html(&group.explanation));
        html.push_str("</div><table><thead><tr><th>File or folder</th><th>Size</th><th>Details</th></tr></thead><tbody>");
        for row in &group.rows {
            html.push_str("<tr><td class=\"path\">");
            html.push_str(&escape_html(&row.path));
            html.push_str("</td><td class=\"num\">");
            html.push_str(&format_bytes(row.size));
            html.push_str("</td><td>");
            html.push_str(&escape_html(row.message.as_deref().unwrap_or("")));
            if row.inferred {
                html.push_str(" <span class=\"muted\">(inferred)</span>");
            }
            html.push_str("</td></tr>");
        }
        html.push_str("</tbody></table></section>");
    }

    if report.hashed {
        let copied: Vec<&TransferReportRow> = report
            .rows
            .iter()
            .filter(|row| {
                row.source_hash.is_some() && (row.status == "copied" || row.status == "arrived")
            })
            .collect();
        if !copied.is_empty() {
            html.push_str("<section><h2>Copied files with content hash (BLAKE3)</h2><div class=\"muted\">The destination was read back after copying and its hash matched the source.");
            if copied.len() > MAX_HTML_COPIED_ROWS {
                html.push_str(" The first ");
                html.push_str(&MAX_HTML_COPIED_ROWS.to_string());
                html.push_str(" are shown; the CSV export lists all ");
                html.push_str(&copied.len().to_string());
                html.push('.');
            }
            html.push_str("</div><table><thead><tr><th>File</th><th>Size</th><th>Hash</th></tr></thead><tbody>");
            for row in copied.into_iter().take(MAX_HTML_COPIED_ROWS) {
                html.push_str("<tr><td class=\"path\">");
                html.push_str(&escape_html(&row.path));
                html.push_str("</td><td class=\"num\">");
                html.push_str(&format_bytes(row.size));
                html.push_str("</td><td class=\"hash\">");
                html.push_str(&escape_html(row.source_hash.as_deref().unwrap_or("")));
                html.push_str("</td></tr>");
            }
            html.push_str("</tbody></table></section>");
        }
    }

    html.push_str("</body></html>");
    html
}

/// One line per file: path, status, reason, OS message, size, both paths and hashes.
pub fn render_transfer_csv_report(report: &TransferReport) -> String {
    let mut csv = String::from(
        "path,status,reason,message,size,source_path,destination_path,source_hash,destination_hash,inferred\n",
    );
    for row in &report.rows {
        let fields = [
            escape_csv(&row.path),
            escape_csv(&row.status),
            escape_csv(row.reason.as_deref().unwrap_or("")),
            escape_csv(row.message.as_deref().unwrap_or("")),
            row.size.to_string(),
            escape_csv(&row.source_path),
            escape_csv(&row.destination_path),
            escape_csv(row.source_hash.as_deref().unwrap_or("")),
            escape_csv(row.dest_hash.as_deref().unwrap_or("")),
            (if row.inferred { "yes" } else { "no" }).to_owned(),
        ];
        csv.push_str(&fields.join(","));
        csv.push('\n');
    }
    csv
}

/// The same report in the shared model, for the generic JSON/XML/Markdown renderers.
pub fn transfer_report_to_unified(report: &TransferReport) -> UnifiedReport {
    let summary = ReportSection {
        kind: ReportSectionKind::Summary,
        title: "Summary".to_owned(),
        rows: report
            .fields
            .iter()
            .map(|field| ReportRow {
                label: field.label.clone(),
                left: Some(field.value.clone()),
                right: None,
                status: ReportRowStatus::Unchanged,
            })
            .collect(),
    };
    let mut unified = UnifiedReport::new(
        ReportKind::Transfer,
        report.title.clone(),
        ReportMetadata {
            generated_at: report.generated_at.clone(),
            left_source: Some(report.source.clone()),
            right_source: Some(report.destination.clone()),
        },
    )
    .with_section(summary);
    for group in &report.groups {
        unified = unified.with_section(ReportSection {
            kind: ReportSectionKind::Differences,
            title: group.title.clone(),
            rows: group
                .rows
                .iter()
                .map(|row| ReportRow {
                    label: row.path.clone(),
                    left: Some(row.source_path.clone()),
                    right: row.message.clone(),
                    status: ReportRowStatus::Removed,
                })
                .collect(),
        });
    }
    unified
}

/// `1.5 MB` style, 1024-based like Explorer.
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["bytes", "KB", "MB", "GB", "TB"];
    if bytes < 1024 {
        return format!("{bytes} bytes");
    }
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.1} {}", UNITS[unit])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(path: &str, status: &str, reason: Option<&str>) -> TransferReportRow {
        TransferReportRow {
            path: path.to_owned(),
            status: status.to_owned(),
            reason: reason.map(str::to_owned),
            message: reason.map(|_| "Access is denied. (os error 5)".to_owned()),
            size: 2048,
            source_path: format!("C:\\src\\{path}"),
            destination_path: format!("E:\\dst\\{path}"),
            source_hash: None,
            dest_hash: None,
            inferred: false,
        }
    }

    fn report(not_copied: bool) -> TransferReport {
        let failed = row("secret, <b>.txt", "notCopied", Some("Access denied"));
        let mut rows = vec![row("a.txt", "copied", None)];
        let mut groups = Vec::new();
        if not_copied {
            rows.push(failed.clone());
            groups.push(TransferReportGroup {
                title: "Access denied".to_owned(),
                explanation: "Windows refused access.".to_owned(),
                bytes: 2048,
                rows: vec![failed],
            });
        }
        TransferReport {
            title: "Transfer report".to_owned(),
            generated_at: "2026-09-30 06:00:00 UTC".to_owned(),
            source: "C:\\src".to_owned(),
            destination: "E:\\dst".to_owned(),
            fields: vec![ReportField {
                label: "Machine".to_owned(),
                value: "HOST".to_owned(),
            }],
            hashed: false,
            not_copied: u64::from(not_copied),
            groups,
            rows,
        }
    }

    #[test]
    fn html_shows_a_table_per_reason_and_escapes_names() {
        let html = render_transfer_html_report(&report(true));
        assert!(html.contains("1 file or folder was not copied"));
        assert!(html.contains("<h2>Access denied (1, 2.0 KB)</h2>"));
        assert!(html.contains("secret, &lt;b&gt;.txt"));
        assert!(!html.contains("<b>.txt"));
    }

    #[test]
    fn html_says_so_when_everything_was_copied() {
        assert!(render_transfer_html_report(&report(false)).contains("Every file was copied"));
    }

    #[test]
    fn csv_lists_every_file_with_quoting() {
        let csv = render_transfer_csv_report(&report(true));
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[1].starts_with("a.txt,copied,,,2048,"));
        assert!(lines[2].starts_with("\"secret, <b>.txt\",notCopied,Access denied,"));
    }

    #[test]
    fn unified_report_has_a_summary_and_a_section_per_group() {
        let unified = transfer_report_to_unified(&report(true));
        assert_eq!(unified.kind, ReportKind::Transfer);
        assert_eq!(unified.sections.len(), 2);
        assert_eq!(unified.sections[1].title, "Access denied");
    }

    #[test]
    fn formats_sizes() {
        assert_eq!(format_bytes(12), "12 bytes");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(5 * 1024 * 1024 * 1024), "5.0 GB");
    }
}
