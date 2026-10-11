//! Teams message bodies, both ways.
//!
//! Graph hands chat and channel messages over as an HTML fragment. Wren has
//! no browser engine, so [`parse`] reads the small subset Teams writes
//! (paragraphs, emphasis, links, code, quotes, lists, mentions, emoji,
//! inline images) into [`RichText`], which the interface lays out with egui.
//! Anything else degrades to its text. [`compose`] goes the other way: the
//! composer's light markdown becomes the HTML Graph accepts.
//!
//! The parser is a tolerant tokenizer rather than a full HTML5 parser: the
//! input is machine-written by Teams, and a hand-written loop keeps a large
//! dependency out of the binary. It never panics on malformed input.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Style {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
    pub code: bool,
    pub mention: bool,
    pub link: Option<String>,
}

impl Style {
    fn is_plain(&self) -> bool {
        *self == Self::default()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub text: String,
    #[serde(default, skip_serializing_if = "Style::is_plain")]
    pub style: Style,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Block {
    Paragraph(Vec<Span>),
    Heading(Vec<Span>),
    Quote(Vec<Span>),
    /// A list item; `Some(n)` numbers it.
    Item {
        ordered: Option<u32>,
        spans: Vec<Span>,
    },
    Code(String),
    Image {
        src: String,
        alt: Option<String>,
        width: Option<f32>,
        height: Option<f32>,
    },
    Rule,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RichText {
    pub blocks: Vec<Block>,
}

impl RichText {
    pub fn plain(text: &str) -> Self {
        let blocks = text
            .split('\n')
            .map(|line| {
                Block::Paragraph(vec![Span {
                    text: line.to_owned(),
                    style: Style::default(),
                }])
            })
            .collect();
        Self { blocks }
    }

    pub fn is_empty(&self) -> bool {
        self.blocks.iter().all(|block| match block {
            Block::Paragraph(spans)
            | Block::Heading(spans)
            | Block::Quote(spans)
            | Block::Item { spans, .. } => spans.iter().all(|span| span.text.trim().is_empty()),
            Block::Code(text) => text.trim().is_empty(),
            Block::Image { .. } | Block::Rule => false,
        })
    }

    /// The text alone, one line per block: for previews, notifications,
    /// search, and copying.
    pub fn to_plain(&self) -> String {
        let mut out = String::new();
        for block in &self.blocks {
            if !out.is_empty() {
                out.push('\n');
            }
            match block {
                Block::Paragraph(spans) | Block::Heading(spans) => push_spans(&mut out, spans),
                Block::Quote(spans) => {
                    out.push_str("> ");
                    push_spans(&mut out, spans);
                }
                Block::Item { ordered, spans } => {
                    match ordered {
                        Some(n) => out.push_str(&format!("{n}. ")),
                        None => out.push_str("• "),
                    }
                    push_spans(&mut out, spans);
                }
                Block::Code(code) => out.push_str(code),
                Block::Image { alt, .. } => out.push_str(
                    alt.as_deref()
                        .filter(|a| !a.is_empty())
                        .unwrap_or("[image]"),
                ),
                Block::Rule => out.push('—'),
            }
        }
        out
    }

    /// One line for the chat list.
    pub fn summary(&self, limit: usize) -> String {
        let flat: String = self
            .to_plain()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        truncate(&flat, limit)
    }

    /// Inline images, for lazy loading.
    pub fn images(&self) -> impl Iterator<Item = &str> {
        self.blocks.iter().filter_map(|block| match block {
            Block::Image { src, .. } => Some(src.as_str()),
            _ => None,
        })
    }
}

fn push_spans(out: &mut String, spans: &[Span]) {
    for span in spans {
        out.push_str(&span.text);
    }
}

/// Cuts `text` to at most `limit` characters, with an ellipsis.
pub fn truncate(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_owned();
    }
    let mut cut: String = text.chars().take(limit.saturating_sub(1)).collect();
    cut.push('…');
    cut
}

// ---------------------------------------------------------------------------
// Parsing

#[derive(Debug)]
enum Token<'a> {
    Text(&'a str),
    Open {
        name: String,
        attrs: Vec<(String, String)>,
        closed: bool,
    },
    Close(String),
}

/// Splits HTML into text, opening and closing tags. Comments, doctypes and
/// processing instructions are dropped.
fn tokenize(input: &str) -> Vec<Token<'_>> {
    let mut tokens = Vec::new();
    let bytes = input.as_bytes();
    let mut i = 0;
    let mut text_start = 0;
    while i < bytes.len() {
        if bytes[i] != b'<' {
            i += 1;
            continue;
        }
        // A '<' that does not start a tag is text.
        let next = bytes.get(i + 1).copied();
        let starts_tag = matches!(next, Some(b'/' | b'!' | b'?'))
            || next.is_some_and(|c| c.is_ascii_alphabetic());
        if !starts_tag {
            i += 1;
            continue;
        }
        if text_start < i {
            tokens.push(Token::Text(&input[text_start..i]));
        }
        if input[i..].starts_with("<!--") {
            let end = input[i + 4..]
                .find("-->")
                .map_or(input.len(), |at| i + 4 + at + 3);
            i = end;
            text_start = i;
            continue;
        }
        let Some(end) = find_tag_end(bytes, i + 1) else {
            // An unterminated tag: keep the rest as text.
            tokens.push(Token::Text(&input[i..]));
            text_start = input.len();
            break;
        };
        let inner = &input[i + 1..end];
        i = end + 1;
        text_start = i;
        if inner.starts_with('!') || inner.starts_with('?') {
            continue;
        }
        if let Some(name) = inner.strip_prefix('/') {
            tokens.push(Token::Close(tag_name(name)));
            continue;
        }
        let closed = inner.trim_end().ends_with('/');
        let inner = inner.trim_end().trim_end_matches('/');
        let name = tag_name(inner);
        let attrs = parse_attrs(
            &inner[inner
                .find(|c: char| c.is_whitespace())
                .unwrap_or(inner.len())..],
        );
        tokens.push(Token::Open {
            name,
            attrs,
            closed,
        });
    }
    if text_start < input.len() {
        tokens.push(Token::Text(&input[text_start..]));
    }
    tokens
}

/// The `>` that ends a tag starting at `from`, skipping quoted attribute
/// values.
fn find_tag_end(bytes: &[u8], from: usize) -> Option<usize> {
    let mut quote: Option<u8> = None;
    for (offset, &byte) in bytes[from..].iter().enumerate() {
        match quote {
            Some(q) if byte == q => quote = None,
            Some(_) => {}
            None if byte == b'"' || byte == b'\'' => quote = Some(byte),
            None if byte == b'>' => return Some(from + offset),
            None => {}
        }
    }
    None
}

fn tag_name(inner: &str) -> String {
    inner
        .trim_start()
        .split(|c: char| c.is_whitespace() || c == '/' || c == '>')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase()
}

fn parse_attrs(mut rest: &str) -> Vec<(String, String)> {
    let mut attrs = Vec::new();
    loop {
        rest = rest.trim_start();
        if rest.is_empty() {
            break;
        }
        let name_end = rest
            .find(|c: char| c == '=' || c.is_whitespace())
            .unwrap_or(rest.len());
        let name = rest[..name_end].to_ascii_lowercase();
        rest = rest[name_end..].trim_start();
        let value = if let Some(after) = rest.strip_prefix('=') {
            let after = after.trim_start();
            match after.chars().next() {
                Some(q @ ('"' | '\'')) => {
                    let body = &after[1..];
                    let close = body.find(q).unwrap_or(body.len());
                    rest = body.get(close + 1..).unwrap_or("");
                    decode_entities(&body[..close])
                }
                _ => {
                    let end = after.find(char::is_whitespace).unwrap_or(after.len());
                    rest = &after[end..];
                    decode_entities(&after[..end])
                }
            }
        } else {
            String::new()
        };
        if name.is_empty() {
            // Stray characters; skip one to make progress.
            let skip = rest.chars().next().map_or(0, char::len_utf8);
            rest = &rest[skip..];
            continue;
        }
        attrs.push((name, value));
    }
    attrs
}

fn attr<'a>(attrs: &'a [(String, String)], name: &str) -> Option<&'a str> {
    attrs
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.as_str())
}

