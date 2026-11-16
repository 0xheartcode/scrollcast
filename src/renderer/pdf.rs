use crate::renderer::{DocumentMetadata, DocumentRenderer};
use crate::syntax::highlighter::SyntaxHighlighter;
use anyhow::Result;
use printpdf::{
    BuiltinFont, Color, Mm, Op, PdfDocument, PdfPage, PdfSaveOptions, PdfWarnMsg, Point, Pt, Rgb,
    TextItem,
};
use pulldown_cmark::{CodeBlockKind, Event, Tag, TagEnd};

// US-Letter page geometry, in PDF points (1/72").
const PAGE_W: f32 = 612.0;
const PAGE_H: f32 = 792.0;
const MARGIN: f32 = 56.0;
const CONTENT_W: f32 = PAGE_W - 2.0 * MARGIN;
const TOP_Y: f32 = PAGE_H - MARGIN;
const BOTTOM_Y: f32 = MARGIN;

// Built-in font metrics (fraction of em advance per glyph).
const COURIER_ADV: f32 = 0.6; // Courier is monospaced at 600 units/em.
const HELV_ADV: f32 = 0.5; // Rough average advance for Helvetica.

/// A single laid-out run of text sharing one colour.
struct Run {
    text: String,
    color: Color,
}

/// A line of text ready to be placed on a page.
struct Line {
    runs: Vec<Run>,
    font: BuiltinFont,
    size: f32,
    leading: f32,
    space_before: f32,
}

/// Semantic blocks parsed out of the markdown event stream.
enum Block {
    Heading {
        level: u32,
        text: String,
    },
    Paragraph(String),
    ListItem(String),
    Code {
        language: Option<String>,
        code: String,
    },
    PageBreak,
}

fn rgb(r: f32, g: f32, b: f32) -> Color {
    Color::Rgb(Rgb {
        r,
        g,
        b,
        icc_profile: None,
    })
}

fn syntect_color(c: syntect::highlighting::Color) -> Color {
    rgb(c.r as f32 / 255.0, c.g as f32 / 255.0, c.b as f32 / 255.0)
}

/// Transcode text to an ASCII-safe form. The built-in PDF fonts only encode
/// ASCII reliably, so common typographic characters are mapped to sensible
/// equivalents and anything else outside ASCII becomes '?', rather than being
/// emitted as raw UTF-8 bytes that render as mojibake.
fn ascii_safe(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '\u{2018}' | '\u{2019}' | '\u{201B}' => '\'',
            '\u{201C}' | '\u{201D}' | '\u{201F}' => '"',
            '\u{2013}' | '\u{2014}' | '\u{2212}' => '-',
            '\u{2022}' | '\u{00B7}' => '*',
            '\u{2026}' => '.', // ellipsis -> single dot keeps width sane
            '\u{00A0}' => ' ',
            c if c.is_ascii() => c,
            _ => '?',
        })
        .collect()
}

const BLACK: Color = Color::Rgb(Rgb {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    icc_profile: None,
});

pub struct PdfRenderer;

impl PdfRenderer {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
}

/// Collapse pulldown events into a flat list of semantic blocks.
fn parse_blocks(events: Vec<Event>) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut text = String::new();
    let mut heading_level: Option<u32> = None;
    let mut in_item = false;
    let mut code: Option<(Option<String>, String)> = None;

    let flush_para = |text: &mut String, blocks: &mut Vec<Block>| {
        let t = text.trim();
        if !t.is_empty() {
            blocks.push(Block::Paragraph(t.to_string()));
        }
        text.clear();
    };

    for event in events {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                flush_para(&mut text, &mut blocks);
                heading_level = Some(level as u32);
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some(level) = heading_level.take() {
                    let t = text.trim();
                    if !t.is_empty() {
                        blocks.push(Block::Heading {
                            level,
                            text: t.to_string(),
                        });
                    }
                    text.clear();
                }
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                flush_para(&mut text, &mut blocks);
                let lang = match kind {
                    CodeBlockKind::Fenced(l) if !l.is_empty() => Some(l.to_string()),
                    _ => None,
                };
                code = Some((lang, String::new()));
            }
            Event::End(TagEnd::CodeBlock) => {
                if let Some((language, body)) = code.take() {
                    blocks.push(Block::Code {
                        language,
                        code: body,
                    });
                }
            }
            Event::Start(Tag::Item) => {
                flush_para(&mut text, &mut blocks);
                in_item = true;
            }
            Event::End(TagEnd::Item) => {
                let t = text.trim();
                if !t.is_empty() {
                    blocks.push(Block::ListItem(t.to_string()));
                }
                text.clear();
                in_item = false;
            }
            Event::End(TagEnd::Paragraph) => {
                if !in_item && heading_level.is_none() {
                    flush_para(&mut text, &mut blocks);
                }
            }
            Event::Text(t) => {
                if let Some((_, body)) = code.as_mut() {
                    body.push_str(&t);
                } else if t.contains("\\newpage") {
                    let parts: Vec<&str> = t.split("\\newpage").collect();
                    for (i, part) in parts.iter().enumerate() {
                        text.push_str(part);
                        if i < parts.len() - 1 {
                            flush_para(&mut text, &mut blocks);
                            blocks.push(Block::PageBreak);
                        }
                    }
                } else {
                    text.push_str(&t);
                }
            }
            Event::Code(c) => {
                text.push('`');
                text.push_str(&c);
                text.push('`');
            }
            Event::SoftBreak | Event::HardBreak => text.push(' '),
            _ => {}
        }
    }

    let t = text.trim();
    if !t.is_empty() {
        blocks.push(Block::Paragraph(t.to_string()));
    }
    blocks
}

