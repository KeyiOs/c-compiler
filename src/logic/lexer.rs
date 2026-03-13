use std::iter::Peekable;
use std::str::Chars;

use crate::data::TokenType;
use crate::data::definitions::{DOUBLE_OPERATOR_MAP, KEYWORD_MAP, SINGLE_OPERATOR_MAP, TRIPLE_OPERATOR_MAP, Type};
use crate::error::{LexerError, LexerResult};


#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub line: u16,
}

impl Token {
    #[inline]
    pub fn new(token_type: TokenType, line: u16) -> Self {
        Self { token_type, line }
    }
}


pub fn lexer_start(source: &str) -> LexerResult<Vec<Token>> {
    /**/ /* * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * */ /**/
    /**/ /*                    Lexer State Variables                    */ /**/
    /**/ /* * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * */ /**/
    /**/ let mut token: Vec<Token> = Vec::new();                           /**/
    /**/ let mut chars: Peekable<Chars<'_>> = source.chars().peekable();   /**/
    /**/ let mut buffer: String = String::new();                           /**/
    /**/ let mut start_of_line: bool = true;                               /**/
    /**/ let mut has_decimal: bool = false;                                /**/
    /**/ let mut line: u16 = 1;                                            /**/
    /**/ /* * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * */ /**/
    

    while let Some(character) = chars.next() {
        if character.is_ascii_alphabetic() || character == '_' {
            buffer.push(character);

            while let Some(&next_char) = chars.peek() {
                if !next_char.is_ascii_alphanumeric() && next_char != '_' {
                    break;
                }

                buffer.push(chars.next().unwrap());
            }

            if KEYWORD_MAP.contains_key(buffer.as_str()) {
                token.push(Token::new(TokenType::Keyword(buffer.clone()), line));
            } else {
                token.push(Token::new(TokenType::Identifier(buffer.clone()), line));
            }
            
            buffer.clear();
        } else if character.is_ascii_digit() || ((character == '.' || character == '-') && chars.peek().map_or(false, |c| c.is_ascii_digit())) {
            match (character, chars.peek()) {
                ('0', Some(&next_char)) if is_octal(next_char) => {
                    let mut oct_digits = String::new();

                    while let Some(&c) = chars.peek() {
                        if !is_octal(c) {
                            break;
                        }

                        chars.next();
                        oct_digits.push(c);
                    }

                    let oct_value = u32::from_str_radix(&oct_digits, 8)
                        .map_err(|_| LexerError::InvalidOctalNumber { value: oct_digits.clone(), line })?;

                    token.push(Token::new(TokenType::Literal(oct_value.to_string(), Type::Int), line));
                    start_of_line = false;

                    continue;
                } ('0', Some(&next_char)) if next_char == 'x' || next_char == 'X' => {
                    let mut hex_digits = String::new();

                    chars.next();
                    while let Some(&c) = chars.peek() {
                        if !c.is_ascii_hexdigit() {
                            break;
                        }

                        hex_digits.push(c);
                        chars.next();
                    }

                    if hex_digits.is_empty() {
                        return Err(LexerError::EmptyHexNumber { line });
                    }

                    let hex_value = u32::from_str_radix(&hex_digits, 16)
                        .map_err(|_| LexerError::InvalidHexNumber { value: hex_digits.clone(), line })?;

                    token.push(Token::new(TokenType::Literal(hex_value.to_string(), Type::Int), line));
                    start_of_line = false;

                    continue;
                } _ => {}
            }

            if character == '.' {
                buffer.push('0');
                has_decimal = true;
            }

            buffer.push(character);

            while let Some(&next_char) = chars.peek() {
                if next_char.is_ascii_digit() {
                    buffer.push(chars.next().unwrap());
                } else if next_char == '.' {
                    if has_decimal {
                        return Err(LexerError::MultipleDecimalPoints { line });
                    }

                    has_decimal = true;
                    buffer.push(chars.next().unwrap());
                } else if next_char == 'e' || next_char == 'E' {
                    buffer.push(chars.next().unwrap());

                    if matches!(chars.peek(), Some(&c) if c == '+' || c == '-') {
                        buffer.push(chars.next().unwrap());
                    }

                    let buffer_length = buffer.len();
                    while let Some(&exp_digit) = chars.peek() {
                        if !exp_digit.is_ascii_digit() {
                            break;
                        }

                        buffer.push(chars.next().unwrap());
                    }

                    if buffer_length == buffer.len() {
                        return Err(LexerError::InvalidExponent { line });
                    }
                } else if next_char.is_ascii_alphabetic() {
                    let mut suffix = String::new();

                    for _ in 0..3 {
                        let Some(&c) = chars.peek() else {
                            break;
                        };

                        let c = c.to_ascii_lowercase();

                        if c != 'u' && c != 'l' && c != 'f' && c != 'd' {
                            break;
                        }

                        suffix.push(chars.next().unwrap());
                    }

                    if let Some(&next) = chars.peek() {
                        if !next.is_whitespace() && !SINGLE_OPERATOR_MAP.contains_key(&next) {
                            return Err(LexerError::InvalidNumberSuffix { line });
                        }
                    }

                    if !matches!(suffix.to_ascii_lowercase().as_str(), "u" | "d" | "l" | "f" | "ul" | "ll" | "ull" | "lf" | "ld") {
                        return Err(LexerError::InvalidNumberSuffix { line });
                    }

                    buffer.push_str(&suffix);
                } else {
                    break;
                }
            }

            let literal_type = if buffer.chars().last().map_or(false, |c| c.is_ascii_alphabetic()) {
                let suffix = buffer.to_ascii_lowercase();

                if suffix.ends_with("ull") {
                    Type::Unsigned(Box::new(Type::LongLong))
                } else if suffix.ends_with("ul") {
                    Type::Unsigned(Box::new(Type::Long))
                } else if suffix.ends_with("ll") {
                    Type::LongLong
                } else if suffix.ends_with("lf"){
                    Type::LongFloat
                } else if suffix.ends_with("ld") {
                    Type::LongDouble
                } else if suffix.ends_with("u") {
                    Type::Unsigned(Box::new(Type::Int))
                } else if suffix.ends_with("d") {
                    Type::Double
                } else if suffix.ends_with("l") {
                    Type::Long
                } else if suffix.ends_with('f') {
                    Type::Float
                } else {
                    Type::Int
                }
            } else if has_decimal {
                Type::Double
            } else {
                Type::Int
            };

            token.push(Token::new(TokenType::Literal(buffer.clone(), literal_type), line));
            has_decimal = false;
            
            buffer.clear();
        } else if character.is_whitespace() {
            if character == '\n' {
                line += 1;
                start_of_line = true;
            } else {
                start_of_line = false;
            }

            continue;
        } else if SINGLE_OPERATOR_MAP.contains_key(&character) {
            if character == '\'' {
                let Some(character) = chars.next() else {
                    return Err(LexerError::UnterminatedCharLiteral { line });
                };

                if character == '\'' {
                    return Err(LexerError::EmptyCharLiteral { line });
                } else if character == '\\' {
                    if chars.peek().is_none() {
                        return Err(LexerError::UnterminatedCharLiteral { line });
                    };

                    let char_literal = process_escape_sequence(&mut chars, line)?;

                    token.push(Token::new(TokenType::Literal(char_literal, Type::Char), line));
                } else {    
                    token.push(Token::new(TokenType::Literal(character.to_string(), Type::Char), line));
                }

                if chars.next() != Some('\'') {
                    while let Some(next_char) = chars.next() {
                        if next_char == '\n' {
                            return Err(LexerError::UnterminatedCharLiteral { line });
                        } else if next_char == '\'' {
                            return Err(LexerError::CharLiteralTooLong { line });
                        }
                    }

                    return Err(LexerError::UnterminatedCharLiteral { line });
                }
            } else if character == '\"' {
                let mut string_lit = String::new();
                let mut ok = false;

                while let Some(next_char) = chars.next() {
                    match next_char {
                        '"' => {
                            let mut temp_iter = chars.clone();
                            let mut skip_count = 0;
                            let mut found_quote = false;

                            while let Some(&c) = temp_iter.peek() {
                                if !c.is_whitespace() {
                                    if c == '"' {
                                        found_quote = true;
                                    }
                                    break;
                                }
                                skip_count += 1;
                                temp_iter.next();
                            }

                            if !found_quote {
                                token.push(Token::new(TokenType::Literal(string_lit, Type::Pointer(Box::new(Type::Char))), line));
                                ok = true;
                                break;
                            }

                            for _ in 0..skip_count {
                                chars.next();
                            }
                            chars.next();

                            continue;
                        } '\\' => {
                            let Some(&escape_char) = chars.peek() else {
                                return Err(LexerError::UnterminatedStringLiteral { line });
                            };

                            if escape_char == '\n' || escape_char == '\r' {
                                chars.next();
                                chars.next();

                                line += 1;

                                continue;
                            }

                            process_escape_sequence(&mut chars, line)?;
                        }

                        '\n' => return Err(LexerError::UnterminatedStringLiteral { line }),

                        _ => string_lit.push(next_char),
                    }
                }

                if !ok {
                    return Err(LexerError::UnterminatedStringLiteral { line });
                }
            } else if let Some(second_char) = chars.peek().copied() {
                let mut lookahead = chars.clone();
                lookahead.next();

                if let Some(third_char) = lookahead.peek() {
                    let triple_symbol = format!("{}{}{}", character, second_char, third_char);

                    if TRIPLE_OPERATOR_MAP.contains_key(triple_symbol.as_str()) {
                        for _ in 0..2 {
                            chars.next();
                        }

                        token.push(Token::new(TokenType::Operator(triple_symbol), line));
                        start_of_line = false;

                        continue;
                    }
                }

                let double_symbol = format!("{}{}", character, second_char);

                if DOUBLE_OPERATOR_MAP.contains_key(double_symbol.as_str()) {
                    token.push(Token::new(TokenType::Operator(double_symbol), line));
                    chars.next();
                } else {
                    token.push(Token::new(TokenType::Operator(character.to_string()), line));
                }
            } else {
                token.push(Token::new(TokenType::Operator(character.to_string()), line));
            }
        } else if character == '#' && start_of_line {
            let mut pp_line_num = String::new();

            while let Some(c) = chars.next() {
                if c.is_numeric() {
                    pp_line_num.push(c);

                    if !chars.peek().map_or(false, |c| c.is_ascii_digit()) {
                        line = pp_line_num.parse::<u16>().unwrap_or(0);
                    }
                } else if c == '"' {
                    let mut _pp_filename = String::new();

                    while let Some(fc) = chars.next() {
                        if fc == '"' {
                            break;
                        }

                        _pp_filename.push(fc);
                    }
                } else if c == '\n' || c == '\r' {
                    break;
                }
            }

            while let Some(&c) = chars.peek() {
                if c != '\n' && c != '\r' {
                    break;
                }

                chars.next();
            }

            continue;
        } else {
            return Err(LexerError::UnknownCharacter { ch: character, line });
        }

        start_of_line = false;
    }

    let eof_line = if line > 1 && start_of_line { line - 1 } else { line };
    token.push(Token::new(TokenType::EOF, eof_line));

    token.reverse();
    
    Ok(token)
}


