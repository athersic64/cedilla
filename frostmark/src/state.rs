//
// Original code by: Mrmayman <navneetkrishna22@gmail.com>
// https://github.com/Mrmayman/frostmark
//

use std::{
    collections::{HashMap, HashSet},
};

use html5ever::{ParseOpts, tendril::TendrilSink};
use markup5ever_rcdom::RcDom;

use crate::structs::{UpdateMsg, UpdateMsgKind};

/// The state of the document.
///
/// - Put this in your Application struct.
/// - Use [`Self::with_html`] and [`Self::with_html_and_markdown`]
///   functions to create this.
/// - Create a new one if the document changes
///
/// ```no_run
/// # use frostmark::MarkState;
/// # const YOUR_TEXT: &str = "";
/// # fn e() { let m =
/// MarkState::with_html_and_markdown(YOUR_TEXT)
/// # ;
/// // or if you just want HTML
/// # let m =
/// MarkState::with_html(YOUR_TEXT)
/// # ; }
/// ```
pub struct MarkState {
    pub(crate) dom: RcDom,

    pub(crate) selection_state: HashMap<String, widgets::text_editor::Content>,
    pub(crate) dropdown_state: HashMap<usize, bool>,
}

impl MarkState {
    /// Processes documents containing **pure HTML**,
    /// without any Markdown support.
    ///
    /// Use this if you prioritize performance and
    /// don't need Markdown support,
    /// or if you want to avoid potential artifacts
    /// from mixing HTML and Markdown.
    #[must_use]
    #[allow(clippy::missing_panics_doc)] // Will never panic
    pub fn with_html(input: &str) -> Self {
        let dom = html5ever::parse_document(RcDom::default(), ParseOpts::default())
            .from_utf8()
            .read_from(&mut input.as_bytes())
            // Will not panic as reading from &[u8] cannot fail
            .unwrap();

        let mut selection_state = HashMap::new();
        let mut dropdown_state = HashMap::new();
        let mut dropdown_counter = 0;

        find_state(
            &dom.document,
            &mut selection_state,
            &mut dropdown_state,
            &mut dropdown_counter,
            false,
        );

        Self {
            dom,
            selection_state,
            dropdown_state,
        }
    }

    /// Processes documents containing both
    /// **HTML and Markdown** (or a mix of both).
    ///
    /// Use this method when you need to support
    /// Markdown formatting. However, note that
    /// it may introduce formatting bugs when
    /// dealing with pure HTML documents.
    #[must_use]
    #[cfg(feature = "markdown")]
    pub fn with_html_and_markdown(input: &str) -> Self {
        let normalized = normalize_math_delimiters(input);
        let html = comrak::markdown_to_html(
            &normalized,
            &comrak::Options {
                extension: comrak::options::Extension {
                    strikethrough: true,
                    cjk_friendly_emphasis: true,
                    tasklist: true,
                    superscript: true,
                    subscript: true,
                    underline: true,
                    table: true,
                    math_dollars: true,
                    math_code: true,
                    ..Default::default()
                },
                parse: comrak::options::Parse::default(),
                render: comrak::options::Render {
                    // Our renderer doesn't have the
                    // vulnerabilities of a browser
                    r#unsafe: true,
                    ..Default::default()
                },
            },
        );

        Self::with_html(&html)
    }

    /// Processes documents containing **pure Markdown**,
    /// filtering out any HTML content.
    ///
    /// Useful for things like messaging apps.
    #[must_use]
    #[cfg(feature = "markdown")]
    pub fn with_markdown_only(input: &str) -> Self {
        let mut out = String::new();
        _ = comrak::html::escape(&mut out, input);
        Self::with_html_and_markdown(&out)
    }

    /// Updates the internal state of the document.
    ///
    /// Call this method after receiving an update message
    /// from [`crate::MarkWidget::on_updating_state`].
    /// It currently handles the update of text selection
    /// within code blocks, but additional use cases may be
    /// supported in the future.
    pub fn update(&mut self, action: UpdateMsg) {
        match action.kind {
            UpdateMsgKind::TextEditor(code, action) => {
                if !action.is_edit()
                    && let Some(n) = self.selection_state.get_mut(&code)
                {
                    n.perform(action);
                }
            }
            UpdateMsgKind::DetailsToggle(id, action) => {
                self.dropdown_state.insert(id, action);
            }
        }
    }

