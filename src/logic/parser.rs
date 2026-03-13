use std::{array, str::FromStr};

use crate::{Token, TokenType, data::{AstNode, definitions::Type}};
use crate::error::{ParseError, ParseResult};


pub struct Tokens {
    pub tokens: Vec<Token>,
}

impl Tokens {
    #[inline]
    pub fn next(&mut self) -> Token {
        self.tokens.pop().unwrap()
    }

    #[inline]
    pub fn peek(&self) -> &Token {
        self.tokens.last().unwrap()
    }

    #[inline]
    pub fn peek_n(&self, n: usize) -> Option<&Token> {
        if self.tokens.len() > n {
            self.tokens.get(self.tokens.len() - 1 - n)
        } else {
            None
        }
    }

    pub fn operator_match(&mut self, other: &str) -> ParseResult<()> {
        let var = self.next();

        if var.token_type.eq(&TokenType::Operator(other.to_string())) {
            Ok(())
        } else {
            Err(ParseError::ExpectedOperator { 
                expected: other.to_string(), 
                found: var.token_type.value().to_string(), 
                line: var.line 
            })
        }
    }

    pub fn type_match(&mut self, other: &TokenType) -> bool {
        let var = self.peek();

        if var.token_type.eq(other) {
            self.next();
            true
        } else {
            false
        }
    }
}


pub fn parser_start(input: &mut Tokens, depth: usize) -> ParseResult<Vec<AstNode>> {
    let mut ast = Vec::new();

    loop {
        let next_token = input.peek().clone();
        if next_token.token_type == TokenType::EOF {
            break;
        }

        match &next_token.token_type {
            TokenType::Keyword(k) => {
                let tmp = input.next();

                match Type::from_str(k.as_str()) {
                    Ok(_) => {
                        input.tokens.push(tmp);
                        ast.extend(process_declaration(input, depth)?);
                    },
                    Err(_) => {
                        match k.as_str() {
                            "if" => ast.push(process_if(input, depth)?),
                            "struct" => ast.push(process_struct(input)?),
                            "for" => ast.push(process_for(input, depth)?),
                            "while" => ast.push(process_while(input, depth)?),
                            "switch" => ast.push(process_switch(input, depth)?),
                            "enum" => {
                                let is_definition = matches!(input.peek().token_type, TokenType::Identifier(_))
                                    && input.peek_n(1).map_or(false, |tok| {
                                        tok.token_type.eq(&TokenType::Operator("{".to_string()))
                                    });

                                if is_definition {
                                    ast.push(process_enum(input)?);
                                } else {
                                    input.tokens.push(tmp);
                                    ast.extend(process_declaration(input, depth)?);
                                }
                            }
                            "printf" => ast.push(process_printf(input)?),
                            "break" => {
                                if depth == 0 {
                                    return Err(ParseError::BreakOutsideLoop(input.peek().line));
                                }

                                ast.push(AstNode::Break);
                                input.operator_match(";")?;
                                break;
                            }
                            "continue" => {
                                if depth == 0 {
                                    return Err(ParseError::ContinueOutsideLoop(input.peek().line));
                                }
                                
                                ast.push(AstNode::Continue);
                                input.operator_match(";")?;
                            }
                            "return" => {
                                if depth == 0 {
                                    return Err(ParseError::ReturnOutsideFunction(input.peek().line));
                                }
                                
                                if input.peek().token_type.eq(&TokenType::Operator(";".to_string())) {
                                    ast.push(AstNode::Return { expression: None });
                                    input.next();
                                } else {
                                    let expr = process_expression(input, 0, false)?;
                                    ast.push(AstNode::Return { expression: Some(Box::new(expr)) });
                                    input.operator_match(";")?;
                                }
                            }
                            _ => {
                                input.tokens.push(tmp);
                                break;
                            }
                        }
                    }
                }
            }
            TokenType::Literal(_, _) => {
                ast.push(process_expression(input, 0, false)?);
                input.operator_match(";")?;
            } TokenType::Identifier(_) => {
                ast.push(process_expression(input, 0, false)?);
                input.operator_match(";")?;
            } TokenType::Operator(s) => {
                match s.as_str() {
                    "}" => break,
                    _ => return Err(ParseError::UnexpectedOperator { operator: s.clone(), line: input.peek().line })
                }
            }
            _ => { input.next(); }
        }
    }

    Ok(ast)
}


