use std::fmt;

#[derive(Debug, Clone)]
pub enum LexerError {
    InvalidOctalNumber { value: String, line: u16 },
    InvalidHexNumber { value: String, line: u16 },
    EmptyHexNumber { line: u16 },
    MultipleDecimalPoints { line: u16 },
    InvalidExponent { line: u16 },
    InvalidNumberSuffix { line: u16 },
    UnterminatedCharLiteral { line: u16 },
    EmptyCharLiteral { line: u16 },
    CharLiteralTooLong { line: u16 },
    UnterminatedStringLiteral { line: u16 },
    InvalidEscapeSequence { sequence: String, line: u16 },
    InvalidHexEscape { value: String, line: u16 },
    InvalidUnicodeEscape { value: String, line: u16 },
    InvalidUnicodeCodepoint { value: String, line: u16 },
    InvalidOctalEscape { value: String, line: u16 },
    UnicodeEscapeIncomplete { found: usize, line: u16 },
    InvalidCharInUnicodeEscape { ch: char, line: u16 },
    EmptyUnicodeEscape { line: u16 },
    UnknownCharacter { ch: char, line: u16 },
    ExpectedHexDigitsAfterX { line: u16 },
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexerError::InvalidOctalNumber { value, line } => 
                write!(f, "Invalid octal number '{}' on line {}", value, line),
            LexerError::InvalidHexNumber { value, line } => 
                write!(f, "Invalid hex number '{}' on line {}", value, line),
            LexerError::EmptyHexNumber { line } => 
                write!(f, "Invalid hex number on line {}", line),
            LexerError::MultipleDecimalPoints { line } => 
                write!(f, "Multiple decimal points in number on line {}", line),
            LexerError::InvalidExponent { line } => 
                write!(f, "Invalid exponent in number on line {}", line),
            LexerError::InvalidNumberSuffix { line } => 
                write!(f, "Invalid number suffix on line {}", line),
            LexerError::UnterminatedCharLiteral { line } => 
                write!(f, "Unterminated character literal on line {}", line),
            LexerError::EmptyCharLiteral { line } => 
                write!(f, "Empty character literal on line {}", line),
            LexerError::CharLiteralTooLong { line } => 
                write!(f, "Character literal too long on line {}", line),
            LexerError::UnterminatedStringLiteral { line } => 
                write!(f, "Unterminated string literal on line {}", line),
            LexerError::InvalidEscapeSequence { sequence, line } => 
                write!(f, "Invalid escape sequence: \\{} on line {}", sequence, line),
            LexerError::InvalidHexEscape { value, line } => 
                write!(f, "Invalid hex escape: \\x{} on line {}", value, line),
            LexerError::InvalidUnicodeEscape { value, line } => 
                write!(f, "Invalid unicode escape: \\u{{{}}} on line {}", value, line),
            LexerError::InvalidUnicodeCodepoint { value, line } => 
                write!(f, "Invalid unicode codepoint: \\u{{{}}} on line {}", value, line),
            LexerError::InvalidOctalEscape { value, line } => 
                write!(f, "Invalid octal escape: \\{} on line {}", value, line),
            LexerError::UnicodeEscapeIncomplete { found, line } => 
                write!(f, "Invalid escape sequence. Requires 4 hex digits, but found {} on line {}", found, line),
            LexerError::InvalidCharInUnicodeEscape { ch, line } => 
                write!(f, "Invalid character '{}' in unicode escape on line {}", ch, line),
            LexerError::EmptyUnicodeEscape { line } => 
                write!(f, "Empty unicode escape on line {}", line),
            LexerError::UnknownCharacter { ch, line } => 
                write!(f, "Unknown character: '{}' on line {}", ch, line),
            LexerError::ExpectedHexDigitsAfterX { line } => 
                write!(f, "Expected hex digits after \\x on line {}", line),
        }
    }
}

impl std::error::Error for LexerError {}
pub type LexerResult<T> = Result<T, LexerError>;


