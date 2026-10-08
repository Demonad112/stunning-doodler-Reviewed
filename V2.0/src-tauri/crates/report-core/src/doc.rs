use serde::Serialize;

/// A report laid out for export: header, summary tiles, notes and tables.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    /// "Compare", "Record a copy", ...
    pub title: String,
    /// What it was about: "C:\Data → \\nas\backup".
    pub subtitle: String,
    /// Label and value pairs under the title: client, ticket, date, machine, ...
    pub meta: Vec<(String, String)>,
    pub tiles: Vec<Tile>,
    pub notes: Vec<String>,
    /// A headline bar under the tiles (Compare: how much of the source is at the destination).
    pub share: Option<Share>,
    /// The first table is the one exported as CSV.
    pub tables: Vec<Table>,
}

/// A labelled progress bar: `percent` is 0 to 100.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Share {
    pub label: String,
    pub detail: String,
    pub percent: f64,
    pub tone: Tone,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tile {
    pub label: String,
    pub value: String,
    pub detail: String,
    pub tone: Tone,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Tone {
    #[default]
    Plain,
    Good,
    Bad,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Table {
    pub heading: String,
    pub columns: Vec<String>,
    /// Columns shown right-aligned (sizes, counts).
    pub numeric: Vec<bool>,
    pub rows: Vec<Vec<String>>,
    /// Rows drawn in red.
    pub bad_rows: bool,
    /// Said under the table when only part of the list is in the report.
    pub footnote: String,
}

impl Document {
    /// A standalone HTML page (inline styles, prints well) headed with `brand`.
    pub fn html(&self, brand: &str, generated: &str) -> String {
        let brand = if brand.trim().is_empty() {
            "DeepServer"
        } else {
            brand.trim()
        };
        let mut out = String::with_capacity(16 * 1024);
        out.push_str("<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n");
        out.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
        out.push_str(&format!(
            "<title>{} - {}</title>\n<style>{STYLE}</style>\n</head>\n<body>\n<main>\n",
            esc(&self.title),
            esc(brand)
        ));
        let verdict = if self.tiles.iter().any(|tile| tile.tone == Tone::Bad) {
            Some(("bad", "Needs attention"))
        } else if self.tiles.iter().any(|tile| tile.tone == Tone::Good) {
            Some(("good", "All clear"))
        } else {
            None
        };
        out.push_str(&format!(
            "<header><div class=\"top\"><p class=\"brand\">{}</p>{}</div><h1>{}</h1><p class=\"subtitle\">{}</p>",
            esc(brand),
            verdict.map_or(String::new(), |(tone, text)| format!(
                "<span class=\"pill {tone}\">{text}</span>"
            )),
            esc(&self.title),
            esc(&self.subtitle)
        ));
        if !self.meta.is_empty() {
            out.push_str("<dl>");
            for (label, value) in &self.meta {
                out.push_str(&format!(
                    "<div><dt>{}</dt><dd>{}</dd></div>",
                    esc(label),
                    esc(value)
                ));
            }
            out.push_str("</dl>");
        }
        out.push_str("</header>\n");

        if !self.tiles.is_empty() {
            out.push_str("<section class=\"tiles\">");
            for tile in &self.tiles {
                out.push_str(&format!(
                    "<div class=\"tile {}\"><p class=\"label\">{}</p><p class=\"value\">{}</p><p class=\"detail\">{}</p></div>",
                    tone_class(tile.tone),
                    esc(&tile.label),
                    esc(&tile.value),
                    esc(&tile.detail)
                ));
            }
            out.push_str("</section>\n");
        }
        if let Some(share) = &self.share {
            out.push_str(&format!(
                "<section class=\"share {}\"><div class=\"head\"><strong>{}</strong><span>{}</span></div><div class=\"bar\"><i style=\"width:{:.1}%\"></i></div></section>\n",
                tone_class(share.tone),
                esc(&share.label),
                esc(&share.detail),
                share.percent.clamp(0.0, 100.0)
            ));
        }
        for note in &self.notes {
            out.push_str(&format!("<p class=\"note\">{}</p>\n", esc(note)));
        }
        for table in &self.tables {
            out.push_str(&format!(
                "<section class=\"card\"><h2>{}<span class=\"count\">{}</span></h2>",
                esc(&table.heading),
                table.rows.len()
            ));
            if table.rows.is_empty() {
                out.push_str("<p class=\"empty\">Nothing to list.</p></section>\n");
                continue;
            }
            out.push_str("<div class=\"scroll\">");
            out.push_str("<table><thead><tr>");
            for (index, column) in table.columns.iter().enumerate() {
                out.push_str(&format!(
                    "<th{}>{}</th>",
                    num_class(table, index),
                    esc(column)
                ));
            }
            out.push_str("</tr></thead><tbody>");
            for row in &table.rows {
                out.push_str(if table.bad_rows {
                    "<tr class=\"bad\">"
                } else {
                    "<tr>"
                });
                for (index, cell) in row.iter().enumerate() {
                    out.push_str(&format!(
                        "<td{}>{}</td>",
                        num_class(table, index),
                        esc(cell)
                    ));
                }
                out.push_str("</tr>");
            }
            out.push_str("</tbody></table></div>");
            if !table.footnote.is_empty() {
                out.push_str(&format!("<p class=\"note\">{}</p>", esc(&table.footnote)));
            }
            out.push_str("</section>\n");
        }
        out.push_str(&format!(
            "<footer><span>Made with DeepServer 2.0</span><span>{}</span></footer>\n</main>\n</body>\n</html>\n",
            esc(generated)
        ));
        out
    }

    /// The first table as CSV for Excel: UTF-8 with a BOM, CRLF line ends.
    pub fn csv(&self) -> String {
        let mut out = String::from("\u{feff}");
        let Some(table) = self.tables.first() else {
            return out;
        };
        let mut line = |cells: &[String]| {
            let row: Vec<String> = cells.iter().map(|cell| csv_cell(cell)).collect();
            out.push_str(&row.join(","));
            out.push_str("\r\n");
        };
        line(&table.columns);
        for row in &table.rows {
            line(row);
        }
        out
    }
}

fn csv_cell(cell: &str) -> String {
    // A leading = + - @ would run as a formula in Excel.
    let cell = if cell.starts_with(['=', '+', '-', '@']) {
        format!("'{cell}")
    } else {
        cell.to_string()
    };
    if cell.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", cell.replace('"', "\"\""))
    } else {
        cell
    }
}

fn esc(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

fn tone_class(tone: Tone) -> &'static str {
    match tone {
        Tone::Plain => "",
        Tone::Good => "good",
        Tone::Bad => "bad",
    }
}

fn num_class(table: &Table, index: usize) -> &'static str {
    if table.numeric.get(index).copied().unwrap_or(false) {
        " class=\"num\""
    } else {
        ""
    }
}