fn precedence(operator: &str) -> u8 {
    match operator {
        "++" | "--" => 5,
        "*" | "/" | "%" => 4,
        "+" | "-" => 3,
        ">" | "<" | ">=" | "<=" | "==" | "!=" => 2,
        "+=" | "-=" | "*=" | "/=" | "%=" | "=" | "&&" | "||"=> 1,
        _ => 0,
    }
}


fn get_identifier(token: Token) -> ParseResult<String> {
    match token.token_type {
        TokenType::Identifier(ref s) => Ok(s.to_string()),
        _ => Err(ParseError::ExpectedIdentifier(token.line))
    }
}


fn parse_pointer_prefix(input: &mut Tokens, base_type: Type) -> Type {
    let mut current_type = base_type;
    while input.peek().token_type.eq(&TokenType::Operator("*".to_string())) {
        input.next();
        current_type = Type::Pointer(Box::new(current_type));
    }
    current_type
}


fn parse_array_suffix(input: &mut Tokens, mut base_type: Type) -> ParseResult<Type> {
    while input.peek().token_type.eq(&TokenType::Operator("[".to_string())) {
        input.next();
        let size = if input.peek().token_type.eq(&TokenType::Operator("]".to_string())) {
            None
        } else {
            let next = input.peek().clone();
            let token_type = next.token_type.clone();
            match token_type {
                TokenType::Literal(_, t) if matches!(t, Type::Int | Type::Long | Type::LongLong | Type::Short | Type::Char | Type::Unsigned(_) | Type::Signed(_)) => {
                    let val = input.next().token_type.value().to_string();
                    match val.parse::<usize>() {
                        Ok(n) => Some(n),
                        Err(_) => return Err(ParseError::UnexpectedToken {
                            expected: "valid array size (usize)".to_string(),
                            found: val,
                            line: next.line
                        })
                    }
                }
                TokenType::Identifier(_) => return Err(ParseError::UnexpectedToken {
                    expected: "constant integer array size".to_string(),
                    found: input.next().token_type.value().to_string(),
                    line: next.line
                }),
                _ => return Err(ParseError::UnexpectedToken {
                    expected: "number for array size".to_string(),
                    found: next.token_type.value().to_string(),
                    line: next.line
                })
            }
        };
        input.operator_match("]")?;
        base_type = Type::Array(Box::new(base_type), size);
    }

    Ok(base_type)
}


fn parse_comma_separated_expressions(input: &mut Tokens) -> ParseResult<Vec<AstNode>> {
    let mut items = Vec::new();
    while input.peek().token_type.ne(&TokenType::Operator(")".to_string())) {
        if input.peek().token_type.eq(&TokenType::Operator(",".to_string())) {
            input.next();
        }
        items.push(process_expression(input, 0, false)?);
    }
    Ok(items)
}


fn parse_array_initializer(input: &mut Tokens, var_type: &Type) -> ParseResult<AstNode> {
    input.operator_match("{")?;
    
    if input.peek().token_type.eq(&TokenType::Operator(".".to_string())) {
        let is_struct = matches!(var_type, Type::Struct(_));
        if !is_struct {
            return Err(ParseError::DesignatedInitializerOnNonStruct(input.peek().line));
        }
        
        let mut members: Vec<(String, AstNode)> = Vec::new();
        
        while input.peek().token_type.ne(&TokenType::Operator("}".to_string())) {
            if input.peek().token_type.eq(&TokenType::Operator(",".to_string())) {
                input.next();
                if input.peek().token_type.eq(&TokenType::Operator("}".to_string())) {
                    break;
                }
            }
            
            input.operator_match(".")?;
            let member_name = get_identifier(input.next())?;
            input.operator_match("=")?;
            let value = process_expression(input, 0, false)?;
            
            members.push((member_name, value));
        }
        
        input.operator_match("}")?;
        Ok(AstNode::DesignatedInitializer { members })
    } else {
        let mut items: Vec<AstNode> = Vec::new();

        while input.peek().token_type.ne(&TokenType::Operator("}".to_string())) {
            if input.peek().token_type.eq(&TokenType::Operator(",".to_string())) {
                input.next();
                if input.peek().token_type.eq(&TokenType::Operator("}".to_string())) {
                    break;
                }
            }
            
            if input.peek().token_type.eq(&TokenType::Operator("{".to_string())) {
                items.push(parse_array_initializer(input, var_type)?);
            } else {
                items.push(process_expression(input, 0, false)?);
            }
        }

        input.operator_match("}")?;
        Ok(AstNode::ArrayInitializer { items })
    }
}


