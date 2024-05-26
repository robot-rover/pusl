use std::fmt;

use logos::Span;

#[derive(Debug)]
pub struct ContextSpan {
    pub start: usize,
    pub end_line: u32,
    pub len: u32,
}

impl ContextSpan {
    pub fn new(span: Span, end_line: u32) -> Self {
        Self {
            start: span.start,
            end_line,
            len: (span.end - span.start) as u32,
        }
    }
}

impl Into<Span> for ContextSpan {
    fn into(self) -> Span {
        Span {
            start: self.start,
            end: self.start + self.len as usize,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ErrorContext {
    pub file_name: String,
    pub line_num: u32,
    pub line_content: String,
    pub token_location: Span,
    pub message: String,
}

impl ErrorContext {
    pub fn new(
        file_name: String,
        last_line_num: u32,
        source: &str,
        token_location: Span,
        message: String,
    ) -> Self {
        let start_pos = source[..token_location.start]
            .rfind('\n')
            .map(|loc| loc + 1)
            .unwrap_or(0);
        let end_pos = source[token_location.end..]
            .find('\n')
            .map(|loc| loc + token_location.end)
            .unwrap_or(source.len());
        let line_content = source[start_pos..end_pos].to_string();
        let contained_lines = line_content.chars().filter(|&c| c == '\n').count() as u32;
        Self {
            file_name,
            line_num: last_line_num - contained_lines,
            line_content,
            token_location: Span {
                start: token_location.start - start_pos,
                end: token_location.end - start_pos,
            },
            message,
        }
    }
}

struct ConsoleContext {
    line_num: String,
    content: String,
    carets: String,
}

impl ConsoleContext {
    fn new(line_num: u32) -> Self {
        Self {
            line_num: line_num.to_string(),
            content: String::new(),
            carets: String::new(),
        }
    }

    fn add_char(&mut self, c: char, is_caret: bool) {
        self.content.push(c);
        if is_caret {
            self.carets.push('^');
        } else {
            self.carets.push(' ');
        }
    }

    fn get_line_num_width(&self) -> usize {
        self.line_num.len()
    }

    fn format(&self, f: &mut fmt::Formatter<'_>, line_num_width: usize) -> fmt::Result {
        writeln!(
            f,
            "{:width$} |{}",
            self.line_num,
            self.content,
            width = line_num_width
        )?;
        writeln!(f, "{:width$} |{}", "", self.carets, width = line_num_width)
    }
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            writeln!(f, "Error at {}:{}", self.file_name, self.line_num)?;
            // TODO: Print current line
            let mut current_line = self.line_num;
            let mut lines = vec![ConsoleContext::new(current_line)];
            for (idx, c) in self.line_content.char_indices() {
                if c == '\n' {
                    current_line += 1;
                    lines.push(ConsoleContext::new(current_line));
                } else {
                    let is_caret =
                        idx >= self.token_location.start && idx < self.token_location.end;
                    lines.last_mut().unwrap().add_char(c, is_caret);
                }
            }
            let line_num_width = lines
                .iter()
                .map(|line| line.get_line_num_width())
                .max()
                .unwrap();
            for line in lines {
                line.format(f, line_num_width)?;
            }
            writeln!(f, "{}", self.message)?;
        } else {
            writeln!(f, "{}:{} {}", self.file_name, self.line_num, self.message)?;
        }
        Ok(())
    }
}