#[derive(Debug, Clone)]
pub enum ParseError {
    BreakOutsideLoop(u16),
    ContinueOutsideLoop(u16),
    UnexpectedOperator { operator: String, line: u16 },
    ExpectedIdentifier(u16),
    ExpectedLiteral(u16),
    ExpectedCommaOrSemicolon(u16),
    ExpectedClosingParen(u16),
    UnexpectedToken { expected: String, found: String, line: u16 },
    ChainedComparison(u16),
    ExpectedFormatString(u16),
    InvalidSignedType(u16),
    VoidParameterError(u16),
    ExpectedBraceOrSemicolon { found: String, line: u16 },
    ExpectedCommaOrSemicolonInFor { found: String, line: u16 },
    ExpectedCommaOrClosingParen { found: String, line: u16 },
    DefaultCaseNotLast(u16),
    ExpectedCase { found: String, line: u16 },
    InvalidCaseLabel(u16),
    EmptySwitch(u16),
    ExpectedIdentifierOrSemicolon(u16),
    EmptyEnum(u16),
    ExpectedOperator { expected: String, found: String, line: u16 },
    NestedFunctionDeclaration { identifier: String, line: u16 },
    ReturnOutsideFunction(u16),
    StructMemberInitializer(u16),
    DesignatedInitializerOnNonStruct(u16),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::BreakOutsideLoop(line) => 
                write!(f, "'break' used outside of loop on line {}", line),
            ParseError::ContinueOutsideLoop(line) => 
                write!(f, "'continue' used outside of loop on line {}", line),
            ParseError::UnexpectedOperator { operator, line } => 
                write!(f, "Unexpected operator '{}' on line {}", operator, line),
            ParseError::ExpectedIdentifier(line) => 
                write!(f, "Expected identifier on line {}", line),
            ParseError::ExpectedLiteral(line) => 
                write!(f, "Expected literal on line {}", line),
            ParseError::ExpectedCommaOrSemicolon(line) => 
                write!(f, "Expected ',' or ';' on line {}", line),
            ParseError::ExpectedClosingParen(line) => 
                write!(f, "Expected ')' on line {}", line),
            ParseError::UnexpectedToken { expected, found, line } => 
                write!(f, "Expected {} on line {}, found {}", expected, line, found),
            ParseError::ChainedComparison(line) => 
                write!(f, "Chained comparisons are not allowed on line {}", line),
            ParseError::ExpectedFormatString(line) => 
                write!(f, "Expected format string literal in printf on line {}", line),
            ParseError::InvalidSignedType(line) => 
                write!(f, "Only integer types can be signed or unsigned on line {}", line),
            ParseError::VoidParameterError(line) => 
                write!(f, "Void type can only be used for single 'void' parameter on line {}", line),
            ParseError::ExpectedBraceOrSemicolon { found, line } => 
                write!(f, "Expected '{{' or ';' after function declaration on line {}, but found {}", line, found),
            ParseError::ExpectedCommaOrSemicolonInFor { found, line } => 
                write!(f, "Expected ',' or ';' after for loop declaration but found {} on line {}", found, line),
            ParseError::ExpectedCommaOrClosingParen { found, line } => 
                write!(f, "Expected ',' or ')' after increment but found {} on line {}", found, line),
            ParseError::DefaultCaseNotLast(line) => 
                write!(f, "Default case must be the last case in switch on line {}", line),
            ParseError::ExpectedCase { found, line } => 
                write!(f, "Expected 'case' in switch statement on line {} but found {}", line, found),
            ParseError::InvalidCaseLabel(line) => 
                write!(f, "Case label must be an integer constant expression on line {}", line),
            ParseError::EmptySwitch(line) => 
                write!(f, "Switch must have at least one case on line {}", line),
            ParseError::ExpectedIdentifierOrSemicolon(line) => 
                write!(f, "Expected identifier or ';' in struct variable list on line {}", line),
            ParseError::EmptyEnum(line) => 
                write!(f, "Enum must have at least one variant on line {}", line),
            ParseError::ExpectedOperator { expected, found, line } => 
                write!(f, "Expected '{}' on line {} but found '{}'", expected, line, found),
            ParseError::NestedFunctionDeclaration { identifier, line } => 
                write!(f, "Function '{}' cannot be declared inside another function on line {}", identifier, line),
            ParseError::ReturnOutsideFunction(line) => 
                write!(f, "'return' is not allowed at file scope on line {}", line),
            ParseError::StructMemberInitializer(line) =>
                write!(f, "Struct declaration members cannot have initializers (line {})", line),
            ParseError::DesignatedInitializerOnNonStruct(line) => 
                write!(f, "Designated initializers can only be used with struct types on line {}", line),
        }
    }
}

impl std::error::Error for ParseError {}
pub type ParseResult<T> = Result<T, ParseError>;


#[derive(Debug, Clone)]
pub enum SemanticError {
    DuplicateIdentifier(String),
    TypeMismatch(String),
    UndefinedVariable(String),
    UndefinedFunction(String),
    ArgumentCountMismatch {
        function: String,
        expected: usize,
        found: usize,
    },
    StructMemberCountMismatch {
        struct_name: String,
        expected: usize,
        found: usize,
    },
    InternalError(String),
}

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SemanticError::DuplicateIdentifier(name) => {
                write!(f, "Duplicate identifier: '{}'", name)
            }
            SemanticError::TypeMismatch(msg) => {
                write!(f, "Type mismatch: {}", msg)
            }
            SemanticError::UndefinedVariable(name) => {
                write!(f, "Undefined variable: '{}'", name)
            }
            SemanticError::UndefinedFunction(name) => {
                write!(f, "Undefined function: '{}'", name)
            }
            SemanticError::ArgumentCountMismatch { function, expected, found } => {
                write!(f, "Function '{}' expects {} argument(s), but found {}", function, expected, found)
            }
            SemanticError::StructMemberCountMismatch { struct_name, expected, found } => {
                write!(f, "Struct '{}' member count mismatch: expected {}, found {}", struct_name, expected, found)
            }
            SemanticError::InternalError(msg) => {
                write!(f, "Internal error: {}", msg)
            }
        }
    }
}

impl std::error::Error for SemanticError {}
pub type SemanticResult<T> = Result<T, SemanticError>;


#[derive(Debug, Clone)]
pub enum SymbolError {
    SymbolAlreadyExists { name: String },
    SymbolNotFound { name: String },
    NotAFunction { name: String },
    TypeMismatch { name: String, message: String },
}

impl fmt::Display for SymbolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SymbolError::SymbolAlreadyExists { name } =>
                write!(f, "Symbol '{}' already exists", name),
            SymbolError::SymbolNotFound { name } =>
                write!(f, "Symbol '{}' not found", name),
            SymbolError::NotAFunction { name } =>
                write!(f, "Symbol '{}' is not a function", name),
            SymbolError::TypeMismatch { name, message } =>
                write!(f, "Type mismatch for '{}': {}", name, message),
        }
    }
}

impl std::error::Error for SymbolError {}
pub type SymbolResult<T> = Result<T, SymbolError>;