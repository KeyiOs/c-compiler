use std::str::FromStr;
use std::fmt;
use std::collections::HashMap;
use std::sync::LazyLock;

use serde::{Serialize, Serializer};


#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenType {
    Keyword(String),
    Operator(String),
    Literal(String, Type),
    Identifier(String),
    EOF,
}

impl TokenType {
    #[inline]
    pub fn value(&self) -> &str {
        match self {
            TokenType::Keyword(s)
            | TokenType::Operator(s)
            | TokenType::Literal(s, _)
            | TokenType::Identifier(s) => s.as_str(),
            | TokenType::EOF => "EOF",
        }
    }
}


#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Type {
    Array(Box<Type>, Option<usize>),
    Bool,
    Char,
    Double,
    Enum(String),
    Float,
    Identifier,
    Int,
    Long,
    LongDouble,
    LongFloat,
    LongLong,
    Pointer(Box<Type>),
    Short,
    Signed(Box<Type>),
    Struct(String),
    Unsigned(Box<Type>),
    Void,
    None,
}

impl FromStr for Type {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "void" => Ok(Type::Void),
            "bool" => Ok(Type::Bool),
            "char" => Ok(Type::Char),
            "int" => Ok(Type::Int),
            "float" => Ok(Type::Float),
            "double" => Ok(Type::Double),
            "long" => Ok(Type::Long),
            "short" => Ok(Type::Short),
            "unsigned" => Ok(Type::Unsigned(Box::new(Type::Int))),
            "signed" => Ok(Type::Signed(Box::new(Type::Int))),
            _ => Err(()),
        }
    }
    
}

impl Serialize for Type {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Type::Array(inner, size) => {
                use serde::ser::SerializeMap;
                let mut map = serializer.serialize_map(Some(1))?;
                #[derive(Serialize)]
                struct ArrayFields<'a> {
                    #[serde(rename = "Type")]
                    ty: &'a Type,
                    size: &'a Option<usize>,
                }
                let fields = ArrayFields { ty: &**inner, size };
                map.serialize_entry("Array", &fields)?;
                map.end()
            },
            Type::Pointer(inner) => {
                use serde::ser::SerializeMap;
                let mut map = serializer.serialize_map(Some(1))?;
                #[derive(Serialize)]
                struct PointerFields<'a> {
                    #[serde(rename = "Type")]
                    ty: &'a Type,
                }
                let fields = PointerFields { ty: &**inner };
                map.serialize_entry("Pointer", &fields)?;
                map.end()
            },
            Type::Struct(name) => {
                use serde::ser::SerializeMap;
                let mut map = serializer.serialize_map(Some(1))?;
                #[derive(Serialize)]
                struct StructFields<'a> {
                    #[serde(rename = "Identifier")]
                    name: &'a String,
                }
                let fields = StructFields { name };
                map.serialize_entry("Struct", &fields)?;
                map.end()
            },
            _ => {
                serializer.serialize_str(&format!("{}", self))
            }
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Array(inner, size) => {
                match size {
                    Some(s) => write!(f, "Array({}, Size[{}])", inner, s),
                    None => write!(f, "Array({}, None)", inner),
                }
            }
            Type::Bool => write!(f, "Bool"),
            Type::Char => write!(f, "Char"),
            Type::Double => write!(f, "Double"),
            Type::Enum(name) => write!(f, "Enum({})", name),
            Type::Float => write!(f, "Float"),
            Type::Identifier => write!(f, "Identifier"),
            Type::Int => write!(f, "Int"),
            Type::Long => write!(f, "Long"),
            Type::LongDouble => write!(f, "LongDouble"),
            Type::LongFloat => write!(f, "LongFloat"),
            Type::LongLong => write!(f, "LongLong"),
            Type::None => write!(f, "None"),
            Type::Pointer(inner) => write!(f, "Pointer({})", inner),
            Type::Short => write!(f, "Short"),
            Type::Signed(inner) => write!(f, "Signed({})", inner),
            Type::Struct(name) => write!(f, "Struct({})", name),
            Type::Unsigned(inner) => write!(f, "Unsigned({})", inner),
            Type::Void => write!(f, "Void"),
        }
    }
}


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Keyword {
    Bool,
    Break,
    Case,
    Char,
    Continue,
    Default,
    Double,
    Else,
    Enum,
    False,
    Float,
    For,
    If,
    Int,
    Long,
    Printf,
    Return,
    Short,
    Signed,
    Struct,
    Switch,
    True,
    Unsigned,
    Void,
    While,
}