fn process_body(input: &mut Tokens, depth: usize) -> ParseResult<Vec<AstNode>> {
    input.operator_match("{")?;
    let body = parser_start(input, depth + 1)?;
    input.operator_match("}")?;

    Ok(body)
}


fn process_expression(input: &mut Tokens, l_power: u8, mut comparison: bool) -> ParseResult<AstNode> {
    let line = input.peek().line;

    let mut lhs = match input.next().token_type {
        TokenType::Literal(s, t) => AstNode::Literal { value: s, data_type: t },
        TokenType::Identifier(s) => {
            if input.peek().token_type == TokenType::Operator("(".to_string()) {
                process_function_call(input, s)?
            } else if input.peek().token_type == TokenType::Operator("[".to_string()) {
                process_array_access(input, s)?
            } else if input.peek().token_type == TokenType::Operator(".".to_string()) {
                process_member_access(input, s, false)?
            } else if input.peek().token_type == TokenType::Operator("->".to_string()) {
                process_member_access(input, s, true)?
            } else {
                AstNode::Literal { value: s, data_type: Type::Identifier }
            }
        }
        TokenType::Operator(ref s) if s == "(" => { 
            let lhs = process_expression(input, 0, comparison)?;
            if !input.next().token_type.eq(&TokenType::Operator(")".to_string())) {
                return Err(ParseError::ExpectedClosingParen(line));
            };
            lhs
        },
        TokenType::Operator(ref s) if s == "*" => {
            let operand = process_expression(input, 5, comparison)?;
            AstNode::Dereference { operand: Box::new(operand) }
        },
        TokenType::Operator(ref s) if s == "&" => {
            let operand = process_expression(input, 5, comparison)?;
            AstNode::Reference { operand: Box::new(operand) }
        },
        TokenType::Keyword(ref s) if s == "true" || s == "false" => AstNode::Literal { value: s.clone(), data_type: Type::Bool },
        _ => return Err(ParseError::UnexpectedToken { 
            expected: "expression".to_string(), 
            found: input.peek().token_type.value().to_string(), 
            line: input.peek().line 
        })
    };

    loop {
        let operator = match &input.peek().token_type {
            TokenType::EOF => break,
            TokenType::Operator(s) if s == ")" || s == ";" => break,
            TokenType::Operator(s) => s.clone(),
            _ => return Err(ParseError::UnexpectedToken { 
                expected: "operator".to_string(), 
                found: input.peek().token_type.value().to_string(), 
                line: input.peek().line 
            })
        };

        if operator == "++" || operator == "--" {
            input.next();
            return Ok(AstNode::UnaryOperation{ operand: Box::new(lhs), operator });
        } else if operator == "." {
            input.next();
            let member = get_identifier(input.next())?;
            lhs = AstNode::MemberAccess {
                object: Box::new(lhs),
                member,
                is_arrow: false,
            };
            continue;
        } else if operator == "->" {
            input.next();
            let member = get_identifier(input.next())?;
            lhs = AstNode::MemberAccess {
                object: Box::new(lhs),
                member,
                is_arrow: true,
            };
            continue;
        } else if operator == "&&" || operator == "||" {
            comparison = false;
        }

        let r_power = precedence(&operator);
        if r_power == 0 || l_power >= r_power {
            break;
        } else if r_power == 2 {
            if comparison {
                return Err(ParseError::ChainedComparison(input.peek().line));
            }

            comparison = true;
        }

        input.next();
        let rhs = process_expression(input, r_power, comparison)?;

        lhs = AstNode::BinaryOperation {
            left: Box::new(lhs),
            operator: operator,
            right: Box::new(rhs),
        }
    }

    Ok(lhs)
}


fn process_array_access(input: &mut Tokens, identifier: String) -> ParseResult<AstNode> {
    let mut array = AstNode::Literal { value: identifier, data_type: Type::Identifier };
    
    while input.peek().token_type.eq(&TokenType::Operator("[".to_string())) {
        input.next();
        let index = process_expression(input, 0, false)?;
        input.operator_match("]")?;
        
        array = AstNode::ArrayAccess {
            array: Box::new(array),
            index: Box::new(index),
        };
    }
    
    Ok(array)
}