    /// Retrieves all image URLs that need to be loaded, returned as a [`HashSet<String>`].
    ///
    /// This method gathers all image URLs in the document, which you can:
    /// 1. Download somehow (pass to an async downloader maybe?)
    /// 2. Store using, if SVG image, `iced::widget::svg::Handle::from_memory`.
    ///    - For normal images: `iced::widget::image::Handle::from_bytes`.
    /// 3. Handle the rendering of these images via [`crate::MarkWidget::on_drawing_image`].
    #[must_use]
    pub fn find_image_links(&self) -> HashSet<String> {
        let mut storage = HashSet::new();
        find_image_links(&self.dom.document, &mut storage);
        storage
    }
}

impl Default for MarkState {
    fn default() -> Self {
        Self::with_html("")
    }
}

fn find_state(
    node: &markup5ever_rcdom::Node,
    selection_state: &mut HashMap<String, widgets::text_editor::Content>,
    dropdown_state: &mut HashMap<usize, bool>,
    dropdown_counter: &mut usize,
    scan_text: bool,
) {
    let borrow = node.children.borrow();
    match &node.data {
        markup5ever_rcdom::NodeData::Element { name, .. } if &name.local == "code" => {
            for child in &*borrow {
                find_state(
                    child,
                    selection_state,
                    dropdown_state,
                    dropdown_counter,
                    true,
                );
            }
        }
        markup5ever_rcdom::NodeData::Element { name, .. } if &name.local == "details" => {
            dropdown_state.insert(*dropdown_counter, false);
            *dropdown_counter += 1;
            for child in &*borrow {
                find_state(
                    child,
                    selection_state,
                    dropdown_state,
                    dropdown_counter,
                    false,
                );
            }
        }
        markup5ever_rcdom::NodeData::Text { contents } if scan_text => {
            let contents = contents.borrow().to_string();
            let trimmed = contents.trim_end_matches('\n').to_string();
            let v = widgets::text_editor::Content::with_text(&trimmed);
            selection_state.insert(contents.clone(), v);
        }
        _ => {
            for child in &*borrow {
                find_state(
                    child,
                    selection_state,
                    dropdown_state,
                    dropdown_counter,
                    scan_text,
                );
            }
        }
    }
}

fn find_image_links(node: &markup5ever_rcdom::Node, storage: &mut HashSet<String>) {
    let borrow = node.children.borrow();
    match &node.data {
        markup5ever_rcdom::NodeData::Element { name, attrs, .. } if &name.local == "img" => {
            let attrs = attrs.borrow();
            if let Some(attr) = attrs.iter().find(|attr| &*attr.name.local == "src") {
                let url = &*attr.value;
                if !url.is_empty() {
                    storage.insert(url.to_owned());
                }
            }
        }
        _ => {
            for child in &*borrow {
                find_image_links(child, storage);
            }
        }
    }
}