pub static KEYWORD_MAP: LazyLock<HashMap<&'static str, Keyword>> = LazyLock::new(|| {
    HashMap::from([
        ("bool", Keyword::Bool),
        ("break", Keyword::Break),
        ("case", Keyword::Case),
        ("char", Keyword::Char),
        ("continue", Keyword::Continue),
        ("default", Keyword::Default),
        ("double", Keyword::Double),
        ("else", Keyword::Else),
        ("enum", Keyword::Enum),
        ("false", Keyword::False),
        ("float", Keyword::Float),
        ("for", Keyword::For),
        ("if", Keyword::If),
        ("int", Keyword::Int),
        ("long", Keyword::Long),
        ("printf", Keyword::Printf),
        ("return", Keyword::Return),
        ("short", Keyword::Short),
        ("signed", Keyword::Signed),
        ("struct", Keyword::Struct),
        ("switch", Keyword::Switch),
        ("true", Keyword::True),
        ("unsigned", Keyword::Unsigned),
        ("void", Keyword::Void),
        ("while", Keyword::While),
    ])
});


#[derive(Debug, PartialEq, Eq, Hash)]
pub enum SingleOperator {
    Ampersand,
    Asterisk,
    Backslash,
    Caret,
    Colon,
    Comma,
    Dot,
    DoubleQuote,
    Equal,
    Exclamation,
    GreaterThan,
    LessThan,
    Minus,
    ParenthesisLeft,
    ParenthesisRight,
    Percent,
    Pipe,
    Plus,
    Question,
    Semicolon,
    SingleQuote,
    Slash,
    SquareBracketLeft,
    SquareBracketRight,
    Tilde,
    CurlyBracketLeft,
    CurlyBracketRight,
}

pub static SINGLE_OPERATOR_MAP: LazyLock<HashMap<char, SingleOperator>> = LazyLock::new(|| {
    HashMap::from([
        ('&', SingleOperator::Ampersand),
        ('*', SingleOperator::Asterisk),
        ('\\', SingleOperator::Backslash),
        ('^', SingleOperator::Caret),
        (':', SingleOperator::Colon),
        (',', SingleOperator::Comma),
        ('.', SingleOperator::Dot),
        ('"', SingleOperator::DoubleQuote),
        ('=', SingleOperator::Equal),
        ('!', SingleOperator::Exclamation),
        ('>', SingleOperator::GreaterThan),
        ('<', SingleOperator::LessThan),
        ('-', SingleOperator::Minus),
        ('(', SingleOperator::ParenthesisLeft),
        (')', SingleOperator::ParenthesisRight),
        ('%', SingleOperator::Percent),
        ('|', SingleOperator::Pipe),
        ('+', SingleOperator::Plus),
        ('?', SingleOperator::Question),
        (';', SingleOperator::Semicolon),
        ('\'', SingleOperator::SingleQuote),
        ('/', SingleOperator::Slash),
        ('[', SingleOperator::SquareBracketLeft),
        (']', SingleOperator::SquareBracketRight),
        ('~', SingleOperator::Tilde),
        ('{', SingleOperator::CurlyBracketLeft),
        ('}', SingleOperator::CurlyBracketRight),
    ])
});


#[derive(Debug, PartialEq, Eq, Hash)]
pub enum DoubleOperator {
    DoubleAmpersand,
    DoubleMinus,
    DoublePipe,
    DoublePlus,
    DoubleGreaterThan,
    DoubleLessThan,
    LessThanEqual,
    GreaterThanEqual,
    DoubleEqual,
    ExclamationEqual,
    PlusEqual,
    MinusEqual,
    AsteriskEqual,
    SlashEqual,
    PercentEqual,
    AmpersandEqual,
    CaretEqual,
    PipeEqual,
    Arrow,
}

pub static DOUBLE_OPERATOR_MAP: LazyLock<HashMap<&str, DoubleOperator>> = LazyLock::new(|| {
    HashMap::from([
        ("&&", DoubleOperator::DoubleAmpersand),
        ("--", DoubleOperator::DoubleMinus),
        ("||", DoubleOperator::DoublePipe),
        ("++", DoubleOperator::DoublePlus),
        (">>", DoubleOperator::DoubleGreaterThan),
        ("<<", DoubleOperator::DoubleLessThan),
        ("<=", DoubleOperator::LessThanEqual),
        (">=", DoubleOperator::GreaterThanEqual),
        ("==", DoubleOperator::DoubleEqual),
        ("!=", DoubleOperator::ExclamationEqual),
        ("+=", DoubleOperator::PlusEqual),
        ("-=", DoubleOperator::MinusEqual),
        ("*=", DoubleOperator::AsteriskEqual),
        ("/=", DoubleOperator::SlashEqual),
        ("%=", DoubleOperator::PercentEqual),
        ("&=", DoubleOperator::AmpersandEqual),
        ("^=", DoubleOperator::CaretEqual),
        ("|=", DoubleOperator::PipeEqual),
        ("->", DoubleOperator::Arrow),
    ])
});