fn process_member_access(input: &mut Tokens, identifier: String, is_arrow: bool) -> ParseResult<AstNode> {
    let mut object = AstNode::Literal { value: identifier, data_type: Type::Identifier };
    
    while input.peek().token_type.eq(&TokenType::Operator(if is_arrow { "->" } else { "." }.to_string())) {
        input.next();
        let member = get_identifier(input.next())?;
        
        object = AstNode::MemberAccess {
            object: Box::new(object),
            member,
            is_arrow,
        };
    }
    
    Ok(object)
}


fn process_function_call(input: &mut Tokens, identifier: String) -> ParseResult<AstNode> {
    input.operator_match("(")?;
    let arguments = parse_comma_separated_expressions(input)?;
    input.operator_match(")")?;

    Ok(AstNode::FunctionCall {
        identifier,
        arguments,
    })
}


fn process_printf(input: &mut Tokens) -> ParseResult<AstNode> {
    input.operator_match("(")?;

    let format_string = match input.next().token_type {
        TokenType::Literal(s, ref t) if matches!(t, Type::Pointer(inner) if **inner == Type::Char) => s,
        _ => return Err(ParseError::ExpectedFormatString(input.peek().line))
    };

    let arguments = parse_comma_separated_expressions(input)?;
    input.operator_match(")")?;
    input.operator_match(";")?;

    Ok(AstNode::Printf {
        format_string,
        arguments,
    })
}


fn process_data_type(input: &mut Tokens) -> ParseResult<Type> {
    let mut var_type;
    let mut sign = Option::<String>::None;

    match input.peek().token_type {
        TokenType::Keyword(ref s) if s == "enum" => {
            input.next();
            let enum_name = get_identifier(input.next())?;
            return Ok(Type::Enum(enum_name));
        }
        TokenType::Keyword(ref s) if s == "unsigned" || s == "signed" => sign = Some(input.next().token_type.value().to_string()),
        _ => {}
    }
    
    let ttype = input.next();
    if ttype.token_type.eq(&TokenType::Keyword("long".to_string())) {
        if input.peek().token_type.eq(&TokenType::Keyword("long".to_string())) {
            var_type = Type::LongLong;
            input.next();
        } else {
            var_type = Type::Long;
        }
    } else {
        match Type::from_str(ttype.token_type.value()) {
            Ok(t) => var_type = t,
            Err(_) => {
                return Err(ParseError::UnexpectedToken {
                    expected: "data type".to_string(),
                    found: ttype.token_type.value().to_string(),
                    line: ttype.line,
                });
            }
        }
    }

    if sign.is_some() {
        match var_type {
            Type::Int | Type::Short | Type::Long | Type::LongLong | Type::Char => {
                match sign.unwrap().as_str() {
                    "unsigned" =>  var_type = Type::Unsigned(Box::new(var_type)),
                    "signed" => var_type = Type::Signed(Box::new(var_type)),
                    _ => {}
                }
            }
            _ => return Err(ParseError::InvalidSignedType(input.peek().line))
        }
    }

    Ok(var_type)
}


fn process_declaration(input: &mut Tokens, depth: usize) -> ParseResult<Vec<AstNode>> {
    let var_type = process_data_type(input)?;
    
    let mut i = input.tokens.len() -1;
    while input.tokens[i].token_type.eq(&TokenType::Operator("*".to_string())) {
        i -= 1;
    }

    if input.tokens[i - 1].token_type == TokenType::Operator("(".to_string()) {
        Ok(vec![process_fn_declaration(input, var_type, depth)?])
    } else {
        process_var_declaration(input, var_type)
    }
}


