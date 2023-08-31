//! Change block docquote style from C (`/** ... */`) to Rust (`///`).
//!
//! `prettyplease` is pretty great at what it does. However, it emits C style block quotes.
//! This is an understandable choice; for code that humans never look at or touch, it works, and is very
//! simple to implement. Unfortunately, that doesn't work out if you ever touch the emitted code.
//!
//! The problem is that C style block comments treat their formatting naively, from the start of the line.
//! So you'll start with a comment like
//!
//! ```text
//!     /**method topline
//!
//! more text here
//!
//! */
//! ```
//!
//! Then, Rustfmt sees that and decides it's ugly, so that gets converted to
//!
//! ```text
//!     /**method topline
//!
//!     more text here
//!
//!     */
//! ```
//!
//! At this point, Rustdoc thinks that the raw contents of this documentation are
//!
//! ```text
//! method topline
//!
//!     more text here
//!
//! ```
//!
//! As Rustdoc processes doc comments with markdown, it sees those four leading spaces and thinks "aha, this wants to be an anonymous code block".
//!
//! Now `cargo test` gets involved: anonymous code blocks within Rust documentation are assumed to be Rust code which should be tested. That leads to
//! obvious problems, because now there's this doc test which fails (because `more text here` is not valid Rust) where the user doesn't expect a test at all.
//!
//! The solution is to detect C style doc comments and convert them to Rust style. If the comment looks like
//!
//! ```text
//! /// method topline
//! ///
//! /// more text here
//! ```
//!
//! then everyone is happy.

use std::{
    fmt,
    io::{self, Cursor, Write},
    str::CharIndices,
};

const START_TOKEN: &str = "/**";
const START_TOKEN_LEN: usize = START_TOKEN.len();
const END_TOKEN: &str = "*/";
const END_TOKEN_LEN: usize = END_TOKEN.len();

/// Roughly maps to `Range`: it is bounded inclusively below and exclusively above.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Span {
    start: usize,
    end: usize,
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Span { start, end } = self;
        write!(f, "{start}..{end}")
    }
}

fn maybe_start(input: &str, start: usize) -> &str {
    &input[start..(start + START_TOKEN_LEN).min(input.len())]
}

fn maybe_end(input: &str, end: usize) -> &str {
    &input[end..(end + END_TOKEN_LEN).min(input.len())]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BlockComment<'a> {
    /// What to emit on each line before the comment start.
    ///
    /// This is exactly the span from the start of the current line,
    /// to immediately before the block comment began. This treatment
    /// ensures that if we discover a C-style block comment within a
    /// Rust-style doccomment, we end up with the right output.
    ///
    /// ## Example
    ///
    /// Input:
    ///
    /// ```text
    /// /// /**some documentation*/
    /// ```
    ///
    /// `indentation` is `/// `, which ensures we produce
    ///
    /// ```text
    /// /// /// some documentation
    /// ```
    ///
    /// CRITICAL ASSUMPTION: block comments always only have repeatable text before them on the line.
    indentation: &'a str,
    /// The position of the leading `/**`
    start_token: Span,
    /// The position of the trailing `*/`
    end_token: Span,
    /// The text of the comment
    text: &'a str,
    /// The position of the start of the line before this comment.
    line_start: usize,
}

impl<'a> BlockComment<'a> {
    fn new(input: &'a str, line_start: usize, start: usize, end: usize) -> Self {
        debug_assert_eq!(maybe_start(input, start), START_TOKEN);
        debug_assert_eq!(maybe_end(input, end), END_TOKEN);

        let indentation = &input[line_start..start];
        let start_token = Span {
            start,
            end: start + START_TOKEN_LEN,
        };
        let end_token = Span {
            start: end,
            end: end + END_TOKEN_LEN,
        };
        let text = &input[start_token.end..end];

        Self {
            indentation,
            start_token,
            end_token,
            text,
            line_start,
        }
    }
}

