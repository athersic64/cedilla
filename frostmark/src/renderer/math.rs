// SPDX-License-Identifier: GPL-3.0-only

use std::{cell::RefCell, collections::HashMap};

use cosmic::{
    iced::{Alignment, Length, widget::image::Handle},
    widget,
};

use crate::{
    MarkWidget,
    renderer::{ValidTheme, typst::render_typst},
    structs::{ChildData, RenderedSpan},
};

type MathKey = (u32, u32, bool, String);

thread_local! {
    static MATH_CACHE: RefCell<HashMap<MathKey, Option<Handle>>> = RefCell::default();
}

impl<'a, M: Clone + 'static, T: ValidTheme + 'a> MarkWidget<'a, M, T> {
    pub fn draw_math(&mut self, source: &str, is_display: bool, _data: ChildData) -> RenderedSpan<'a, M, T> {
        let trimmed = source.trim();
        if trimmed.is_empty() {
            return RenderedSpan::None;
        }

        let text_color = self.style.and_then(|s| s.text_color).unwrap_or_else(|| {
            let c = cosmic::theme::active().cosmic().on_bg_color();
            cosmic::iced::Color::from_rgb(c.red, c.green, c.blue)
        });
        let [r, g, b, _] = text_color.into_rgba8();

        let typst_size = if is_display {
            self.text_size * 0.95
        } else {
            self.text_size * 0.85
        };

        let key: MathKey = (
            u32::from_be_bytes([0, r, g, b]),
            typst_size.to_bits(),
            is_display,
            trimmed.to_owned(),
        );

        let entry = MATH_CACHE.with_borrow(|c| c.get(&key).cloned()).unwrap_or_else(|| {
            let typst_math = prepare_typst_math(trimmed);
            let block_flag = if is_display { "true" } else { "false" };
            let wrapped = format!(
                "#set page(width: auto, height: auto, margin: (x: 2pt, y: 2pt), fill: none)\n\
                 #set text(fill: rgb({r}, {g}, {b}), size: {typst_size}pt)\n\
                 #set math.equation(block: {block_flag})\n\
                 $ {typst_math} $"
            );

            let rendered = render_typst(&wrapped, 2.0).ok().map(|res| res.handle);

            MATH_CACHE.with_borrow_mut(|c| {
                if c.len() > 256 {
                    c.clear();
                }
                c.insert(key, rendered.clone());
            });

            rendered
        });

        match entry {
            Some(handle) => {
                if is_display {
                    cosmic::iced::widget::column![widget::image(handle).width(Length::Shrink)]
                        .width(Length::Fill)
                        .align_x(Alignment::Center)
                        .padding(cosmic::iced::Padding {
                            top: 6.0,
                            bottom: 6.0,
                            left: 0.0,
                            right: 0.0,
                        })
                        .into()
                } else {
                    // Inline math element: fixed height matching font line-height
                    let line_h = (self.text_size * 1.3).round();
                    widget::image(handle)
                        .height(Length::Fixed(line_h))
                        .into()
                }
            }
            None => {
                // Graceful fallback to formatted code text if equation compilation fails
                let display_text = if is_display {
                    format!("$${trimmed}$$")
                } else {
                    format!("${trimmed}$")
                };
                self.codeblock(display_text, self.text_size, !is_display)
            }
        }
    }
}

/// Converts a math string (LaTeX or Typst) to valid Typst equation content.
pub fn prepare_typst_math(input: &str) -> String {
    let s = input.trim();

    // Check for explicit typ/typst prefixes
    if let Some(rest) = s.strip_prefix("typ:") {
        return split_unknown_identifiers(rest.trim());
    }
    if let Some(rest) = s.strip_prefix("typst:") {
        return split_unknown_identifiers(rest.trim());
    }

    latex_to_typst(s)
}