const STYLE: &str = "
:root{color-scheme:light dark;--page:#f3f5f8;--surface:#fff;--text:#14181f;--muted:#5b6472;--line:#e3e7ee;--zebra:#f8fafc;--accent:#0b5cad;--accent-2:#2f86e0;--good:#0f7b3f;--good-bg:#e6f6ec;--bad:#c42b1c;--bad-bg:#fdecea;--bad-line:#f3b9b2}
@media (prefers-color-scheme:dark){:root{--page:#0f1319;--surface:#181e27;--text:#e8ecf2;--muted:#9aa5b5;--line:#2a3340;--zebra:#1c232d;--accent:#7ab6f5;--accent-2:#4a98ea;--good:#58c98a;--good-bg:#12301f;--bad:#ff8a7d;--bad-bg:#35181a;--bad-line:#6b2c2c}}
*{box-sizing:border-box}
body{margin:0;background:var(--page);color:var(--text);font:14px/1.5 'Segoe UI Variable Text','Segoe UI',system-ui,sans-serif;overflow-wrap:anywhere}
main{max-width:1000px;margin:0 auto;padding:24px 16px 40px}
header{background:linear-gradient(135deg,#0b3d75,#1a73c8);color:#fff;border-radius:14px;padding:24px 28px;margin-bottom:16px;box-shadow:0 6px 24px rgba(11,61,117,.25)}
.top{display:flex;justify-content:space-between;align-items:center;gap:12px}
.brand{margin:0;font-weight:600;letter-spacing:.08em;text-transform:uppercase;font-size:12px;opacity:.85}
.pill{border-radius:999px;padding:3px 12px;font-size:12px;font-weight:600;background:rgba(255,255,255,.18);border:1px solid rgba(255,255,255,.45)}
.pill.good{background:#d7f5e3;color:#0b5a2d;border-color:#d7f5e3}.pill.bad{background:#ffe0dc;color:#9c1f12;border-color:#ffe0dc}
h1{margin:10px 0 0;font:600 30px/1.15 'Segoe UI Variable Display','Segoe UI',system-ui,sans-serif}
.subtitle{margin:6px 0 0;opacity:.9;font-size:15px}
dl{display:flex;flex-wrap:wrap;gap:6px 28px;margin:16px 0 0;padding-top:14px;border-top:1px solid rgba(255,255,255,.25)}
dl div{display:flex;flex-direction:column}dt{font-size:11px;text-transform:uppercase;letter-spacing:.06em;opacity:.75}dd{margin:0;font-weight:600}
.tiles{display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:12px;margin-bottom:16px}
.tile{background:var(--surface);border:1px solid var(--line);border-top:4px solid var(--accent-2);border-radius:10px;padding:12px 16px;box-shadow:0 1px 3px rgba(0,0,0,.05)}
.tile p{margin:0}.label{color:var(--muted);font-size:12px;text-transform:uppercase;letter-spacing:.05em}.detail{color:var(--muted);font-size:12px}
.value{font-size:24px;font-weight:600;font-variant-numeric:tabular-nums;margin:2px 0}
.tile.good{border-top-color:var(--good)}.tile.good .value{color:var(--good)}
.tile.bad{border-top-color:var(--bad);background:var(--bad-bg);border-color:var(--bad-line)}.tile.bad .value{color:var(--bad)}
.share{background:var(--surface);border:1px solid var(--line);border-radius:10px;padding:12px 16px;margin-bottom:16px}
.share .head{display:flex;justify-content:space-between;gap:12px;flex-wrap:wrap;margin-bottom:8px}.share .head span{color:var(--muted)}
.bar{height:12px;border-radius:999px;background:var(--line);overflow:hidden}.bar i{display:block;height:100%;border-radius:999px;background:linear-gradient(90deg,var(--accent),var(--accent-2))}
.share.good .bar i{background:var(--good)}.share.bad .bar i{background:var(--bad)}
.note{color:var(--muted);font-size:13px;margin:8px 4px}.empty{color:var(--muted);padding:8px 0}
.card{background:var(--surface);border:1px solid var(--line);border-radius:10px;padding:8px 16px 12px;margin-bottom:16px;box-shadow:0 1px 3px rgba(0,0,0,.05)}
h2{font-size:16px;margin:10px 0;display:flex;align-items:center;gap:10px}
.count{font-size:12px;font-weight:600;color:var(--accent);background:var(--page);border-radius:999px;padding:1px 10px}
.scroll{overflow-x:auto}
table{width:100%;border-collapse:collapse;font-size:13px}
th{position:sticky;top:0;background:var(--surface);text-align:left;color:var(--muted);font-size:11px;text-transform:uppercase;letter-spacing:.05em;border-bottom:2px solid var(--line);padding:8px}
td{border-bottom:1px solid var(--line);padding:6px 8px;vertical-align:top}
tbody tr:nth-child(even){background:var(--zebra)}
.num{text-align:right;white-space:nowrap;font-variant-numeric:tabular-nums}
tr.bad td{color:var(--bad)}tr.bad td:first-child{border-left:3px solid var(--bad)}
footer{display:flex;justify-content:space-between;gap:12px;flex-wrap:wrap;margin-top:24px;color:var(--muted);font-size:12px}
@media (max-width:560px){header{padding:18px}h1{font-size:24px}.value{font-size:20px}}
@media print{:root{--page:#fff}body{background:#fff}main{padding:0}header{box-shadow:none;-webkit-print-color-adjust:exact;print-color-adjust:exact}.tile,.card,tr{break-inside:avoid;box-shadow:none}th{position:static}}
";

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Document {
        Document {
            title: "Compare".into(),
            subtitle: r"C:\A & B → D:\<copy>".into(),
            meta: vec![("Client".into(), "O'Brien & Co".into())],
            tiles: vec![Tile {
                label: "Missing".into(),
                value: "2 files".into(),
                tone: Tone::Bad,
                ..Tile::default()
            }],
            notes: vec![],
            share: None,
            tables: vec![Table {
                heading: "Missing at destination".into(),
                columns: vec!["Path".into(), "Size".into()],
                numeric: vec![false, true],
                rows: vec![
                    vec!["a, \"quoted\".txt".into(), "1 KB".into()],
                    vec!["=cmd|' /C calc'!A0".into(), "2 KB".into()],
                ],
                bad_rows: true,
                footnote: String::new(),
            }],
        }
    }

    #[test]
    fn html_is_branded_and_escaped() {
        let html = sample().html("Acme IT <script>", "Oct 3, 2026");
        assert!(html.contains("<p class=\"brand\">Acme IT &lt;script&gt;</p>"));
        assert!(html.contains(r"C:\A &amp; B → D:\&lt;copy&gt;"));
        assert!(html.contains("O&#39;Brien &amp; Co"));
        assert!(html.contains("<tr class=\"bad\">"));
        assert!(html.contains("<td class=\"num\">1 KB</td>"));
        assert!(!html.contains("<script>"));
        assert!(sample()
            .html("  ", "x")
            .contains("<p class=\"brand\">DeepServer</p>"));
    }

    #[test]
    fn html_shows_verdict_share_bar_and_row_count() {
        let mut doc = sample();
        doc.share = Some(Share {
            label: "Found".into(),
            detail: "8 of 10".into(),
            percent: 180.0,
            tone: Tone::Plain,
        });
        let html = doc.html("Acme", "x");
        assert!(html.contains("<span class=\"pill bad\">Needs attention</span>"));
        assert!(html.contains("<i style=\"width:100.0%\"></i>"));
        assert!(html.contains("<span class=\"count\">2</span>"));
        assert!(html.contains("prefers-color-scheme:dark"));
        doc.tiles[0].tone = Tone::Good;
        assert!(doc.html("Acme", "x").contains("All clear"));
        doc.tiles.clear();
        assert!(!doc.html("Acme", "x").contains("class=\"pill"));
    }

    #[test]
    fn csv_quotes_and_defuses_formulas() {
        let csv = sample().csv();
        assert!(csv.starts_with('\u{feff}'));
        let lines: Vec<&str> = csv.trim_start_matches('\u{feff}').split("\r\n").collect();
        assert_eq!(lines[0], "Path,Size");
        assert_eq!(lines[1], "\"a, \"\"quoted\"\".txt\",1 KB");
        assert_eq!(lines[2], "'=cmd|' /C calc'!A0,2 KB");
        assert_eq!(Document::default().csv(), "\u{feff}");
    }
}