/// Decodes character references: the named ones Teams and Outlook write,
/// and every numeric one.
pub fn decode_entities(text: &str) -> String {
    if !text.contains('&') {
        return text.to_owned();
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find('&') {
        out.push_str(&rest[..at]);
        rest = &rest[at..];
        let Some(semi) = rest[1..].find(';').map(|i| i + 1).filter(|&i| i <= 12) else {
            out.push('&');
            rest = &rest[1..];
            continue;
        };
        let entity = &rest[1..semi];
        let decoded = if let Some(number) = entity.strip_prefix('#') {
            let value = match number.strip_prefix(['x', 'X']) {
                Some(hex) => u32::from_str_radix(hex, 16).ok(),
                None => number.parse::<u32>().ok(),
            };
            value.and_then(char::from_u32).map(String::from)
        } else {
            named_entity(entity).map(String::from)
        };
        match decoded {
            Some(value) => {
                out.push_str(&value);
                rest = &rest[semi + 1..];
            }
            None => {
                out.push('&');
                rest = &rest[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

fn named_entity(name: &str) -> Option<&'static str> {
    Some(match name {
        "amp" => "&",
        "lt" => "<",
        "gt" => ">",
        "quot" => "\"",
        "apos" => "'",
        "nbsp" => "\u{a0}",
        "hellip" => "…",
        "mdash" => "—",
        "ndash" => "–",
        "lsquo" => "‘",
        "rsquo" => "’",
        "ldquo" => "“",
        "rdquo" => "”",
        "bull" => "•",
        "middot" => "·",
        "copy" => "©",
        "reg" => "®",
        "trade" => "™",
        "euro" => "€",
        "pound" => "£",
        "times" => "×",
        "rarr" => "→",
        "larr" => "←",
        "zwj" => "\u{200d}",
        "zwnj" => "\u{200c}",
        _ => return None,
    })
}

#[derive(Default)]
struct Builder {
    blocks: Vec<Block>,
    spans: Vec<Span>,
    styles: Vec<(String, Style)>,
    quote_depth: usize,
    heading: bool,
    list: Vec<Option<u32>>,
    in_item: Option<Option<u32>>,
    pre: Option<String>,
    /// Inside an `<emoji>` or Skype emoji span: its alt text has been used.
    skip_text: usize,
    /// Inside a `<style>` or `<script>` element.
    hidden: usize,
}

impl Builder {
    fn style(&self) -> Style {
        self.styles
            .last()
            .map(|(_, s)| s.clone())
            .unwrap_or_default()
    }

    fn push_style(&mut self, tag: &str, change: impl FnOnce(&mut Style)) {
        let mut style = self.style();
        change(&mut style);
        self.styles.push((tag.to_owned(), style));
    }

    fn pop_style(&mut self, tag: &str) {
        if let Some(index) = self.styles.iter().rposition(|(name, _)| name == tag) {
            self.styles.truncate(index);
        }
    }

    fn text(&mut self, raw: &str) {
        if self.hidden > 0 || self.skip_text > 0 {
            return;
        }
        let decoded = decode_entities(raw);
        if let Some(pre) = &mut self.pre {
            pre.push_str(&decoded);
            return;
        }
        // HTML whitespace collapses; a non-breaking space stays.
        let mut collapsed = String::with_capacity(decoded.len());
        let mut last_space = self
            .spans
            .last()
            .is_none_or(|span| span.text.ends_with(' '));
        for c in decoded.chars() {
            if c.is_whitespace() && c != '\u{a0}' {
                if !last_space {
                    collapsed.push(' ');
                    last_space = true;
                }
            } else {
                collapsed.push(if c == '\u{a0}' { ' ' } else { c });
                last_space = false;
            }
        }
        if collapsed.is_empty() {
            return;
        }
        self.push_text(collapsed, self.style());
    }

    fn push_text(&mut self, text: String, style: Style) {
        if let Some(last) = self.spans.last_mut()
            && last.style == style
        {
            last.text.push_str(&text);
            return;
        }
        self.spans.push(Span { text, style });
    }

    fn line_break(&mut self) {
        if let Some(pre) = &mut self.pre {
            pre.push('\n');
            return;
        }
        // A <br> ends the line but keeps the block's kind for the next one.
        self.flush();
    }

    /// Ends the current block of inline text, if it has any.
    fn flush(&mut self) {
        let mut spans = std::mem::take(&mut self.spans);
        if let Some(last) = spans.last_mut() {
            let trimmed = last.text.trim_end().len();
            last.text.truncate(trimmed);
        }
        if let Some(first) = spans.first_mut() {
            first.text = first.text.trim_start().to_owned();
        }
        spans.retain(|span| !span.text.is_empty());
        if spans.is_empty() {
            return;
        }
        let block = if let Some(ordered) = self.in_item {
            Block::Item { ordered, spans }
        } else if self.quote_depth > 0 {
            Block::Quote(spans)
        } else if self.heading {
            Block::Heading(spans)
        } else {
            Block::Paragraph(spans)
        };
        self.blocks.push(block);
    }

    fn open(&mut self, name: &str, attrs: &[(String, String)], closed: bool) {
        if self.hidden > 0 && !closed {
            if matches!(name, "style" | "script") {
                self.hidden += 1;
            }
            return;
        }
        match name {
            "style" | "script" | "title" | "head" if !closed => self.hidden += 1,
            "b" | "strong" => self.push_style(name, |s| s.bold = true),
            "i" | "em" => self.push_style(name, |s| s.italic = true),
            "u" | "ins" => self.push_style(name, |s| s.underline = true),
            "s" | "strike" | "del" => self.push_style(name, |s| s.strike = true),
            "code" if self.pre.is_none() => self.push_style(name, |s| s.code = true),
            "a" => {
                let href = attr(attrs, "href").map(str::to_owned);
                self.push_style(name, |s| s.link = href);
            }
            "at" => self.push_style(name, |s| s.mention = true),
            "span" => {
                let itemtype = attr(attrs, "itemtype").unwrap_or_default();
                if itemtype.ends_with("/Mention") {
                    self.push_style(name, |s| s.mention = true);
                } else if itemtype.ends_with("/Emoji") {
                    // The <img> inside carries the emoji in its alt.
                    self.push_style(name, |_| {});
                } else {
                    self.push_style(name, |_| {});
                }
            }
            "emoji" => {
                if let Some(alt) = attr(attrs, "alt").filter(|a| !a.is_empty()) {
                    let style = self.style();
                    self.push_text(alt.to_owned(), style);
                }
                if !closed {
                    self.skip_text += 1;
                }
            }
            "br" => self.line_break(),
            "p" | "div" | "tr" | "section" | "article" | "header" | "footer" | "table" => {
                self.flush();
            }
            "td" | "th" => {
                if !self.spans.is_empty() {
                    let style = self.style();
                    self.push_text(" · ".into(), style);
                }
            }
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                self.flush();
                self.heading = true;
            }
            "blockquote" => {
                self.flush();
                self.quote_depth += 1;
            }
            "pre" | "codeblock" => {
                self.flush();
                self.pre = Some(String::new());
            }
            "ul" => {
                self.flush();
                self.list.push(None);
            }
            "ol" => {
                self.flush();
                let start = attr(attrs, "start")
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(1);
                self.list.push(Some(start));
            }
            "li" => {
                self.flush();
                let ordered = match self.list.last_mut() {
                    Some(Some(n)) => {
                        let current = *n;
                        *n += 1;
                        Some(current)
                    }
                    _ => None,
                };
                self.in_item = Some(ordered);
            }
            "hr" => {
                self.flush();
                self.blocks.push(Block::Rule);
            }
            "img" => self.image(attrs),
            _ => {}
        }
    }

    fn image(&mut self, attrs: &[(String, String)]) {
        let alt = attr(attrs, "alt").map(str::to_owned);
        let itemtype = attr(attrs, "itemtype").unwrap_or_default();
        let src = attr(attrs, "src").unwrap_or_default();
        // Teams draws its emoji as small images whose alt is the emoji.
        let is_emoji = itemtype.ends_with("/Emoji")
            || attr(attrs, "class").is_some_and(|c| c.contains("emoji"))
            || src.contains("/emoticons/");
        if is_emoji {
            if let Some(alt) = alt.filter(|a| !a.is_empty()) {
                let style = self.style();
                self.push_text(alt, style);
            }
            return;
        }
        if src.is_empty() {
            return;
        }
        self.flush();
        let size =
            |name| attr(attrs, name).and_then(|v| v.trim_end_matches("px").parse::<f32>().ok());
        self.blocks.push(Block::Image {
            src: src.to_owned(),
            alt,
            width: size("width"),
            height: size("height"),
        });
    }

    fn close(&mut self, name: &str) {
        if self.hidden > 0 {
            if matches!(name, "style" | "script" | "title" | "head") {
                self.hidden -= 1;
            }
            return;
        }
        match name {
            "b" | "strong" | "i" | "em" | "u" | "ins" | "s" | "strike" | "del" | "a" | "at"
            | "span" => self.pop_style(name),
            "code" if self.pre.is_none() => self.pop_style(name),
            "emoji" => self.skip_text = self.skip_text.saturating_sub(1),
            "p" | "div" | "tr" | "section" | "article" | "header" | "footer" | "table" => {
                self.flush()
            }
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                self.flush();
                self.heading = false;
            }
            "blockquote" => {
                self.flush();
                self.quote_depth = self.quote_depth.saturating_sub(1);
            }
            "pre" | "codeblock" => {
                if let Some(code) = self.pre.take() {
                    let code = code.trim_matches('\n').to_owned();
                    if !code.is_empty() {
                        self.blocks.push(Block::Code(code));
                    }
                }
            }
            "li" => {
                self.flush();
                self.in_item = None;
            }
            "ul" | "ol" => {
                self.flush();
                self.list.pop();
                self.in_item = None;
            }
            _ => {}
        }
    }
}

/// Reads a Teams HTML body.
pub fn parse(html: &str) -> RichText {
    let mut builder = Builder::default();
    for token in tokenize(html) {
        match token {
            Token::Text(text) => builder.text(text),
            Token::Open {
                name,
                attrs,
                closed,
            } => builder.open(&name, &attrs, closed),
            Token::Close(name) => builder.close(&name),
        }
    }
    if let Some(code) = builder.pre.take()
        && !code.trim().is_empty()
    {
        builder.blocks.push(Block::Code(code));
    }
    builder.flush();
    RichText {
        blocks: builder.blocks,
    }
}

// ---------------------------------------------------------------------------
// Composing

pub fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

/// Turns the composer's text into the HTML body Graph sends.
///
/// Supported: `**bold**`, `*italic*` / `_italic_`, `~~strike~~`,
/// `` `code` ``, fenced code blocks, `> quotes`, `- ` and `1. ` lists, and
/// bare `http(s)://` links. Everything else is escaped text; line breaks are
/// kept.
pub fn compose(text: &str) -> String {
    let mut out = String::new();
    let mut lines = text.lines().peekable();
    let mut list: Option<&'static str> = None;
    let mut paragraph: Vec<String> = Vec::new();

    let flush_paragraph = |out: &mut String, paragraph: &mut Vec<String>| {
        if !paragraph.is_empty() {
            out.push_str("<p>");
            out.push_str(&paragraph.join("<br>"));
            out.push_str("</p>");
            paragraph.clear();
        }
    };
    let close_list = |out: &mut String, list: &mut Option<&'static str>| {
        if let Some(tag) = list.take() {
            out.push_str(&format!("</{tag}>"));
        }
    };

    while let Some(line) = lines.next() {
        if line.trim_start().starts_with("```") {
            flush_paragraph(&mut out, &mut paragraph);
            close_list(&mut out, &mut list);
            let mut code = Vec::new();
            for inner in lines.by_ref() {
                if inner.trim_start().starts_with("```") {
                    break;
                }
                code.push(escape(inner));
            }
            out.push_str("<pre><code>");
            out.push_str(&code.join("\n"));
            out.push_str("</code></pre>");
            continue;
        }
        let trimmed = line.trim_start();
        if let Some(quote) = trimmed
            .strip_prefix("> ")
            .or_else(|| (trimmed == ">").then_some(""))
        {
            flush_paragraph(&mut out, &mut paragraph);
            close_list(&mut out, &mut list);
            out.push_str("<blockquote>");
            out.push_str(&inline(quote));
            out.push_str("</blockquote>");
            continue;
        }
        let bullet = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "));
        let numbered = trimmed
            .split_once(". ")
            .filter(|(n, _)| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
            .map(|(_, rest)| rest);
        if let Some((tag, item)) = bullet.map(|b| ("ul", b)).or(numbered.map(|n| ("ol", n))) {
            flush_paragraph(&mut out, &mut paragraph);
            if list != Some(tag) {
                close_list(&mut out, &mut list);
                out.push_str(&format!("<{tag}>"));
                list = Some(tag);
            }
            out.push_str("<li>");
            out.push_str(&inline(item));
            out.push_str("</li>");
            continue;
        }
        close_list(&mut out, &mut list);
        paragraph.push(inline(line));
    }
    flush_paragraph(&mut out, &mut paragraph);
    close_list(&mut out, &mut list);
    out
}

/// Like [`compose`], with `@Name` turned into Teams mentions.
///
/// `mentions` are the people picked in the composer, in order. The first
/// `@Name` in the text for each becomes `<at id="n">Name</at>`, numbered in
/// the order they appear; people whose `@Name` was deleted are dropped.
/// Returns the HTML and the mentions it uses, indexed by their `id`.
pub fn compose_with_mentions(
    text: &str,
    mentions: &[crate::model::MentionRef],
) -> (String, Vec<crate::model::MentionRef>) {
    // Mark each mention with private-use characters that markdown and
    // escaping leave alone, then swap the marks for <at> tags.
    let mut marked = text.to_owned();
    let mut found: Vec<(usize, crate::model::MentionRef)> = Vec::new();
    for mention in mentions {
        let needle = format!("@{}", mention.name);
        if found.iter().any(|(_, m)| m.user_id == mention.user_id) {
            continue;
        }
        let Some(at) = marked.find(&needle) else {
            continue;
        };
        found.push((at, mention.clone()));
        let token = format!("\u{E000}{}\u{E001}", found.len() - 1);
        marked.replace_range(at..at + needle.len(), &token);
    }
    // Number mentions by where they appear in the text, as Teams does.
    let mut order: Vec<usize> = (0..found.len()).collect();
    order.sort_by_key(|&index| {
        marked
            .find(&format!("\u{E000}{index}\u{E001}"))
            .unwrap_or(usize::MAX)
    });
    let mut html = compose(&marked);
    let mut used = Vec::new();
    for (id, &index) in order.iter().enumerate() {
        let mention = &found[index].1;
        html = html.replace(
            &format!("\u{E000}{index}\u{E001}"),
            &format!("<at id=\"{id}\">{}</at>", escape(&mention.name)),
        );
        used.push(mention.clone());
    }
    (html, used)
}

/// Inline markup for one line: code spans first (their content is literal),
/// then emphasis and links in the rest.
fn inline(line: &str) -> String {
    let mut out = String::new();
    let mut rest = line;
    while let Some(start) = rest.find('`') {
        let Some(len) = rest[start + 1..].find('`') else {
            break;
        };
        out.push_str(&emphasis(&rest[..start]));
        out.push_str("<code>");
        out.push_str(&escape(&rest[start + 1..start + 1 + len]));
        out.push_str("</code>");
        rest = &rest[start + 1 + len + 1..];
    }
    out.push_str(&emphasis(rest));
    out
}

fn emphasis(text: &str) -> String {
    let linked = autolink(text);
    let mut out = linked;
    for (marker, tag) in [("**", "strong"), ("~~", "s"), ("*", "em"), ("_", "em")] {
        out = wrap_pairs(&out, marker, tag);
    }
    out
}

/// Escapes text and turns bare links into anchors.
fn autolink(text: &str) -> String {
    let mut out = String::new();
    for (index, word) in text.split(' ').enumerate() {
        if index > 0 {
            out.push(' ');
        }
        let trimmed = word.trim_end_matches(['.', ',', ')', '!', '?', ';', ':']);
        if (trimmed.starts_with("https://") || trimmed.starts_with("http://")) && trimmed.len() > 8
        {
            let tail = &word[trimmed.len()..];
            let href = escape(trimmed);
            out.push_str(&format!("<a href=\"{href}\">{href}</a>"));
            out.push_str(&escape(tail));
        } else {
            out.push_str(&escape(word));
        }
    }
    out
}

/// Wraps text between pairs of `marker` in `tag`, leaving anchors alone.
/// A marker must touch a word on its inner side (`*a*`, not `a * b`).
fn wrap_pairs(text: &str, marker: &str, tag: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(open) = find_marker(rest, marker, true) {
        let after = &rest[open + marker.len()..];
        let Some(close) = find_marker(after, marker, false) else {
            break;
        };
        out.push_str(&rest[..open]);
        out.push_str(&format!("<{tag}>{}</{tag}>", &after[..close]));
        rest = &after[close + marker.len()..];
    }
    out.push_str(rest);
    out
}

fn find_marker(text: &str, marker: &str, opening: bool) -> Option<usize> {
    let mut from = 0;
    let mut in_tag = false;
    let bytes = text.as_bytes();
    while from < text.len() {
        let c = bytes[from];
        if c == b'<' {
            in_tag = true;
        } else if c == b'>' {
            in_tag = false;
        }
        if !in_tag && text[from..].starts_with(marker) {
            let before = text[..from].chars().next_back();
            let after = text[from + marker.len()..].chars().next();
            let ok = if opening {
                after.is_some_and(|c| !c.is_whitespace())
                    && before.is_none_or(|c| !c.is_alphanumeric())
            } else {
                before.is_some_and(|c| !c.is_whitespace())
                    && after.is_none_or(|c| !c.is_alphanumeric())
            };
            // Underscores inside words (snake_case) are not emphasis, and an
            // inner `*` of `**` is not a single marker.
            let doubled = marker.len() == 1
                && (text[from + 1..].starts_with(marker) || text[..from].ends_with(marker));
            if ok && !doubled {
                return Some(from);
            }
        }
        from += text[from..].chars().next().map_or(1, char::len_utf8);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spans(text: &str) -> Vec<Span> {
        vec![Span {
            text: text.into(),
            style: Style::default(),
        }]
    }

    #[test]
    fn plain_paragraphs_and_breaks() {
        let rich = parse("<p>Hello <b>world</b></p><p>Second<br>line</p>");
        assert_eq!(rich.to_plain(), "Hello world\nSecond\nline");
        let Block::Paragraph(first) = &rich.blocks[0] else {
            panic!("paragraph expected");
        };
        assert!(first[1].style.bold);
    }

    #[test]
    fn whitespace_collapses_but_code_keeps_it() {
        let rich = parse("<div>a   \n  b</div><pre>x  =  1\n  y</pre>");
        assert_eq!(rich.blocks[0], Block::Paragraph(spans("a b")));
        assert_eq!(rich.blocks[1], Block::Code("x  =  1\n  y".into()));
    }

    #[test]
    fn entities_decode() {
        assert_eq!(
            decode_entities("a &amp; b &lt;c&gt; &#128512; &#x1F600; &nbsp;"),
            "a & b <c> 😀 😀 \u{a0}"
        );
        assert_eq!(decode_entities("AT&T; & &unknown;"), "AT&T; & &unknown;");
    }

    #[test]
    fn mentions_links_and_emoji() {
        let rich = parse(
            r#"<p><at id="0">Ada Lovelace</at> see <a href="https://example.com/x?a=1&amp;b=2">this</a> <emoji id="smile" alt="😄" title="Smile"></emoji></p>"#,
        );
        let Block::Paragraph(spans) = &rich.blocks[0] else {
            panic!("paragraph expected");
        };
        assert!(spans[0].style.mention);
        assert_eq!(spans[0].text, "Ada Lovelace");
        let link = spans.iter().find(|s| s.style.link.is_some()).unwrap();
        assert_eq!(
            link.style.link.as_deref(),
            Some("https://example.com/x?a=1&b=2")
        );
        assert_eq!(rich.to_plain(), "Ada Lovelace see this 😄");
    }

    #[test]
    fn skype_emoji_images_become_text() {
        let rich = parse(
            r#"<p>Nice <span itemtype="http://schema.skype.com/Emoji" itemscope=""><img src="https://statics.teams.cdn.office.net/evergreen-assets/personal-expressions/v2/assets/emoticons/like/default/20_f.png" alt="👍" itemtype="http://schema.skype.com/Emoji" style="width:20px"></span></p>"#,
        );
        assert_eq!(rich.to_plain(), "Nice 👍");
        assert_eq!(rich.images().count(), 0);
    }

    #[test]
    fn hosted_images_are_blocks() {
        let rich = parse(
            r#"<p>Look:</p><p><img src="https://graph.microsoft.com/v1.0/chats/19:x/messages/1/hostedContents/aW1n/$value" width="250" height="140" alt="image"></p>"#,
        );
        assert_eq!(rich.images().count(), 1);
        assert!(matches!(
            rich.blocks[1],
            Block::Image {
                width: Some(250.0),
                ..
            }
        ));
    }

    #[test]
    fn lists_quotes_and_headings() {
        let rich = parse(
            "<h2>Plan</h2><ol><li>One</li><li>Two</li></ol><ul><li>Dot</li></ul><blockquote>Wise</blockquote><hr>",
        );
        assert_eq!(rich.to_plain(), "Plan\n1. One\n2. Two\n• Dot\n> Wise\n—");
        assert!(matches!(rich.blocks[0], Block::Heading(_)));
    }

    #[test]
    fn malformed_input_never_panics() {
        for input in [
            "<",
            "<<<>>>",
            "<p",
            "<a href='x",
            "</b></i>text",
            "<!-- open",
            "a < b > c",
            "&",
            "&#xZZ;",
            "<img src=>",
            "<p>ünïcödé <b>ok",
            "<at>",
            "<emoji alt='x'>",
        ] {
            let _ = parse(input).to_plain();
        }
        assert_eq!(parse("a < b > c").to_plain(), "a < b > c");
    }

    #[test]
    fn style_and_script_are_dropped() {
        let rich = parse("<style>p { color: red }</style><p>visible</p><script>alert(1)</script>");
        assert_eq!(rich.to_plain(), "visible");
    }

    #[test]
    fn compose_escapes_and_formats() {
        assert_eq!(compose("a < b & c"), "<p>a &lt; b &amp; c</p>");
        assert_eq!(
            compose("**bold** and *it* and `x<y`"),
            "<p><strong>bold</strong> and <em>it</em> and <code>x&lt;y</code></p>"
        );
        assert_eq!(compose("line1\nline2"), "<p>line1<br>line2</p>");
        assert_eq!(
            compose("snake_case_name stays"),
            "<p>snake_case_name stays</p>"
        );
        assert_eq!(compose("2 * 3 * 4"), "<p>2 * 3 * 4</p>");
        assert_eq!(compose("~~gone~~"), "<p><s>gone</s></p>");
    }

    #[test]
    fn compose_links_lists_quotes_code() {
        assert_eq!(
            compose("see https://example.com/a_b_c."),
            "<p>see <a href=\"https://example.com/a_b_c\">https://example.com/a_b_c</a>.</p>"
        );
        assert_eq!(
            compose("- a\n- b\ntext"),
            "<ul><li>a</li><li>b</li></ul><p>text</p>"
        );
        assert_eq!(compose("1. a\n2. b"), "<ol><li>a</li><li>b</li></ol>");
        assert_eq!(compose("> quoted"), "<blockquote>quoted</blockquote>");
        assert_eq!(
            compose("```\nfn x() {}\n<tag>\n```"),
            "<pre><code>fn x() {}\n&lt;tag&gt;</code></pre>"
        );
    }

    #[test]
    fn compose_round_trips_through_parse() {
        let text = "Hello **team**\n- first\n- second";
        assert_eq!(
            parse(&compose(text)).to_plain(),
            "Hello team\n• first\n• second"
        );
    }

    #[test]
    fn mentions_become_at_tags_in_text_order() {
        use crate::model::MentionRef;
        let ada = MentionRef {
            user_id: "u-ada".into(),
            name: "Ada Lovelace".into(),
        };
        let grace = MentionRef {
            user_id: "u-grace".into(),
            name: "Grace <Hopper>".into(),
        };
        let gone = MentionRef {
            user_id: "u-gone".into(),
            name: "Alan Turing".into(),
        };
        let (html, used) = compose_with_mentions(
            "@Grace <Hopper> and @Ada Lovelace: **ship** it",
            &[ada.clone(), grace.clone(), gone],
        );
        assert_eq!(
            html,
            "<p><at id=\"0\">Grace &lt;Hopper&gt;</at> and <at id=\"1\">Ada Lovelace</at>: <strong>ship</strong> it</p>"
        );
        assert_eq!(used, vec![grace, ada]);
        let (plain, none) = compose_with_mentions("no mentions", &[]);
        assert_eq!(plain, "<p>no mentions</p>");
        assert!(none.is_empty());
    }

    #[test]
    fn summary_flattens_and_truncates() {
        let rich = parse("<p>one</p><p>two   three</p>");
        assert_eq!(rich.summary(100), "one two three");
        assert_eq!(rich.summary(5), "one …");
    }
}