struct BlockCommentScanner<'a> {
    input: &'a str,
    inner: CharIndices<'a>,
    line_start: usize,
}

impl<'a> BlockCommentScanner<'a> {
    fn new(input: &'a str) -> Self {
        let inner = input.char_indices();
        Self {
            input,
            inner,
            line_start: 0,
        }
    }

    /// Advance the interior iterator until `condition` is met, then perform `action`.
    ///
    /// `action` is performed 0 or 1 times, depending on whether `condition` is met.
    ///
    /// Updates `self.line_start` appropriately.
    fn advance_until<Condition, Action>(&mut self, condition: Condition, mut action: Action)
    where
        Condition: Fn(usize, char) -> bool,
        Action: FnMut(usize, char),
    {
        for (position, ch) in self.inner.by_ref() {
            if ch == '\r' || ch == '\n' {
                self.line_start = position + 1;
                continue;
            }
            if condition(position, ch) {
                action(position, ch);
                break;
            }
        }
    }
}

impl<'a> Iterator for BlockCommentScanner<'a> {
    type Item = BlockComment<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // first, find the next start token
        let mut start_token = None;
        self.advance_until(
            |start, _| maybe_start(self.input, start) == START_TOKEN,
            |start, _| start_token = Some(start),
        );
        let Some(start) = start_token else {
            return None;
        };
        // fastforward the interior iterator through the remaining characters of `start`, so we don't overlap tokens
        // failure to do so would trigger a block comment on `/**/` with an undefined inner
        for _ in 0..(START_TOKEN_LEN - 1) {
            self.inner.next();
        }
        // make a note of where the line started at the current position. (it'll probably have updated by the time we want it.)
        let line_start = self.line_start;

        // now find the next end token
        let mut end_token = None;
        self.advance_until(
            |end, _| maybe_end(self.input, end) == END_TOKEN,
            |end, _| end_token = Some(end),
        );
        let Some(end) = end_token else {
            return None;
        };
        // fastforward the interior iterator through the remaining characters of `end`, so we don't overlap tokens
        // failure to do so would trigger a second block comment on `/***/***/`, instead of a single block comment followed by `***/`.
        for _ in 0..(END_TOKEN_LEN - 1) {
            self.inner.next();
        }

        Some(BlockComment::new(self.input, line_start, start, end))
    }
}

/// Process `input` as a Rust file, converting C style block documentation comments to Rust style.
///
/// This performs no interior buffering, so in the event that writes are expensive, the
/// writer should be wrapped in a `BufWriter` or similar.
pub fn fix_block_comments<Output: Write>(input: &str, mut output: Output) -> io::Result<()> {
    let mut end_of_previous = 0;

    for block_comment in BlockCommentScanner::new(input) {
        let non_comment_text = &input[end_of_previous..block_comment.line_start];
        output.write_all(non_comment_text.as_bytes())?;

        for comment_line in block_comment.text.trim().lines() {
            write!(
                output,
                "\n{}/// {}",
                block_comment.indentation, comment_line
            )?;
        }

        end_of_previous = block_comment.end_token.end;
    }

    // there is likely relevant data after the final block comment
    let trailing_text = &input[end_of_previous..];
    output.write_all(trailing_text.as_bytes())?;

    Ok(())
}

/// Process `input` as a Rust file, converting C style block documentation comments to Rust style.
pub fn fix_block_comments_to_string(input: &str) -> io::Result<String> {
    // this overallocates in most cases, but should for typical file sizes this is fine
    let mut output = String::with_capacity(2 * input.len());
    // constructing a cursor here is safe because `fix_block_comments` only ever writes valid utf-8 to its output,
    // so we know that `output` is always a valid `String`.
    let mut cursor = unsafe { Cursor::new(output.as_mut_vec()) };
    fix_block_comments(input, &mut cursor)?;
    output.shrink_to_fit();
    Ok(output)
}