fn process_var_declaration(input: &mut Tokens, var_type: Type) -> ParseResult<Vec<AstNode>> {
    let mut declarations = Vec::new();

    loop {
        let mut current_type = parse_pointer_prefix(input, var_type.clone());
        let identifier = get_identifier(input.next())?;
        let mut array_size_unspecified = false;
         current_type = parse_array_suffix(input, current_type)?;
        if let Type::Array(ref base, ref size) = current_type {
            if size.is_none() && **base == Type::Char {
                array_size_unspecified = true;
            }
        }

        if input.peek().token_type.eq(&TokenType::Operator("=".to_string())) {
            input.next();

            let value: Box<AstNode>;
            if input.peek().token_type.eq(&TokenType::Operator("{".to_string())) {
                if !matches!(current_type, Type::Array(_, _)) {
                    return Err(ParseError::UnexpectedToken {
                        expected: "array type for initializer".to_string(),
                        found: "{".to_string(),
                        line: input.peek().line,
                    });
                }
                let base_type = match &current_type {
                    Type::Array(inner, _) => inner.as_ref().clone(),
                    _ => unreachable!()
                };
                value = Box::new(parse_array_initializer(input, &base_type)?);
            } else {
                let expr = process_expression(input, 0, false)?;
                if array_size_unspecified {
                    if let AstNode::Literal { value: str_val, data_type: Type::Pointer(ptr_base) } = &expr {
                        if **ptr_base == Type::Char {
                            let new_size = str_val.len() + 1;
                            current_type = Type::Array(Box::new(Type::Char), Some(new_size));
                        }
                    }
                }
                value = Box::new(expr);
            }

            declarations.push(AstNode::VarDefinition {
                identifier,
                datatype: current_type,
                value,
            });
        } else {
            declarations.push(AstNode::VarDeclaration {
                identifier,
                datatype: current_type,
            });
        };

        let token = input.next();
        match token.token_type {
            TokenType::Operator(ref s) if s == "," => {},
            TokenType::Operator(ref s) if s == ";" => break,
            _ => return Err(ParseError::ExpectedCommaOrSemicolon(token.line))
        };
    }

    Ok(declarations)
}


fn process_fn_declaration(input: &mut Tokens, return_type: Type, depth: usize) -> ParseResult<AstNode> {
    let return_type = parse_pointer_prefix(input, return_type);
    let identifier_token = input.next();
    let identifier = get_identifier(identifier_token.clone())?;
    
    if depth > 0 {
        return Err(ParseError::NestedFunctionDeclaration { 
            identifier: identifier.clone(), 
            line: identifier_token.line 
        });
    }

    input.operator_match("(")?;

    let mut parameters: Vec<(Type, String)> = Vec::new();
    while input.peek().token_type.ne(&TokenType::Operator(")".to_string())) {
        let base_type = process_data_type(input)?;
        let mut param_type = parse_pointer_prefix(input, base_type);

        if param_type == Type::Void {
            if !parameters.is_empty() || !input.peek().token_type.eq(&TokenType::Operator(")".to_string())) {
                return Err(ParseError::VoidParameterError(input.peek().line));
            }
            
            parameters.push((param_type, String::new()));
            break;
        }

        let param_identifier = get_identifier(input.next())?;
        param_type = parse_array_suffix(input, param_type)?;

        if input.peek().token_type.ne(&TokenType::Operator(")".to_string())) {
            input.operator_match(",")?;
        }

        parameters.push((param_type, param_identifier));
    }

    input.next();

    let tok = input.peek();
    match tok.token_type {
        TokenType::Operator(ref s) if s == "{" => {
            Ok(AstNode::FnDefinition {
                return_type,
                identifier,
                parameters,
                body: process_body(input, depth + 1)?,
            })
        }
        TokenType::Operator(ref s) if s == ";" => {
            input.next();

            Ok(AstNode::FnDeclaration {
                return_type,
                identifier,
                parameters,
            })
        }
        _ => Err(ParseError::ExpectedBraceOrSemicolon { 
            found: tok.token_type.value().to_string(), 
            line: tok.line 
        })
    }
}


fn process_if(input: &mut Tokens, depth: usize) -> ParseResult<AstNode> {
    input.operator_match("(")?;

    let condition = Box::new(process_expression(input, 0, false)?);

    input.operator_match(")")?;

    Ok(AstNode::IfStatement {
        condition,
        body: process_body(input, depth)?,
        else_branch: process_else(input, depth)?,
    })
}


fn process_else(input: &mut Tokens, depth: usize) -> ParseResult<Option<Box<AstNode>>> {
    if input.type_match(&TokenType::Keyword("else".to_string())) {
        if input.type_match(&TokenType::Keyword("if".to_string())) {
            Ok(Some(Box::new(process_if(input, depth)?)))
        } else {
            let else_body = process_body(input, depth)?;

            Ok(Some(Box::new(AstNode::ElseStatement {
                if_statement: None,
                body: Some(else_body),
            })))
        }
    } else {
        Ok(None)
    }
}