/// Translates common LaTeX math constructs to Typst math.
pub fn latex_to_typst(latex: &str) -> String {
    let mut out = latex.to_string();

    // Handle \text{...} -> "..."
    while let Some(start) = out.find("\\text{") {
        if let Some(end) = find_matching_brace(&out[start + 6..]) {
            let inner = &out[start + 6..start + 6 + end];
            let replacement = format!("\"{inner}\"");
            out.replace_range(start..start + 6 + end + 1, &replacement);
        } else {
            break;
        }
    }

    // Handle \frac{a}{b} -> (a) / (b)
    while let Some(start) = out.find("\\frac{") {
        if let Some(end_num) = find_matching_brace(&out[start + 6..]) {
            let num = out[start + 6..start + 6 + end_num].to_string();
            let after_num = start + 6 + end_num + 1;
            let rem = &out[after_num..];
            if let Some(open_den) = rem.find('{') {
                if let Some(end_den) = find_matching_brace(&rem[open_den + 1..]) {
                    let den = rem[open_den + 1..open_den + 1 + end_den].to_string();
                    let total_end = after_num + open_den + 1 + end_den + 1;
                    let replacement = format!("({num}) / ({den})");
                    out.replace_range(start..total_end, &replacement);
                    continue;
                }
            }
        }
        break;
    }

    // Handle \sqrt[n]{x} and \sqrt{x}
    while let Some(start) = out.find("\\sqrt{") {
        if let Some(end) = find_matching_brace(&out[start + 6..]) {
            let inner = &out[start + 6..start + 6 + end];
            let replacement = format!("sqrt({inner})");
            out.replace_range(start..start + 6 + end + 1, &replacement);
        } else {
            break;
        }
    }

    // Common Greek letters (lowercase)
    let greek_lower = [
        ("\\alpha", "alpha"),
        ("\\beta", "beta"),
        ("\\gamma", "gamma"),
        ("\\delta", "delta"),
        ("\\epsilon", "epsilon"),
        ("\\varepsilon", "epsilon.alt"),
        ("\\zeta", "zeta"),
        ("\\eta", "eta"),
        ("\\theta", "theta"),
        ("\\vartheta", "theta.alt"),
        ("\\iota", "iota"),
        ("\\kappa", "kappa"),
        ("\\lambda", "lambda"),
        ("\\mu", "mu"),
        ("\\nu", "nu"),
        ("\\xi", "xi"),
        ("\\pi", "pi"),
        ("\\varpi", "pi.alt"),
        ("\\rho", "rho"),
        ("\\varrho", "rho.alt"),
        ("\\sigma", "sigma"),
        ("\\varsigma", "sigma.alt"),
        ("\\tau", "tau"),
        ("\\upsilon", "upsilon"),
        ("\\phi", "phi"),
        ("\\varphi", "phi.alt"),
        ("\\chi", "chi"),
        ("\\psi", "psi"),
        ("\\omega", "omega"),
    ];

    for (tex, typ) in greek_lower {
        out = replace_word(&out, tex, typ);
    }

    // Common Greek letters (uppercase)
    let greek_upper = [
        ("\\Gamma", "Gamma"),
        ("\\Delta", "Delta"),
        ("\\Theta", "Theta"),
        ("\\Lambda", "Lambda"),
        ("\\Xi", "Xi"),
        ("\\Pi", "Pi"),
        ("\\Sigma", "Sigma"),
        ("\\Upsilon", "Upsilon"),
        ("\\Phi", "Phi"),
        ("\\Psi", "Psi"),
        ("\\Omega", "Omega"),
    ];

    for (tex, typ) in greek_upper {
        out = replace_word(&out, tex, typ);
    }

    // Spacing
    let spaces = [
        ("\\qquad", "wide"),
        ("\\quad", "quad"),
        ("\\,", "thin"),
        ("\\;", "med"),
        ("\\!", ""),
        ("\\ ", "space"),
    ];
    for (tex, typ) in spaces {
        out = out.replace(tex, typ);
    }

    // Mathematical symbols and operators
    let symbols = [
        ("\\infty", "infinity"),
        ("\\pm", "plus.minus"),
        ("\\mp", "minus.plus"),
        ("\\times", "times"),
        ("\\cdot", "dot"),
        ("\\approx", "approx"),
        ("\\neq", "!="),
        ("\\ne", "!="),
        ("\\leq", "<="),
        ("\\le", "<="),
        ("\\geq", ">="),
        ("\\ge", ">="),
        ("\\ll", "<<"),
        ("\\gg", ">>"),
        ("\\to", "->"),
        ("\\rightarrow", "->"),
        ("\\leftarrow", "<-"),
        ("\\Rightarrow", "=>"),
        ("\\Leftarrow", "<="),
        ("\\leftrightarrow", "<->"),
        ("\\Leftrightarrow", "<=>"),
        ("\\partial", "diff"),
        ("\\nabla", "nabla"),
        ("\\hbar", "planck.reduce"),
        ("\\sum", "sum"),
        ("\\prod", "product"),
        ("\\int", "integral"),
        ("\\oint", "integral.cont"),
        ("\\lim", "lim"),
        ("\\sin", "sin"),
        ("\\cos", "cos"),
        ("\\tan", "tan"),
        ("\\cot", "cot"),
        ("\\sec", "sec"),
        ("\\csc", "csc"),
        ("\\arcsin", "arcsin"),
        ("\\arccos", "arccos"),
        ("\\arctan", "arctan"),
        ("\\sinh", "sinh"),
        ("\\cosh", "cosh"),
        ("\\tanh", "tanh"),
        ("\\ln", "ln"),
        ("\\log", "log"),
        ("\\exp", "exp"),
        ("\\det", "det"),
        ("\\dim", "dim"),
        ("\\ker", "ker"),
        ("\\left(", "("),
        ("\\right)", ")"),
        ("\\left[", "["),
        ("\\right]", "]"),
        ("\\left\\{", "{"),
        ("\\right\\}", "}"),
        ("\\{", "brace.l"),
        ("\\}", "brace.r"),
    ];

    for (tex, typ) in symbols {
        out = replace_word(&out, tex, typ);
    }

    // Convert subscript `{...}` after `_` or `^` into `(...)`
    out = convert_sub_superscripts(&out);

    // Split consecutive letters (like `mc` -> `m c`) outside quotes and known Typst keywords
    out = split_unknown_identifiers(&out);

    out
}