/// Greedy word-wrap for proportional text, measured by approximate advance.
fn wrap_words(s: &str, size: f32, max_w: f32, adv: f32) -> Vec<String> {
    let max_chars = ((max_w / (size * adv)).floor() as usize).max(8);
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in s.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
        } else if current.chars().count() + 1 + word.chars().count() <= max_chars {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(std::mem::take(&mut current));
            current.push_str(word);
        }
        // Hard-break a single word longer than the line.
        while current.chars().count() > max_chars {
            let head: String = current.chars().take(max_chars).collect();
            let tail: String = current.chars().skip(max_chars).collect();
            lines.push(head);
            current = tail;
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

/// Column-aware wrap of a highlighted code line that keeps per-token colour.
fn wrap_code_line(
    spans: &[(syntect::highlighting::Style, String)],
    max_chars: usize,
) -> Vec<Vec<Run>> {
    let mut out: Vec<Vec<Run>> = Vec::new();
    let mut row: Vec<Run> = Vec::new();
    let mut col = 0usize;
    for (style, piece) in spans {
        let color = syntect_color(style.foreground);
        let mut buf = String::new();
        for ch in piece.chars() {
            if col >= max_chars {
                if !buf.is_empty() {
                    row.push(Run {
                        text: std::mem::take(&mut buf),
                        color: color.clone(),
                    });
                }
                out.push(std::mem::take(&mut row));
                col = 0;
            }
            buf.push(ch);
            col += 1;
        }
        if !buf.is_empty() {
            row.push(Run { text: buf, color });
        }
    }
    out.push(row);
    out
}

/// Turn semantic blocks into positioned lines.
fn layout_lines(blocks: Vec<Block>, metadata: &DocumentMetadata) -> Vec<Line> {
    let mut highlighter = SyntaxHighlighter::new().ok();
    if let Some(h) = highlighter.as_mut() {
        let _ = h.set_theme(&metadata.syntax_theme);
    }

    let mut lines = Vec::new();
    for block in blocks {
        match block {
            Block::PageBreak => lines.push(Line {
                runs: vec![],
                font: BuiltinFont::Helvetica,
                size: 10.5,
                leading: 0.0,
                space_before: f32::INFINITY, // sentinel: force a new page
            }),
            Block::Heading { level, text } => {
                let size = match level {
                    1 => 22.0,
                    2 => 16.0,
                    3 => 13.0,
                    _ => 11.0,
                };
                for (i, chunk) in wrap_words(&text, size, CONTENT_W, HELV_ADV)
                    .into_iter()
                    .enumerate()
                {
                    lines.push(Line {
                        runs: vec![Run {
                            text: chunk,
                            color: BLACK,
                        }],
                        font: BuiltinFont::HelveticaBold,
                        size,
                        leading: size * 1.35,
                        space_before: if i == 0 { size * 0.8 } else { 0.0 },
                    });
                }
            }
            Block::Paragraph(text) => {
                for chunk in wrap_words(&text, 10.5, CONTENT_W, HELV_ADV) {
                    lines.push(Line {
                        runs: vec![Run {
                            text: chunk,
                            color: BLACK,
                        }],
                        font: BuiltinFont::Helvetica,
                        size: 10.5,
                        leading: 15.0,
                        space_before: 2.0,
                    });
                }
            }
            Block::ListItem(text) => {
                let chunks = wrap_words(&text, 10.5, CONTENT_W - 14.0, HELV_ADV);
                for (i, chunk) in chunks.into_iter().enumerate() {
                    // Built-in PDF fonts only encode ASCII reliably, so use a
                    // hyphen bullet rather than U+2022 (which would mojibake).
                    let prefix = if i == 0 { "-  " } else { "   " };
                    lines.push(Line {
                        runs: vec![Run {
                            text: format!("{prefix}{chunk}"),
                            color: BLACK,
                        }],
                        font: BuiltinFont::Helvetica,
                        size: 10.5,
                        leading: 14.0,
                        space_before: 1.0,
                    });
                }
            }
            Block::Code { language, code } => {
                let size = 8.5;
                let max_chars = ((CONTENT_W / (size * COURIER_ADV)).floor() as usize).max(16);
                for raw_line in code.lines() {
                    let expanded = raw_line.replace('\t', "    ");
                    // Highlight this single line (with a trailing newline for syntect).
                    let spans: Vec<(syntect::highlighting::Style, String)> = match highlighter
                        .as_ref()
                    {
                        Some(h) => {
                            let with_nl = format!("{expanded}\n");
                            h.highlight_lines(&with_nl, language.as_deref())
                                .into_iter()
                                .next()
                                .unwrap_or_default()
                                .into_iter()
                                .map(|(st, s)| (st, s.trim_end_matches('\n').to_string()))
                                .collect()
                        }
                        None => vec![(syntect::highlighting::Style::default(), expanded.clone())],
                    };
                    let spans = if spans.is_empty() {
                        vec![(syntect::highlighting::Style::default(), String::new())]
                    } else {
                        spans
                    };
                    for row in wrap_code_line(&spans, max_chars) {
                        lines.push(Line {
                            runs: row,
                            font: BuiltinFont::Courier,
                            size,
                            leading: 11.0,
                            space_before: 0.0,
                        });
                    }
                }
            }
        }
    }
    lines
}

/// Emit the ops for a page from its positioned lines, top-down.
///
/// Each line gets its own text section so that the leading `SetTextCursor`
/// is an absolute placement (the text matrix is reset at every
/// `StartTextSection`). Runs within a line then flow left-to-right, with
/// printpdf advancing the cursor from the built-in font's own metrics.
fn render_page(lines: &[PositionedLine]) -> Vec<Op> {
    let mut ops = Vec::new();
    for pl in lines {
        let line = &pl.line;
        if line.runs.iter().all(|r| r.text.is_empty()) {
            continue;
        }
        ops.push(Op::StartTextSection);
        ops.push(Op::SetFontSizeBuiltinFont {
            size: Pt(line.size),
            font: line.font,
        });
        ops.push(Op::SetTextCursor {
            pos: Point {
                x: Pt(MARGIN),
                y: Pt(pl.y),
            },
        });
        for run in &line.runs {
            if run.text.is_empty() {
                continue;
            }
            ops.push(Op::SetFillColor {
                col: run.color.clone(),
            });
            ops.push(Op::WriteTextBuiltinFont {
                items: vec![TextItem::Text(ascii_safe(&run.text))],
                font: line.font,
            });
        }
        ops.push(Op::EndTextSection);
    }
    ops
}

impl DocumentRenderer for PdfRenderer {
    fn render(&self, events: Vec<Event>, metadata: &DocumentMetadata) -> Result<Vec<u8>> {
        let blocks = parse_blocks(events);
        let mut lines = layout_lines(blocks, metadata);

        // Assign y positions and split into pages.
        let mut pages: Vec<Vec<PositionedLine>> = Vec::new();
        let mut page: Vec<PositionedLine> = Vec::new();
        let mut y = TOP_Y;
        for line in lines.drain(..) {
            if line.space_before.is_infinite() {
                // Explicit page break.
                if !page.is_empty() {
                    pages.push(std::mem::take(&mut page));
                }
                y = TOP_Y;
                continue;
            }
            let advance = line.leading.max(line.size);
            if y - line.space_before - advance < BOTTOM_Y && !page.is_empty() {
                pages.push(std::mem::take(&mut page));
                y = TOP_Y;
            }
            y -= line.space_before;
            page.push(PositionedLine { line, y });
            y -= advance;
        }
        if !page.is_empty() {
            pages.push(page);
        }
        if pages.is_empty() {
            pages.push(Vec::new());
        }

        let mut doc = PdfDocument::new(&metadata.title);
        let pdf_pages: Vec<PdfPage> = pages
            .iter()
            .map(|pl| PdfPage::new(Mm(215.9), Mm(279.4), render_page(pl)))
            .collect();

        doc.with_pages(pdf_pages);
        let mut warnings: Vec<PdfWarnMsg> = Vec::new();
        Ok(doc.save(&PdfSaveOptions::default(), &mut warnings))
    }
}

/// A line that has been assigned a vertical position on its page.
struct PositionedLine {
    line: Line,
    y: f32,
}