fn process_for(input: &mut Tokens, depth: usize) -> ParseResult<AstNode> {
    let mut declarations: Option<Vec<AstNode>>;
    let condition: Option<Box<AstNode>>;
    let mut increments: Option<Vec<AstNode>>;

    input.operator_match("(")?;

    if input.peek().token_type.eq(&TokenType::Operator(";".to_string())) {
        declarations = None;
        input.next();
    } else {
        if !matches!(input.peek().token_type, TokenType::Identifier(_)) {
            let var_type = process_data_type(input)?;
            declarations = Some(process_var_declaration(input, var_type)?);
        } else {
            declarations = Some(Vec::new());
            loop {
                if let Some(ref mut vec) = declarations {
                    vec.push(process_expression(input, 0, false)?);
                }

                match input.peek().token_type {
                    TokenType::Operator(ref s) if s == "," => { input.next(); }
                    TokenType::Operator(ref s) if s == ";" => {
                        input.next();
                        break;
                    }
                    _ => return Err(ParseError::ExpectedCommaOrSemicolonInFor { 
                        found: input.peek().token_type.value().to_string(), 
                        line: input.peek().line 
                    })
                }
            }
        }
    }

    if input.peek().token_type.eq(&TokenType::Operator(";".to_string())) {
        condition = None;
        input.next();
    } else {
        condition = Some(Box::new(process_expression(input, 0, false)?));
        input.operator_match(";")?;
    }

    if input.peek().token_type.ne(&TokenType::Operator(")".to_string())) {
        increments = Some(Vec::new());

        while input.peek().token_type.ne(&TokenType::Operator(")".to_string())) {
            if let Some(ref mut vec) = increments {
                vec.push(process_expression(input, 0, false)?);
            }

            let token = input.peek();
            match token.token_type {
                TokenType::Operator(ref s) if s == "," => { input.next(); }
                TokenType::Operator(ref s) if s == ")" => {}
                _ => return Err(ParseError::ExpectedCommaOrClosingParen { 
                    found: token.token_type.value().to_string(), 
                    line: token.line 
                })
            }
        }
    } else {
        increments = None;
    }

    input.next();

    Ok(AstNode::ForStatement {
        declarations,
        condition,
        increments,
        body: process_body(input, depth + 1)?,
    })
}


fn process_while(input: &mut Tokens, depth: usize) -> ParseResult<AstNode> {
    let var = input.peek();

    if var.token_type.eq(&TokenType::Operator("(".to_string())) {
        Ok(AstNode::WhileStatement {
            condition: Box::new(process_expression(input, 0, false)?),
            body: process_body(input, depth)?,
        })
    } else {
        Err(ParseError::ExpectedOperator { 
            expected: "(".to_string(), 
            found: var.token_type.value().to_string(), 
            line: var.line 
        })
    }
}


fn process_switch(input: &mut Tokens, depth: usize) -> ParseResult<AstNode> {
    input.operator_match("(")?;

    let identifier = Box::new(process_expression(input, 0, false)?);
    
    input.operator_match(")")?;
    input.operator_match("{")?;

    let mut cases = Vec::new();
    let mut default = false;
    loop {
        if default {
            return Err(ParseError::DefaultCaseNotLast(input.peek().line));
        }

        let case_identifier: Box<AstNode>;
        if input.type_match(&TokenType::Keyword("default".to_string())) {
            case_identifier = Box::new(AstNode::Literal {
                value: "default".to_string(),
                data_type: Type::Identifier,
            });
            default = true;
        } else if !input.type_match(&TokenType::Keyword("case".to_string())) {
            let found = input.peek();
            return Err(ParseError::ExpectedCase { 
                found: found.token_type.value().to_string(), 
                line: found.line 
            });
        } else {
            let expr = process_expression(input, 0, false)?;
            
            match &expr {
                AstNode::Literal { data_type, .. } => {
                    if !matches!(data_type, 
                        Type::Int | Type::Long | Type::LongLong | Type::Short | Type::Char | 
                        Type::Bool | Type::Unsigned(_) | Type::Signed(_)) {
                        return Err(ParseError::InvalidCaseLabel(input.peek().line));
                    }
                }
                _ => {
                    return Err(ParseError::InvalidCaseLabel(input.peek().line));
                }
            }
            
            case_identifier = Box::new(expr);
        }

        input.operator_match(":")?;

        let body: Vec<AstNode> = parser_start(input, depth + 1)?;
        if body.is_empty() {
            return Err(ParseError::EmptySwitch(input.peek().line));
        }

        cases.push(AstNode::Case {
            identifier: case_identifier,
            body,
        });

        if input.peek().token_type.eq(&TokenType::Operator("}".to_string())) {
            input.next();
            break;
        }
    }

    Ok(AstNode::Switch {
        identifier,
        cases,
    })
}