fn is_octal(c: char) -> bool {
    c >= '0' && c <= '7'
}


fn process_escape_sequence(chars: &mut Peekable<Chars<'_>>, line: u16) -> LexerResult<String> {
    let Some(escape_char) = chars.next() else {
        return Err(LexerError::UnterminatedCharLiteral { line });
    };

    let mut string_lit = String::new();

    match escape_char {
        'a' | 'b' | 'f' | 'n' | 'r' | 't' | 'v' | '\\' | '\'' | '"' => {
            let escaped_char = match escape_char {
                'a' => '\x07',
                'b' => '\x08',
                'f' => '\x0C',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                'v' => '\x0B',
                '\\' => '\\',
                '\'' => '\'',
                '"'  => '"',
                _ => unreachable!(),
            };

            string_lit.push(escaped_char);
            Ok(string_lit)
        }
        
        'x' => {
            let mut hex_digits = String::new();

            for _ in 0..2 {
                let Some(&c) = chars.peek() else {
                    break;
                };

                if !c.is_ascii_hexdigit() {
                    break;
                }

                hex_digits.push(c);
                chars.next();
            }

            if hex_digits.is_empty() {
                return Err(LexerError::ExpectedHexDigitsAfterX { line });
            }

            let value = u8::from_str_radix(&hex_digits, 16)
                .map_err(|_| LexerError::InvalidHexEscape { value: hex_digits.clone(), line })?;
            
            string_lit.push(value as char);
            Ok(string_lit)
        }

        'u' => {
            let mut unicode_digits = String::new();

            for _ in 0..4 {
                let Some(&c) = chars.peek() else {
                    return Err(LexerError::UnicodeEscapeIncomplete { found: unicode_digits.len(), line });
                };

                if c.is_ascii_hexdigit() {
                    unicode_digits.push(c);
                    chars.next();
                } else if c == '\'' {
                    return Err(LexerError::UnicodeEscapeIncomplete { found: unicode_digits.len(), line });
                } else {
                    return Err(LexerError::InvalidCharInUnicodeEscape { ch: c, line });
                }
            }

            if unicode_digits.is_empty() {
                return Err(LexerError::EmptyUnicodeEscape { line });
            }

            let codepoint = u32::from_str_radix(&unicode_digits, 16)
                .map_err(|_| LexerError::InvalidUnicodeEscape { value: unicode_digits.clone(), line })?;

            let Some(ch) = char::from_u32(codepoint) else {
                return Err(LexerError::InvalidUnicodeCodepoint { value: unicode_digits, line });
            };

            string_lit.push(ch);
            Ok(string_lit)
        }

        '0'..='7' => {
            let oct_digits = format!("{}{}", escape_char, process_octal(chars));
            let oct_value = u8::from_str_radix(&oct_digits, 8)
                .map_err(|_| LexerError::InvalidOctalEscape { value: oct_digits.clone(), line })?;
            
            string_lit.push(oct_value as char);
            Ok(string_lit)
        }
        
        _ => Err(LexerError::InvalidEscapeSequence { sequence: escape_char.to_string(), line }),
    }
}


fn process_octal(chars: &mut Peekable<Chars<'_>>) -> String {
    let mut oct_digits = String::new();

    for _ in 0..2 {
        let Some(&c) = chars.peek() else {
            break;
        };

        if !is_octal(c) {
            break;
        }

        chars.next();
        oct_digits.push(c);
    }

    oct_digits
}