/// Normalizes math delimiters for Comrak CommonMark parser:
/// 1. LaTeX delimiters: `\[ ... \]` -> `$$ ... $$` and `\( ... \)` -> `$ ... $`
/// 2. Preserves code fences (``` or ~~~) and inline code (`...`) verbatim.
/// 3. Preserves escaped dollar signs (`\$`) verbatim so they render as literal `$`.
/// 4. Typst math delimiters:
///    - `$ ... $` with surrounding whitespace (space, tab, or newline) -> `$$ ... $$` (display block math)
///    - `$x$`, `$5-x$` without whitespace -> `$x$` (inline math)
///    - Asymmetric spaces are trimmed so Comrak recognizes the inline math.
pub fn normalize_math_delimiters(input: &str) -> String {
    let mut result = String::with_capacity(input.len() + 32);
    let chars: Vec<char> = input.chars().collect();
    let n = chars.len();
    let mut i = 0;

    let mut in_code_fence: Option<(char, usize)> = None; // (fence_char, count)
    let mut in_inline_code = false;

    while i < n {
        let c = chars[i];

        // Check for code fences (``` or ~~~)
        if (c == '`' || c == '~') && !in_inline_code {
            let fence_char = c;
            let mut count = 0;
            while i + count < n && chars[i + count] == fence_char {
                count += 1;
            }

            if count >= 3 {
                if let Some((fc, fc_count)) = in_code_fence {
                    if fc == fence_char && count >= fc_count {
                        in_code_fence = None;
                    }
                } else {
                    in_code_fence = Some((fence_char, count));
                }
                for _ in 0..count {
                    result.push(fence_char);
                }
                i += count;
                continue;
            }
        }

        // Inside code fence, copy verbatim
        if in_code_fence.is_some() {
            result.push(c);
            i += 1;
            continue;
        }

        // Check for inline code `...`
        if c == '`' {
            in_inline_code = !in_inline_code;
            result.push('`');
            i += 1;
            continue;
        }

        // Inside inline code, copy verbatim
        if in_inline_code {
            result.push(c);
            i += 1;
            continue;
        }

        // Escaped characters (\...)
        if c == '\\' && i + 1 < n {
            let next = chars[i + 1];
            // Handle \[ and \] -> $$
            if next == '[' || next == ']' {
                result.push_str("$$");
                i += 2;
                continue;
            }
            // Handle \( and \) -> $
            if next == '(' || next == ')' {
                result.push('$');
                i += 2;
                continue;
            }
            // Handle \$ -> escaped dollar! Preserve verbatim so CommonMark outputs literal $
            if next == '$' {
                result.push('\\');
                result.push('$');
                i += 2;
                continue;
            }
            // Other escaped character
            result.push('\\');
            result.push(next);
            i += 2;
            continue;
        }

        // Check for $$ (already display math)
        if c == '$' && i + 1 < n && chars[i + 1] == '$' {
            result.push_str("$$");
            i += 2;
            continue;
        }

        // Check for single $ delimiter (unescaped)
        if c == '$' {
            // Find matching closing unescaped $
            let mut j = i + 1;
            let mut found_close = None;
            while j < n {
                if chars[j] == '\\' && j + 1 < n {
                    // skip escaped char
                    j += 2;
                    continue;
                }
                if chars[j] == '`' {
                    // Backtick inside potential math? Stop
                    break;
                }
                if chars[j] == '$' {
                    if j + 1 < n && chars[j + 1] == '$' {
                        // Found $$ which is not a single $, skip
                        j += 2;
                        continue;
                    }
                    found_close = Some(j);
                    break;
                }
                j += 1;
            }

            if let Some(close_idx) = found_close {
                let inner_chars = &chars[i + 1..close_idx];
                if !inner_chars.is_empty() {
                    let first = inner_chars[0];
                    let last = inner_chars[inner_chars.len() - 1];
                    let starts_ws = first.is_whitespace();
                    let ends_ws = last.is_whitespace();

                    if starts_ws && ends_ws {
                        // Typst block equation syntax: $ <content> $
                        // Normalize to display math $$ <content> $$
                        result.push_str("$$");
                        for &ch in inner_chars {
                            result.push(ch);
                        }
                        result.push_str("$$");
                        i = close_idx + 1;
                        continue;
                    } else if starts_ws || ends_ws {
                        // Asymmetric whitespace: trim so Comrak can parse as inline math $content$
                        let inner_str: String = inner_chars.iter().collect();
                        let trimmed = inner_str.trim();
                        result.push('$');
                        result.push_str(trimmed);
                        result.push('$');
                        i = close_idx + 1;
                        continue;
                    } else {
                        // Clean inline math: $content$
                        result.push('$');
                        for &ch in inner_chars {
                            result.push(ch);
                        }
                        result.push('$');
                        i = close_idx + 1;
                        continue;
                    }
                }
            }
        }

        result.push(c);
        i += 1;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_escaped_dollars() {
        let input = r"El café cuesta \$5 y el té cuesta \$10.";
        let out = normalize_math_delimiters(input);
        assert_eq!(out, r"El café cuesta \$5 y el té cuesta \$10.");
    }

    #[test]
    fn test_normalize_typst_block_math() {
        let input = "$ x + y = z $";
        let out = normalize_math_delimiters(input);
        assert_eq!(out, "$$ x + y = z $$");

        let multiline = "$\n  E = mc^2\n$";
        let out = normalize_math_delimiters(multiline);
        assert_eq!(out, "$$\n  E = mc^2\n$$");
    }

    #[test]
    fn test_normalize_inline_math() {
        let input = "Formula $E=mc^2$ and $5-x$ here.";
        let out = normalize_math_delimiters(input);
        assert_eq!(out, "Formula $E=mc^2$ and $5-x$ here.");
    }

    #[test]
    fn test_normalize_latex_delimiters() {
        let input = r"Inline \( x + 1 \) and display \[ \int_0^1 x dx \]";
        let out = normalize_math_delimiters(input);
        assert_eq!(out, r"Inline $ x + 1 $ and display $$ \int_0^1 x dx $$");
    }

    #[test]
    fn test_normalize_code_fences_preserved() {
        let input = "```rust\nlet x = \"$ 5 $\";\n```";
        let out = normalize_math_delimiters(input);
        assert_eq!(out, input);
    }
}