fn parse_struct_members(input: &mut Tokens, member_type: Type) -> ParseResult<Vec<AstNode>> {
    let mut members = Vec::new();

    loop {
        let mut current_type = parse_pointer_prefix(input, member_type.clone());
        let identifier = get_identifier(input.next())?;
        current_type = parse_array_suffix(input, current_type)?;

        if input.peek().token_type.eq(&TokenType::Operator("=".to_string())) {
            return Err(ParseError::StructMemberInitializer(input.peek().line));
        }

        members.push(AstNode::VarDeclaration {
            identifier,
            datatype: current_type,
        });

        let token = input.next();
        match token.token_type {
            TokenType::Operator(ref s) if s == "," => {},
            TokenType::Operator(ref s) if s == ";" => break,
            _ => return Err(ParseError::ExpectedCommaOrSemicolon(token.line))
        };
    }

    Ok(members)
}


fn parse_struct_variables(input: &mut Tokens, struct_name: String, loop_flag: bool) -> ParseResult<Vec<AstNode>> {
    let mut variables: Vec<AstNode> = Vec::new();
    
    if loop_flag {
        loop {
            let base_type = Type::Struct(struct_name.clone());
            let mut datatype = parse_pointer_prefix(input, base_type);
            let var_identifier = get_identifier(input.next())?;
            datatype = parse_array_suffix(input, datatype)?;

            variables.push(AstNode::VarDeclaration {
                identifier: var_identifier,
                datatype,
            });

            match input.peek().token_type {
                TokenType::Operator(ref s) if s == ";" => { input.next(); break; }
                TokenType::Operator(ref s) if s == "," => { input.next(); }
                _ => return Err(ParseError::ExpectedIdentifierOrSemicolon(input.peek().line))
            }
        }
    } else {
        let base_type = Type::Struct(struct_name.clone());
        let mut datatype = parse_pointer_prefix(input, base_type);
        let var_identifier = get_identifier(input.next())?;
        datatype = parse_array_suffix(input, datatype)?;

        variables.push(AstNode::VarDeclaration {
            identifier: var_identifier,
            datatype,
        });
    }
    
    Ok(variables)
}


fn process_struct(input: &mut Tokens) -> ParseResult<AstNode> {
    let struct_name = get_identifier(input.next())?;

    if !input.peek().token_type.eq(&TokenType::Operator("{".to_string())) {
        return process_struct_definition(input, struct_name.clone());
    }
    
    input.next();

    let mut members: Vec<AstNode> = Vec::new();
    while !input.peek().token_type.eq(&TokenType::Operator("}".to_string())) {
        let member_type = process_data_type(input)?;
        members.extend(parse_struct_members(input, member_type)?);
    }

    input.operator_match("}")?;

    let mut variables: Vec<AstNode> = Vec::new();
    match input.peek().token_type {
        TokenType::Operator(ref s) if s == ";" => {
            input.next();
        }
        TokenType::Operator(ref s) if s == "*" => {
            variables = parse_struct_variables(input, struct_name.clone(), true)?;
        }
        TokenType::Identifier(_) => {
            variables = parse_struct_variables(input, struct_name.clone(), true)?;
        }
        _ => return Err(ParseError::ExpectedIdentifierOrSemicolon(input.peek().line))
    }

    if variables.is_empty() {
        return Ok(AstNode::StructDeclaration {
            identifier: struct_name,
            members,
        });
    } else {
        Ok(AstNode::StructCombined {
            identifier: struct_name,
            members,
            variables,
        })
    }
}