/// Splits unknown multi-letter identifiers (e.g. `mc` -> `m c`) outside quotes
/// so Typst treats them as separate variables rather than an unknown symbol.
pub fn split_unknown_identifiers(text: &str) -> String {
    let mut result = String::with_capacity(text.len() + 16);
    let mut chars = text.chars().peekable();
    let mut in_quotes = false;

    while let Some(c) = chars.next() {
        if c == '"' {
            in_quotes = !in_quotes;
            result.push(c);
            continue;
        }

        if in_quotes {
            result.push(c);
            continue;
        }

        if c.is_ascii_alphabetic() {
            let mut word = String::new();
            word.push(c);
            while let Some(&next) = chars.peek() {
                if next.is_ascii_alphabetic() || next == '.' {
                    word.push(chars.next().unwrap());
                } else {
                    break;
                }
            }

            if is_known_typst_symbol(&word) || word.len() == 1 {
                result.push_str(&word);
            } else {
                // Split multi-letter identifier into separate spaced single letters
                let mut first = true;
                for ch in word.chars() {
                    if !first {
                        result.push(' ');
                    }
                    result.push(ch);
                    first = false;
                }
            }
            continue;
        }

        result.push(c);
    }

    result
}

fn is_known_typst_symbol(word: &str) -> bool {
    const KNOWN: &[&str] = &[
        "alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta", "iota", "kappa",
        "lambda", "mu", "nu", "xi", "pi", "rho", "sigma", "tau", "upsilon", "phi", "chi", "psi", "omega",
        "Gamma", "Delta", "Theta", "Lambda", "Xi", "Pi", "Sigma", "Upsilon", "Phi", "Psi", "Omega",
        "sin", "cos", "tan", "cot", "sec", "csc", "arcsin", "arccos", "arctan", "sinh", "cosh", "tanh",
        "ln", "log", "exp", "det", "dim", "ker", "lim", "sum", "product", "integral", "sqrt", "root",
        "cases", "mat", "vec", "quad", "wide", "thin", "med", "space", "dot", "times", "approx", "diff",
        "nabla", "infinity", "plus.minus", "minus.plus", "brace.l", "brace.r", "dif", "planck.reduce",
        "plus", "minus", "alt", "brace", "typ", "typst"
    ];
    KNOWN.contains(&word)
}

