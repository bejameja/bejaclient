//! Port of `launchService.ts`'s inline `Log4jParser` class.
//!
//! Modern Minecraft's log4j2.xml config emits `<log4j:Event>...</log4j:Event>` XML fragments to
//! stdout instead of plain lines. This buffers arbitrary chunks (they don't align to XML
//! boundaries) and re-emits: plain text between events untouched, and each event reformatted as
//! `[thread/LEVEL] (logger) message` plus indented stack trace lines.

use once_cell::sync::Lazy;
use regex::Regex;

static LEVEL_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"level="([^"]+)""#).unwrap());
static THREAD_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"thread="([^"]+)""#).unwrap());
static LOGGER_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"logger="([^"]+)""#).unwrap());
// `[\s\S]` (not `.`) matches newlines too, same trick the original TS regex uses instead of a
// dotall flag.
static MESSAGE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<log4j:Message><!\[CDATA\[([\s\S]*?)\]\]></log4j:Message>").unwrap());
static THROWABLE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<log4j:Throwable><!\[CDATA\[([\s\S]*?)\]\]></log4j:Throwable>").unwrap());

#[derive(Default)]
pub struct Log4jParser {
    buf: String,
}

impl Log4jParser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn feed(&mut self, chunk: &str, mut on_line: impl FnMut(&str)) {
        self.buf.push_str(chunk);

        for _ in 0..500 {
            let Some(xs) = self.buf.find("<log4j:Event") else {
                if let Some(nl) = self.buf.rfind('\n') {
                    let head = self.buf[..nl].to_string();
                    for line in head.split('\n').filter(|l| !l.is_empty()) {
                        on_line(line);
                    }
                    self.buf.drain(..=nl);
                }
                break;
            };

            if xs > 0 {
                let head = self.buf[..xs].to_string();
                for line in head.split('\n').filter(|l| !l.is_empty()) {
                    on_line(line);
                }
                self.buf.drain(..xs);
            }

            let Some(xe) = self.buf.find("</log4j:Event>") else { break };
            let end = xe + "</log4j:Event>".len();
            let xml: String = self.buf.drain(..end).collect();

            let level = LEVEL_RE.captures(&xml).map(|c| c[1].to_string()).unwrap_or_else(|| "INFO".to_string());
            let thread = THREAD_RE.captures(&xml).map(|c| c[1].to_string()).unwrap_or_else(|| "main".to_string());
            let logger_full = LOGGER_RE.captures(&xml).map(|c| c[1].to_string()).unwrap_or_default();
            let logger = logger_full.rsplit('.').next().unwrap_or("MC").to_string();
            let msg = MESSAGE_RE.captures(&xml).map(|c| c[1].trim().to_string());
            let trace = THROWABLE_RE.captures(&xml).map(|c| c[1].to_string());

            if let Some(msg) = &msg {
                on_line(&format!("[{thread}/{level}] ({logger}) {msg}"));
            }
            if let Some(trace) = &trace {
                for l in trace.split('\n').filter(|l| !l.is_empty()) {
                    on_line(&format!("  {}", l.trim()));
                }
            }
        }
    }
}