fn process_struct_definition(input: &mut Tokens, struct_name: String) -> ParseResult<AstNode> {
    let mut variables: Vec<(AstNode, Vec<AstNode>)> = Vec::new();
    let mut temp_var: Vec<AstNode>;
    loop {
        match input.peek().token_type {
            TokenType::Operator(ref s) if s == ";" => {
                input.next();
                break;
            }
            TokenType::Operator(ref s) if s == "," => {
                input.next();
            }
            TokenType::Operator(ref s) if s == "*" => {
                temp_var = parse_struct_variables(input, struct_name.clone(), false)?;
                let mut temp_members = Vec::new();
                if let TokenType::Operator(ref s) = input.peek().token_type {
                    if s == "=" {
                        input.next();
                        input.operator_match("{")?;
                        while input.peek().token_type.ne(&TokenType::Operator("}".to_string())) {
                            if input.peek().token_type.eq(&TokenType::Operator(",".to_string())) {
                                input.next();
                            }
                            temp_members.push(process_expression(input, 0, false)?);
                        }
                        input.next();
                    }
                }
                variables.push((temp_var.pop().unwrap(), temp_members));
            }
            TokenType::Identifier(_) => {
                temp_var = parse_struct_variables(input, struct_name.clone(), false)?;
                let mut temp_members = Vec::new();
                if let TokenType::Operator(ref s) = input.peek().token_type {
                    if s == "=" {
                        input.next();
                        input.operator_match("{")?;

                        if input.peek().token_type.eq(&TokenType::Operator("{".to_string())) {
                            let mut array_members = Vec::new();
                            while input.peek().token_type.ne(&TokenType::Operator("}".to_string())) {
                                if input.peek().token_type.eq(&TokenType::Operator(",".to_string())) {
                                    input.next();
                                }
                                
                                input.operator_match("{")?;

                                while input.peek().token_type.ne(&TokenType::Operator("}".to_string())) {
                                    if input.peek().token_type.eq(&TokenType::Operator(",".to_string())) {
                                        input.next();
                                    }
                                    array_members.push(process_expression(input, 0, false)?);
                                }

                                input.next();
                            }

                            temp_members.push(AstNode::ArrayInitializer { items: array_members });

                            input.next();
                        } else {
                            while input.peek().token_type.ne(&TokenType::Operator("}".to_string())) {
                                if input.peek().token_type.eq(&TokenType::Operator(",".to_string())) {
                                    input.next();
                                }
                                temp_members.push(process_expression(input, 0, false)?);
                            }

                            input.next();
                        }
                    }
                }

                variables.push((temp_var.pop().unwrap(), temp_members));
            }
            _ => return Err(ParseError::ExpectedIdentifierOrSemicolon(input.peek().line))
        }
    }

    Ok(AstNode::StructDefinition {
        identifier: struct_name,
        variables,
    })
}

fn process_enum(input: &mut Tokens) -> ParseResult<AstNode> {
    let identifier = get_identifier(input.next())?;

    input.operator_match("{")?;

    let mut variants = Vec::new();
    loop {
        let token = input.next();
        if !matches!(token.token_type, TokenType::Identifier(_)) {
            return Err(ParseError::ExpectedIdentifier(token.line));
        }

        let variant_name = token.token_type.value().to_string();
        let variant_value = if input.peek().token_type == TokenType::Operator("=".to_string()) {
            input.next();
            match input.next().token_type {
                TokenType::Literal(s, _) => Some(s),
                TokenType::Operator(op) if op == "-" => {
                    match input.next().token_type {
                        TokenType::Literal(s, _) => Some(format!("-{}", s)),
                        _ => return Err(ParseError::ExpectedLiteral(input.peek().line))
                    }
                }
                _ => return Err(ParseError::ExpectedLiteral(input.peek().line))
            }
        } else {
            None
        };
        
        variants.push((variant_name, variant_value));
        if !input.peek().token_type.eq(&TokenType::Operator(",".to_string())) {
            input.operator_match("}")?;
            break;
        }

        input.next();
    }

    input.operator_match(";")?;

    if variants.is_empty() {
        return Err(ParseError::EmptyEnum(input.peek().line));
    }
    
    Ok(AstNode::Enum {
        identifier,
        variants,
    })
}


// Typedef ?
// Union ?
// Malloc ?
// Free ?