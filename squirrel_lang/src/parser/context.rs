use std::{fmt, num::{ParseFloatError, ParseIntError}};

use logos::{Lexer, Logos, Span, Source};

use super::lexer::Token;

#[derive(Debug)]
pub struct LexerContext {
    line_num: usize,
    line_start_pos: usize,
}

impl Default for LexerContext {
    fn default() -> Self {
        Self {
            line_num: 0,
            line_start_pos: 0,
        }
    }
}

impl LexerContext {
    pub fn log_newline(&mut self, offset: usize) {
        self.line_num += 1;
        self.line_start_pos = offset + 1;
    }
}


#[derive(Clone, Debug, PartialEq)]
pub struct ErrorContext {
    pub file_name: String,
    pub line_num: usize,
    pub line_content: String,
    pub token_location: Span,
    pub message: String,
}

impl ErrorContext {
    pub fn new(file_name: String, line_num: usize, line_start_pos: usize, source: &str, token_location: Span, message: String) -> Self {
        debug_assert!(token_location.start >= line_start_pos);
        let end_pos = source[token_location.end..].find('\n').map(|loc| loc + token_location.end).unwrap_or(source.len());
        let line_content = source[line_start_pos..end_pos].to_string();
        Self {
            file_name,
            line_num,
            line_content,
            token_location: Span { start: token_location.start - line_start_pos, end: token_location.end - line_start_pos },
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
    fn new(line_num: usize) -> Self {
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
        writeln!(f, "{:width$} |{}", self.line_num, self.content, width = line_num_width)?;
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
                    let is_caret = idx >= self.token_location.start && idx < self.token_location.end;
                    lines.last_mut().unwrap().add_char(c, is_caret);
                }
            }
            let line_num_width = lines.iter().map(|line| line.get_line_num_width()).max().unwrap();
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


#[derive(Debug, Clone, PartialEq)]
pub enum LexerError {
    ParseIntError(ParseIntError),
    ParseFloatError(ParseFloatError),
    UnknownToken,
    General(String)
}

impl Default for LexerError {
    fn default() -> Self {
        LexerError::UnknownToken
    }
}

impl LexerError {
    pub fn with_context<'s>(self, lexer: &Lexer<'s, Token>, file_name: String) -> ErrorContext {
        let token_location = lexer.span();
        let line_start_pos = lexer.extras.line_start_pos;
        let source = lexer.source();
        let message = match self {
            LexerError::ParseIntError(err) => format!("Failed to parse integer literal: {}", err),
            LexerError::ParseFloatError(err) => format!("Failed to parse float literal: {}", err),
            LexerError::UnknownToken => "Unknown token".to_string(),
            LexerError::General(msg) => msg,
        };
        ErrorContext::new(file_name, lexer.extras.line_num, line_start_pos, source, token_location, message)
    }
}

impl From<ParseIntError> for LexerError {
    fn from(err: ParseIntError) -> Self {
        LexerError::ParseIntError(err)
    }
}

impl From<ParseFloatError> for LexerError {
    fn from(err: ParseFloatError) -> Self {
        LexerError::ParseFloatError(err)
    }
}

pub(crate) trait LexerResultContext<T>
where Self: Sized {
    fn with_context<'s>(self, lexer: &Lexer<'s, Token>, file_name: &str) -> Result<T, ErrorContext>;
}

pub type LexerResult<T> = Result<T, LexerError>;
impl<T> LexerResultContext<T> for LexerResult<T> {
    fn with_context<'s>(self, context: &Lexer<'s, Token>, file_name: &str) -> Result<T, ErrorContext> {
        self.map_err(|err| err.with_context(context, file_name.to_string()))
    }
}

enum ParserError {
    UnexpectedToken(Token, Span),
}
