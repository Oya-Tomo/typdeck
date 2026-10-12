//! Speaker notes written in Markdown, one section per slide.
//!
//! Sections are separated by lines consisting only of `---`. Such lines inside
//! fenced code blocks are left alone.

use pulldown_cmark::{Options, Parser, html};

const SEPARATOR: &str = "---\n";

/// The note for one slide.
#[derive(Clone, Default)]
pub struct Note {
    pub markdown: String,
    /// The rendered Markdown. Notes are written by the presenter, so their
    /// HTML is trusted.
    pub html: String,
}

impl Note {
    fn new(markdown: String) -> Self {
        let mut html = String::new();
        html::push_html(&mut html, Parser::new_ext(&markdown, Options::all()));
        Self { markdown, html }
    }
}

pub fn parse(text: &str) -> Vec<Note> {
    split(text).into_iter().map(Note::new).collect()
}

/// Replaces the section of the slide at `index` in the notes `text`, adding
/// empty sections for any slides in between.
pub fn replace(text: &str, index: usize, markdown: &str) -> String {
    let mut sections = split(text);
    if sections.len() <= index {
        sections.resize(index + 1, String::new());
    }

    let markdown = markdown.trim_end();
    sections[index] = if markdown.is_empty() {
        String::new()
    } else {
        format!("{markdown}\n")
    };
    sections.join(SEPARATOR)
}

/// Splits notes into sections that, joined by [`SEPARATOR`], reproduce the
/// text up to a missing trailing newline.
fn split(text: &str) -> Vec<String> {
    let mut sections = vec![String::new()];
    let mut fence: Option<&str> = None;

    for line in text.lines() {
        let trimmed = line.trim();
        match fence {
            Some(marker) if trimmed.starts_with(marker) => fence = None,
            Some(_) => {}
            None if trimmed == SEPARATOR.trim_end() => {
                sections.push(String::new());
                continue;
            }
            None => {
                fence = ["```", "~~~"]
                    .into_iter()
                    .find(|marker| trimmed.starts_with(marker));
            }
        }

        let section = sections.last_mut().expect("sections is never empty");
        section.push_str(line);
        section.push('\n');
    }

    sections
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_on_separator_lines() {
        assert_eq!(split("a\n---\nb\n --- \nc"), ["a\n", "b\n", "c\n"]);
    }

    #[test]
    fn keeps_separators_inside_code_fences() {
        assert_eq!(split("```\n---\n```\n---\nb"), ["```\n---\n```\n", "b\n"]);
    }

    #[test]
    fn renders_markdown() {
        assert_eq!(parse("**hi**")[0].html, "<p><strong>hi</strong></p>\n");
    }

    #[test]
    fn replaces_a_section() {
        assert_eq!(
            replace("a\n---\nb\n---\nc\n", 1, "B\n\n"),
            "a\n---\nB\n---\nc\n"
        );
    }

    #[test]
    fn adds_missing_sections() {
        assert_eq!(replace("a\n", 2, "c"), "a\n---\n---\nc\n");
    }
}