/// Replaces a TeX command only when it appears as a distinct token (not a substring of a longer word).
fn replace_word(text: &str, tex: &str, typ: &str) -> String {
    let mut result = String::new();
    let mut remainder = text;

    while let Some(idx) = remainder.find(tex) {
        let after_idx = idx + tex.len();
        let next_char = remainder[after_idx..].chars().next();
        let is_word_char = next_char.map_or(false, |c| c.is_alphanumeric() || c == '_');

        if !is_word_char {
            result.push_str(&remainder[..idx]);
            result.push_str(typ);
            remainder = &remainder[after_idx..];
        } else {
            result.push_str(&remainder[..after_idx]);
            remainder = &remainder[after_idx..];
        }
    }
    result.push_str(remainder);
    result
}

/// Converts `_{content}` to `_(content)` and `^{content}` to `^(content)` for Typst grouping.
fn convert_sub_superscripts(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if (c == '_' || c == '^') && chars.peek() == Some(&'{') {
            result.push(c);
            chars.next(); // consume '{'
            result.push('(');
            let mut depth = 1;
            while let Some(inner) = chars.next() {
                if inner == '{' {
                    depth += 1;
                    result.push('(');
                } else if inner == '}' {
                    depth -= 1;
                    if depth == 0 {
                        result.push(')');
                        break;
                    } else {
                        result.push(')');
                    }
                } else {
                    result.push(inner);
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

fn find_matching_brace(s: &str) -> Option<usize> {
    let mut depth = 1;
    for (i, c) in s.char_indices() {
        if c == '{' {
            depth += 1;
        } else if c == '}' {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greek_letters_and_spaces() {
        let input = r"\alpha, \quad \beta, \quad \delta, \quad \gamma, \quad \epsilon, \quad \nu, \quad \theta";
        let out = latex_to_typst(input);
        assert_eq!(out, "alpha, quad beta, quad delta, quad gamma, quad epsilon, quad nu, quad theta");
    }

    #[test]
    fn test_subscripts_and_text() {
        let input = r"a_1, \quad a_2, \quad v_{\text{initial}}";
        let out = latex_to_typst(input);
        assert_eq!(out, "a_1, quad a_2, quad v_(\"initial\")");
    }

    #[test]
    fn test_fractions_and_roots() {
        let input = r"\frac{a + 1}{b - 1} + \sqrt{x}";
        let out = latex_to_typst(input);
        assert_eq!(out, "(a + 1) / (b - 1) + sqrt(x)");
    }

    #[test]
    fn test_split_unknown_identifiers() {
        assert_eq!(split_unknown_identifiers("mc^2"), "m c^2");
        assert_eq!(split_unknown_identifiers("E = mc^2"), "E = m c^2");
        assert_eq!(split_unknown_identifiers("alpha"), "alpha");
        assert_eq!(split_unknown_identifiers("typ: mc^2"), "typ: m c^2");
        assert_eq!(prepare_typst_math("mc^2"), "m c^2");
        assert_eq!(prepare_typst_math("typ: mc^2"), "m c^2");
    }
}