#[derive(Debug, PartialEq, Eq, Hash)]
pub enum TripleOperator {
    LeftShiftEqual,
    RightShiftEqual,
}


pub static TRIPLE_OPERATOR_MAP: LazyLock<HashMap<&str, TripleOperator>> = LazyLock::new(|| {
    HashMap::from([
        ("<<=", TripleOperator::LeftShiftEqual),
        (">>=", TripleOperator::RightShiftEqual),
    ])
});


#[derive(Debug, Clone, Serialize)]
pub enum AstNode {
    ArrayAccess {
        array: Box<AstNode>,
        index: Box<AstNode>,
    },

    ArrayInitializer {
        items: Vec<AstNode>,
    },

    BinaryOperation {
        left: Box<AstNode>,
        operator: String,
        right: Box<AstNode>,
    },

    Break,

    Case {
        identifier: Box<AstNode>,
        body: Vec<AstNode>,
    },

    Continue,

    Dereference {
        operand: Box<AstNode>,
    },

    DesignatedInitializer {
        members: Vec<(String, AstNode)>,
    },

    ElseStatement {
        if_statement: Option<Box<AstNode>>,
        body: Option<Vec<AstNode>>,
    },

    Enum {
        identifier: String,
        variants: Vec<(String, Option<String>)>,
    },

    FnDeclaration {
        return_type: Type,
        identifier: String,
        parameters: Vec<(Type, String)>,
    },

    FnDefinition {
        return_type: Type,
        identifier: String,
        parameters: Vec<(Type, String)>,
        body: Vec<AstNode>,
    },

    ForStatement {
        declarations: Option<Vec<AstNode>>,
        condition: Option<Box<AstNode>>,
        increments: Option<Vec<AstNode>>,
        body: Vec<AstNode>,
    },

    FunctionCall {
        identifier: String,
        arguments: Vec<AstNode>,
    },

    IfStatement {
        condition: Box<AstNode>,
        body: Vec<AstNode>,
        else_branch: Option<Box<AstNode>>,
    },

    Literal {
        value: String,
        data_type: Type,
    },

    MemberAccess {
        object: Box<AstNode>,
        member: String,
        is_arrow: bool,
    },

    Printf {
        format_string: String,
        arguments: Vec<AstNode>,
    },

    Reference {
        operand: Box<AstNode>,
    },

    Return {
        expression: Option<Box<AstNode>>,
    },

    StructDeclaration {
        identifier: String,
        members: Vec<AstNode>,
    },

    StructDefinition {
        identifier: String,
        #[serde(serialize_with = "serialize_struct_vars")]
        variables: Vec<(AstNode, Vec<AstNode>)>,
    },

    StructCombined {
        identifier: String,
        members: Vec<AstNode>,
        variables: Vec<AstNode>,
    },

    Switch {
        identifier: Box<AstNode>,
        cases: Vec<AstNode>,
    },

    UnaryOperation {
        operand: Box<AstNode>,
        operator: String,
    },

    VarDeclaration {
        identifier: String,
        datatype: Type,
    },

    VarDefinition {
        identifier: String,
        datatype: Type,
        value: Box<AstNode>,
    },

    WhileStatement {
        condition: Box<AstNode>,
        body: Vec<AstNode>,
    },
}


fn serialize_struct_vars<S>(vars: &Vec<(AstNode, Vec<AstNode>)>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    use serde::ser::SerializeSeq;
    #[derive(Serialize)]
    struct StructVar<'a> {
        #[serde(rename = "Variable")]
        var: &'a AstNode,
        #[serde(rename = "Initializers")]
        initializers: &'a Vec<AstNode>,
    }

    let mut seq = serializer.serialize_seq(Some(vars.len()))?;
    for (var, inits) in vars {
        seq.serialize_element(&StructVar { var, initializers: inits })?;
    }
    seq.end()
}