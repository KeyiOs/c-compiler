use crate::data::{AstNode, SymbolTable};
use crate::data::symbol_table::Symbol;
use crate::data::definitions::Type;
use crate::error::{SemanticError, SemanticResult};
use std::collections::HashSet;
use std::sync::Mutex;


pub struct SemanticContext {
    pub fn_table: SymbolTable,
    pub var_table: SymbolTable,
    pub scope: Vec<String>,
}


// Usage examples for DT_PROMOTION:
// ● Writing to DT_PROMOTION:
// ● *DT_PROMOTION.lock().unwrap() = [Type::Int, Type::Float];
// 
// Reading from DT_PROMOTION:
// ● let current_types = DT_PROMOTION.lock().unwrap().clone();
// 
// Accessing specific element:
// ● let left_type = DT_PROMOTION.lock().unwrap()[0].clone();
// ● let right_type = DT_PROMOTION.lock().unwrap()[1].clone();
static DT_PROMOTION: Mutex<[Type; 2]> = Mutex::new([Type::None, Type::None]);


pub fn semantic_analyze(ast: &[AstNode], sem_ctx: &mut SemanticContext) -> SemanticResult<()> {
    for node in ast {
        analyze_node(node, sem_ctx)?;
    }

    Ok(())
}


fn analyze_node(node: &AstNode, sem_ctx: &mut SemanticContext) -> SemanticResult<()> {
    match node {
        AstNode::VarDeclaration { identifier, datatype } => {
            sem_ctx.var_table.insert_variable(
                identifier.clone(),
                datatype.clone(),
                false,
                sem_ctx.scope.last().unwrap().clone(),
            ).map_err(|_e| {
                SemanticError::DuplicateIdentifier(identifier.to_string())
            })?;
        }
        AstNode::VarDefinition { identifier, datatype, value } => {
            sem_ctx.var_table.insert_variable(
                identifier.clone(),
                datatype.clone(),
                true,
                sem_ctx.scope.last().unwrap().clone(),
            ).map_err(|_e| {
                SemanticError::DuplicateIdentifier(identifier.to_string())
            })?;

            let value_node = value.as_ref();
            if let (Type::Array(base_type, _), AstNode::ArrayInitializer { items }) = (datatype, value_node) {
                for item in items {
                    analyze_node(item, sem_ctx)?;
                    let item_type = DT_PROMOTION.lock().unwrap()[0].clone();

                    if !is_assignable(base_type, &item_type) {
                        return Err(SemanticError::TypeMismatch(
                            format!("Cannot assign {} to {}", item_type, base_type)
                        ));
                    }
                }

                return Ok(());
            }

            analyze_node(value_node, sem_ctx)?;
            let assigned_type = DT_PROMOTION.lock().unwrap()[0].clone();

            if let (Type::Array(base_type, Some(array_size)), AstNode::Literal { value, data_type }) = (datatype, value_node) {
                if **base_type == Type::Char && matches!(data_type, Type::Pointer(ptr_base) if **ptr_base == Type::Char) {
                    let str_len = value.len();
                    if str_len > *array_size {
                        return Err(SemanticError::TypeMismatch(
                            format!("String literal '{}' (len {}) does not fit in char array of size {}", value, str_len, array_size)
                        ));
                    }
                }
            }

            if !is_assignable(datatype, &assigned_type) {
                return Err(SemanticError::TypeMismatch(
                    format!("Cannot assign {} to {}", assigned_type, datatype)
                ));
            }
        }
        AstNode::FnDeclaration { return_type, identifier, parameters } => {
            sem_ctx.fn_table.insert_function(
                identifier.clone(),
                return_type.clone(),
                parameters.clone(),
                false,
                sem_ctx.scope.last().unwrap().clone(),
            ).map_err(|_e| {
                SemanticError::DuplicateIdentifier(identifier.to_string())
            })?;
        }
        AstNode::FnDefinition { return_type, identifier, parameters, body } => {
            sem_ctx.fn_table.insert_function(
                identifier.clone(),
                return_type.clone(),
                parameters.clone(),
                true,
                sem_ctx.scope.last().unwrap().clone(),
            ).map_err(|_e| {
                SemanticError::DuplicateIdentifier(identifier.to_string())
            })?;

            sem_ctx.scope.push(identifier.clone());

            for (param_type, param_name) in parameters {
                sem_ctx.var_table.insert_variable(
                    param_name.clone(),
                    param_type.clone(),
                    true,
                    identifier.clone(),
                ).map_err(|_e| SemanticError::DuplicateIdentifier(param_name.clone()))?;
            }

            let mut has_return = false;
            for child in body {
                if matches!(child, AstNode::Return { .. }) {
                    has_return = true;
                }
                analyze_node(child, sem_ctx)?;
            }

            if return_type != &Type::Void && !has_return {
                return Err(SemanticError::TypeMismatch(
                    format!("Function '{}' must return a value of type {}", identifier, return_type)
                ));
            }

            let vars_to_remove: Vec<String> = sem_ctx.var_table.get_all()
                .iter()
                .filter_map(|symbol| {
                    if let Symbol::Variable { name, scope, .. } = symbol {
                        if scope == identifier {
                            Some(name.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();
            
            for var_name in vars_to_remove {
                sem_ctx.var_table.remove(&var_name).ok();
            }

            sem_ctx.scope.pop();
        }
        AstNode::FunctionCall { identifier, arguments } => {
            let parameters = sem_ctx.fn_table.get_parameters(identifier).map_err(|_e| {
                SemanticError::UndefinedFunction(identifier.to_string())
            })?;

            if parameters.len() != arguments.len() {
                return Err(SemanticError::ArgumentCountMismatch {
                    function: identifier.to_string(),
                    expected: parameters.len(),
                    found: arguments.len(),
                });
            }

            for ((param_type, _), arg_node) in parameters.iter().zip(arguments.iter()) {
                if let Ok(mut dt_promo) = DT_PROMOTION.lock() {
                    dt_promo[0] = Type::None;
                    dt_promo[1] = Type::None;
                }

                analyze_node(arg_node, sem_ctx)?;
                let arg_type = DT_PROMOTION.lock().unwrap()[0].clone();

                if !is_assignable(param_type, &arg_type) {
                    return Err(SemanticError::TypeMismatch(
                        format!("Cannot pass argument of type {} to parameter of type {}", arg_type, param_type)
                    ));
                }
            }

            let return_type = sem_ctx.fn_table.get_type(identifier).map_err(|_e| {
                SemanticError::InternalError(format!("Missing function type for '{}'", identifier))
            })?;

            if let Ok(mut dt_promo) = DT_PROMOTION.lock() {
                dt_promo[0] = return_type;
                dt_promo[1] = Type::None;
            }
        }
        AstNode::Return { expression } => {
            let current_fn = sem_ctx.scope.last().unwrap();
            if let Ok(fn_return_type) = sem_ctx.fn_table.get_type(current_fn) {
                if let Some(expression) = expression {
                    analyze_node(expression, sem_ctx)?;

                    if fn_return_type == Type::Void {
                        return Err(SemanticError::TypeMismatch(
                            format!("Expected Void, got {}", DT_PROMOTION.lock().unwrap()[0].clone())
                        ));
                    }

                    let returned_type = DT_PROMOTION.lock().unwrap()[0].clone();
                    if !is_assignable(&fn_return_type, &returned_type) {
                        return Err(SemanticError::TypeMismatch(
                            format!("Cannot return {} from function expecting {}", returned_type, fn_return_type)
                        ));
                    }
                } else {
                    if fn_return_type != Type::Void {
                        return Err(SemanticError::TypeMismatch(
                            format!("Expected {}, got Void", fn_return_type)
                        ));
                    }
                }
            }
        }
        AstNode::BinaryOperation { left, operator, right } => {
            if let Ok(mut dt_promo) = DT_PROMOTION.lock() {
                dt_promo[0] = Type::None;
                dt_promo[1] = Type::None;
            }

            if operator == "=" {
                if !is_lvalue(left) {
                    return Err(SemanticError::TypeMismatch(
                        "Left side of assignment must be a variable".to_string()
                    ));
                }

                analyze_node(left, sem_ctx)?;
                let left_type = DT_PROMOTION.lock().unwrap()[0].clone();

                analyze_node(right, sem_ctx)?;
                let right_type = DT_PROMOTION.lock().unwrap()[0].clone();

                if !is_assignable(&left_type, &right_type) {
                    return Err(SemanticError::TypeMismatch(
                        format!(
                            "Cannot assign {} to {}",
                            right_type,
                            left_type
                        )
                    ));
                }
                    
                return Ok(());
            }

            analyze_node(left, sem_ctx)?;
            analyze_node(right, sem_ctx)?;

            let result_type = match get_operator_category(operator) {
                Some(cat) => match cat.as_str() {
                    "arithmetic" => get_arithmetic_promotion(),
                    "comparison" | "logical" => Type::Bool,
                    "assignment" | "unary" => DT_PROMOTION.lock().unwrap()[0].clone(),
                    _ => Type::Void,
                },
                None => return Err(SemanticError::InternalError("Unknown operator".to_string())),
            };

            if let Ok(mut dt_promo) = DT_PROMOTION.lock() {
                dt_promo[0] = result_type;
                dt_promo[1] = Type::None;
            }
        }
        AstNode::UnaryOperation { operator, operand } => {
            if let Ok(mut dt_promo) = DT_PROMOTION.lock() {
                dt_promo[0] = Type::None;
                dt_promo[1] = Type::None;
            }

            analyze_node(operand, sem_ctx)?;

            let operand_type = DT_PROMOTION.lock().unwrap()[0].clone();
            
            if !is_unary_incrementable_type(&operand_type) {
                return Err(SemanticError::TypeMismatch(
                    format!("Unary '{}' requires arithmetic or pointer type, got {}", operator, operand_type)
                ));
            }
        }
        AstNode::Literal { value, data_type } => {
            if let Ok(mut dt_promo) = DT_PROMOTION.lock() {
                dt_promo[0] = Type::None;
                dt_promo[1] = Type::None;
            }

            let resolved_type = if *data_type == Type::Identifier {
                sem_ctx.var_table.get_type(value).map_err(|_| {
                    SemanticError::UndefinedVariable(value.clone())
                })?
            } else {
                data_type.clone()
            };

            if let Ok(mut dt_promo) = DT_PROMOTION.lock() {
                if dt_promo[0] == Type::None {
                    dt_promo[0] = resolved_type;
                } else if dt_promo[1] == Type::None {
                    dt_promo[1] = resolved_type;
                } else {
                    return Err(SemanticError::InternalError("DT_PROMOTION array overflow".to_string()));
                }
            }
        }
        AstNode::Switch { identifier, cases } => {
            match identifier.as_ref() {
                AstNode::Literal { value, data_type } => {
                    if data_type == &Type::Identifier {
                        sem_ctx.var_table.is_initialized(value).map_err(|_e| {
                            SemanticError::UndefinedVariable(value.to_string())
                        })?;
                    } else if !is_integral_type(&data_type) {
                        return Err(SemanticError::TypeMismatch(
                            format!("Switch expression must be an integral type, got {}", data_type)
                        ));
                    }

                    sem_ctx.scope.push(format!("switch_{}", value.clone()));
                }
                _ => {
                    return Err(SemanticError::TypeMismatch(
                        "Switch expression must be a literal or identifier".to_string()
                    ));
                }
            };

            for case_node in cases {
                analyze_node(case_node, sem_ctx)?;
            }

            sem_ctx.scope.pop();
        }
        AstNode::StructDeclaration { identifier, members } => {
            sem_ctx.var_table.insert_struct(identifier.clone(), members.clone())
                .map_err(|_e| SemanticError::DuplicateIdentifier(identifier.clone()))?;
        }
        AstNode::StructCombined { identifier, members, variables } => {
            sem_ctx.var_table.insert_struct(identifier.clone(), members.clone())
                .map_err(|_e| SemanticError::DuplicateIdentifier(identifier.clone()))?;

            for variable in variables {
                if let AstNode::VarDeclaration { identifier: var_name, datatype } = variable {
                    sem_ctx.var_table.insert_variable(
                        var_name.clone(),
                        datatype.clone(),
                        false,
                        sem_ctx.scope.last().unwrap().clone(),
                    ).map_err(|_e| SemanticError::DuplicateIdentifier(var_name.clone()))?;
                }
            }
        }
        AstNode::StructDefinition { identifier, variables } => {
            if !sem_ctx.var_table.contains(identifier) {
                return Err(SemanticError::UndefinedVariable(identifier.clone()));
            }

            let struct_members = match sem_ctx.var_table.get_struct_members(identifier) {
                Ok(members) => members,
                Err(_) => {
                    return Err(SemanticError::InternalError(format!("Struct '{}' not found in symbol table", identifier)));
                }
            };

            for (variable, initializers) in variables {
                let count = struct_members.len().min(initializers.len());
                for i in 0..count {
                    let member_type = match &struct_members[i] {
                        AstNode::VarDeclaration { datatype, .. } => datatype,
                        AstNode::VarDefinition { datatype, .. } => datatype,
                        _ => {
                            println!("Struct member at index {} is not a declaration or definition", i);
                            continue;
                        }
                    };
                    let init_type = match &initializers[i] {
                        AstNode::VarDeclaration { datatype, .. } => datatype,
                        AstNode::VarDefinition { datatype, .. } => datatype,
                        AstNode::Literal { data_type, .. } => data_type,
                        _ => {
                            println!("Initializer at index {} is not a declaration, definition, or literal", i);
                            continue;
                        }
                    };
                    if member_type == init_type {
                        println!("Member[{}] type matches: {:?}", i, member_type);
                    } else {
                        println!("Type mismatch at member[{}]: struct type = {:?}, initializer type = {:?}", i, member_type, init_type);
                    }
                }
                if initializers.len() != struct_members.len() {
                    println!("Initializer count ({}) does not match struct member count ({})", initializers.len(), struct_members.len());
                }
            }
        }
        AstNode::Case { identifier, body } => {
            match identifier.as_ref() {
                AstNode::Literal { value, data_type } => {
                    if !is_integral_type(data_type) && value != "default" {
                        return Err(SemanticError::TypeMismatch(
                            format!("Case label must be an integral type, got {}", data_type)
                        ));
                    }
                }
                _ => {
                    return Err(SemanticError::TypeMismatch(
                        "Case label must be a constant integral expression".to_string()
                    ));
                }
            }

            for node in body {
                analyze_node(node, sem_ctx)?;
            }
        }
        AstNode::IfStatement { condition, body, else_branch } => {
            analyze_node(condition, sem_ctx)?;
            let cond_type = DT_PROMOTION.lock().unwrap()[0].clone();

            if !is_non_scalar_type(&cond_type) {
                return Err(SemanticError::TypeMismatch(
                    format!("Condition must be a scalar type, got {}", cond_type)
                ));
            }

            for node in body {
                analyze_node(node, sem_ctx)?;
            }

            if let Some(else_branch) = else_branch {
                analyze_node(else_branch, sem_ctx)?;
            }
        }
        AstNode::ElseStatement { body, if_statement } => {
            if let Some(if_statement) = if_statement {
                analyze_node(if_statement, sem_ctx)?;
            } else if let Some(body) = body {
                for node in body {
                    analyze_node(node, sem_ctx)?;
                }
            } else {
                return Err(SemanticError::InternalError(
                    "ElseStatement must have either if_statement or body".to_string()
                ));
            }
        }
        AstNode::WhileStatement { condition, body } => {
            analyze_node(condition, sem_ctx)?;
            let cond_type = DT_PROMOTION.lock().unwrap()[0].clone();

            if !is_non_scalar_type(&cond_type) {
                return Err(SemanticError::TypeMismatch(
                    format!("Condition must be a scalar type, got {}", cond_type)
                ));
            }

            for node in body {
                analyze_node(node, sem_ctx)?;
            }
        }
        AstNode::ForStatement { declarations, condition, increments, body } => {
            if let Some(declarations) = declarations {
                for declaration in declarations {
                    analyze_node(declaration, sem_ctx)?;
                }
            }

            if let Some(condition) = condition {
                analyze_node(condition, sem_ctx)?;
                let cond_type = DT_PROMOTION.lock().unwrap()[0].clone();

                if !is_non_scalar_type(&cond_type) {
                    return Err(SemanticError::TypeMismatch(
                        format!("Condition must be a scalar type, got {}", cond_type)
                    ));
                }
            }

            if let Some(increments) = increments {
                for increment in increments {
                    analyze_node(increment, sem_ctx)?;
                }
            }

            for node in body {
                analyze_node(node, sem_ctx)?;
            }
        }
        AstNode::Printf { format_string: _, arguments } => {
            for arg in arguments {
                analyze_node(arg, sem_ctx)?;
            }
        }
        AstNode::Enum { identifier: _, variants } => {
            let mut seen_variants: HashSet<&str> = HashSet::new();

            for (variant_name, variant_value) in variants {
                if !seen_variants.insert(variant_name.as_str()) {
                    return Err(SemanticError::DuplicateIdentifier(variant_name.to_string()));
                }

                if let Some(value) = variant_value {
                    if !is_enum_integral_literal(value) {
                        return Err(SemanticError::TypeMismatch(
                            format!("Enum variant '{}' must be an integral literal", variant_name)
                        ));
                    }
                }

                sem_ctx.var_table.insert_variable(
                    variant_name.clone(),
                    Type::Int,
                    true,
                    sem_ctx.scope.last().unwrap().clone(),
                ).map_err(|_e| {
                    SemanticError::DuplicateIdentifier(variant_name.to_string())
                })?;
            }
        }
        _ => {}
    }

    Ok(())
}


fn get_operator_category(operator: &str) -> Option<String> {
    match operator {
        "+" | "-" | "*" | "/" | "%" => Some("arithmetic".to_string()),
        ">" | "<" | ">=" | "<=" | "==" | "!=" => Some("comparison".to_string()),
        "&&" | "||" => Some("logical".to_string()),
        "=" | "+=" | "-=" | "*=" | "/=" | "%=" => Some("assignment".to_string()),
        "++" | "--" => Some("unary".to_string()),
        _ => None,
    }
}


fn is_assignable(left: &Type, right: &Type) -> bool {
    if left == right {
        return true;
    }

    match (left, right) {
        (Type::Array(base, _), Type::Pointer(ptr_base)) if **base == Type::Char && **ptr_base == Type::Char => true,

        (Type::Bool, Type::Int | Type::Long | Type::LongLong | Type::Short | Type::Char | Type::Float | Type::Double) => true,

        (Type::Enum(_), r) if is_integral_type(r) => true,
        (l, Type::Enum(_)) if is_integral_type(l) => true,

        (Type::Unsigned(_) | Type::Signed(_), r) if is_integral_type(r) => true,
        (l, Type::Unsigned(_) | Type::Signed(_)) if is_integral_type(l) => true,

        (Type::Double | Type::Float | Type::LongLong | Type::Long | Type::Int | Type::Short | Type::Char,
         Type::Double | Type::Float | Type::LongLong | Type::Long | Type::Int | Type::Short | Type::Char) => true,

        _ => false,
    }
}


fn is_integral_type(type_: &Type) -> bool {
    matches!(type_, 
        Type::Int | Type::Long | Type::LongLong | Type::Short | Type::Char | Type::Bool | Type::Enum(_) |
        Type::Unsigned(_) | Type::Signed(_)
    )
}


fn is_non_scalar_type(type_: &Type) -> bool {
    !matches!(type_, Type::Struct(_) | Type::Void | Type::Identifier | Type::None)
}


fn is_lvalue(node: &AstNode) -> bool {
    matches!(node,
        AstNode::Literal { data_type: Type::Identifier, .. } |
        AstNode::ArrayAccess { .. } |
        AstNode::MemberAccess { .. } |
        AstNode::Dereference { .. }
    )
}


fn is_unary_incrementable_type(type_: &Type) -> bool {
    matches!(type_,
        Type::Char | Type::Short | Type::Int | Type::Long | Type::LongLong |
        Type::Float | Type::Double | Type::LongFloat | Type::LongDouble |
        Type::Unsigned(_) | Type::Signed(_) | Type::Pointer(_) | Type::Enum(_)
    )
}


fn is_enum_integral_literal(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    let mut trimmed = String::new();
    if first == '-' {
        trimmed.push(first);
        trimmed.extend(chars);
    } else {
        trimmed.push(first);
        trimmed.extend(chars);
    }

    while trimmed.chars().last().map_or(false, |c| c.is_ascii_alphabetic()) {
        trimmed.pop();
    }

    let numeric = if trimmed.starts_with('-') {
        &trimmed[1..]
    } else {
        trimmed.as_str()
    };

    !numeric.is_empty() && numeric.chars().all(|c| c.is_ascii_digit())
}


// Type Promotion Table
// ====================
//
// Arithmetic Operators (+, -, *, /, %)
// Promotion hierarchy: char -> short -> int -> long -> long long -> float -> double
fn get_arithmetic_promotion() -> Type {
    let types = DT_PROMOTION.lock().unwrap();
    
    fn get_rank(t: &Type) -> (u32, bool) {
        match t {
            Type::Double => (9, false),
            Type::Float => (8, false),
            Type::Unsigned(inner) => {
                let (rank, _) = get_rank(inner);
                (rank, true)
            }
            Type::Signed(inner) => {
                let (rank, _) = get_rank(inner);
                (rank, false)
            }
            Type::LongLong => (7, false),
            Type::Long => (6, false),
            Type::Enum(_) => (5, false),
            Type::Int => (5, false),
            Type::Short => (4, false),
            Type::Char => (3, false),
            _ => (0, false),
        }
    }
    
    let (rank0, is_unsigned0) = get_rank(&types[0]);
    let (rank1, is_unsigned1) = get_rank(&types[1]);
    
    let result_rank = rank0.max(rank1);
    let result_is_unsigned = is_unsigned0 || is_unsigned1;
    
    let base_type = match result_rank {
        9 => Type::Double,
        8 => Type::Float,
        7 => Type::LongLong,
        6 => Type::Long,
        5 => Type::Int,
        4 => Type::Short,
        3 => Type::Char,
        _ => Type::Void,
    };
    
    if result_is_unsigned {
        Type::Unsigned(Box::new(base_type))
    } else { 
        base_type
    }
}


// pub enum AstNode {
//     ArrayAccess
//     ArrayInitializer
//     Dereference
//     DesignatedInitializer
//     MemberAccess
//     Reference
//     
//     
//     
//     Struct
// }