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
    /// The first table is the one exported as CSV.
    pub tables: Vec<Table>,
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
        out.push_str(&format!(
            "<header><p class=\"brand\">{}</p><h1>{}</h1><p class=\"subtitle\">{}</p>",
            esc(brand),
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
        for note in &self.notes {
            out.push_str(&format!("<p class=\"note\">{}</p>\n", esc(note)));
        }
        for table in &self.tables {
            out.push_str(&format!("<section><h2>{}</h2>", esc(&table.heading)));
            if table.rows.is_empty() {
                out.push_str("<p class=\"empty\">None.</p></section>\n");
                continue;
            }
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
            out.push_str("</tbody></table>");
            if !table.footnote.is_empty() {
                out.push_str(&format!("<p class=\"note\">{}</p>", esc(&table.footnote)));
            }
            out.push_str("</section>\n");
        }
        out.push_str(&format!(
            "<footer>Made with DeepServer 2.0 · {}</footer>\n</main>\n</body>\n</html>\n",
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
:root{color-scheme:light;--text:#1b1b1b;--muted:#5d5d5d;--line:#e5e5e5;--card:#fafafa;--accent:#005fb8;--good:#0f7b0f;--bad:#c42b1c;--bad-bg:#fde7e9}
*{box-sizing:border-box}
body{margin:0;background:#fff;color:var(--text);font:14px/1.45 'Segoe UI Variable Text','Segoe UI',system-ui,sans-serif}
main{max-width:960px;margin:0 auto;padding:32px 24px}
header{border-bottom:3px solid var(--accent);padding-bottom:16px;margin-bottom:20px}
.brand{margin:0;color:var(--accent);font-weight:600;letter-spacing:.02em}
h1{margin:4px 0 0;font:600 28px/1.2 'Segoe UI Variable Display','Segoe UI',system-ui,sans-serif}
.subtitle{margin:4px 0 0;color:var(--muted);word-break:break-all}
dl{display:flex;flex-wrap:wrap;gap:4px 24px;margin:12px 0 0}
dl div{display:flex;gap:6px}dt{color:var(--muted)}dd{margin:0;font-weight:600}
.tiles{display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:12px;margin-bottom:16px}
.tile{border:1px solid var(--line);border-radius:8px;background:var(--card);padding:10px 14px}
.tile p{margin:0}.label,.detail{color:var(--muted);font-size:12px}
.value{font-size:20px;font-weight:600;font-variant-numeric:tabular-nums}
.tile.good .value{color:var(--good)}.tile.bad{border-color:#f1b8b2;background:var(--bad-bg)}.tile.bad .value{color:var(--bad)}
.note{color:var(--muted);font-size:13px}.empty{color:var(--muted)}
h2{font-size:16px;margin:24px 0 8px}
table{width:100%;border-collapse:collapse;font-size:13px}
th{text-align:left;color:var(--muted);font-weight:600;border-bottom:1px solid var(--line);padding:6px 8px}
td{border-bottom:1px solid var(--line);padding:5px 8px;word-break:break-all;vertical-align:top}
.num{text-align:right;white-space:nowrap;font-variant-numeric:tabular-nums;word-break:normal}
tr.bad td{color:var(--bad)}
footer{margin-top:32px;padding-top:12px;border-top:1px solid var(--line);color:var(--muted);font-size:12px}
@media print{main{padding:0}.tile{break-inside:avoid}tr{break-inside:avoid}}
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
