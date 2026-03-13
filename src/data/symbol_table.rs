use crate::data::AstNode;
use crate::data::definitions::Type;
use crate::error::{SymbolError, SymbolResult};
use std::collections::HashMap;


#[derive(Debug, Clone)]
pub enum Symbol {
    Function {
        name: String,
        return_type: Type,
        parameters: Vec<(Type, String)>,
        is_initialized: bool,
        scope: String,
    },
    Variable {
        name: String,
        var_type: Type,
        is_initialized: bool,
        scope: String,
    },
    StructType {
        name: String,
        members: Vec<AstNode>,
    },
}
    


#[derive(Debug)]
pub struct SymbolTable {
    symbols: HashMap<String, Symbol>,
}


impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            symbols: HashMap::new(),
        }
    }

    pub fn insert_function(
        &mut self,
        name: String,
        return_type: Type,
        parameters: Vec<(Type, String)>,
        initialize: bool,
        scope: String,
    ) -> SymbolResult<()> {
        if self.symbols.contains_key(&name) {
            if self.is_initialized(&name).unwrap_or(false) || !initialize {
                return Err(SymbolError::SymbolAlreadyExists { name });
            }

            self.initialize(&name, Some(return_type), Some(parameters))?;
        } else {
            self.symbols.insert(
                name.clone(),
                Symbol::Function {
                    name,
                    return_type,
                    parameters,
                    is_initialized: initialize,
                    scope,
                },
            );
        }
        
        Ok(())
    }

    pub fn insert_variable(
        &mut self,
        name: String,
        var_type: Type,
        initialize: bool,
        scope: String,
    ) -> SymbolResult<()> {
        if let Some(existing) = self.symbols.get(&name) {
            if let Symbol::Variable { scope: existing_scope, .. } = existing {
                if existing_scope == &scope {
                    return Err(SymbolError::SymbolAlreadyExists { name });
                }
            }
        }
        self.symbols.insert(
            name.clone(),
            Symbol::Variable {
                name,
                var_type,
                is_initialized: initialize,
                scope,
            },
        );
        
        Ok(())
    }

    pub fn insert_struct(
        &mut self,
        name: String,
        members: Vec<AstNode>,
    ) -> SymbolResult<()> {
        if self.symbols.contains_key(&name) {
            return Err(SymbolError::SymbolAlreadyExists { name });
        }
        self.symbols.insert(
            name.clone(),
            Symbol::StructType {
                name,
                members,
            },
        );
        Ok(())
    }

    pub fn remove(&mut self, name: &str) -> SymbolResult<()> {
        if self.symbols.remove(name).is_none() {
            return Err(SymbolError::SymbolNotFound { name: name.to_string() });
        }
        Ok(())
    }

    pub fn get_name(&self, name: &str) -> SymbolResult<String> {
        match self.symbols.get(name) {
            Some(symbol) => match symbol {
                Symbol::Function { name, .. } => Ok(name.clone()),
                Symbol::Variable { name, .. } => Ok(name.clone()),
                Symbol::StructType { name, .. } => Ok(name.clone()),
            },
            None => Err(SymbolError::SymbolNotFound { name: name.to_string() }),
        }
    }

    pub fn get_type(&self, name: &str) -> SymbolResult<Type> {
        match self.symbols.get(name) {
            Some(symbol) => match symbol {
                Symbol::Function { return_type, .. } => Ok(return_type.clone()),
                Symbol::Variable { var_type, .. } => Ok(var_type.clone()),
                Symbol::StructType { name, .. } => Ok(Type::Struct(name.clone())),
            },
            None => Err(SymbolError::SymbolNotFound { name: name.to_string() }),
        }
    }

    pub fn get_symbol_scope(&self, name: &str) -> SymbolResult<String> {
        match self.symbols.get(name) {
            Some(symbol) => match symbol {
                Symbol::Function { scope, .. } => Ok(scope.clone()),
                Symbol::Variable { scope, .. } => Ok(scope.clone()),
                Symbol::StructType { .. } => Err(SymbolError::TypeMismatch {
                    name: name.to_string(),
                    message: "Struct types do not have a scope".to_string(),
                }),
            },
            None => Err(SymbolError::SymbolNotFound { name: name.to_string() }),
        }
    }

    pub fn get_parameters(&self, name: &str) -> SymbolResult<Vec<(Type, String)>> {
        match self.symbols.get(name) {
            Some(Symbol::Function { parameters, .. }) => Ok(parameters.clone()),
            Some(Symbol::Variable { .. }) => {
                Err(SymbolError::NotAFunction { name: name.to_string() })
            }
            Some(Symbol::StructType { .. }) => {
                Err(SymbolError::NotAFunction { name: name.to_string() })
            }
            None => Err(SymbolError::SymbolNotFound { name: name.to_string() }),
        }
    }

    pub fn is_initialized(&self, name: &str) -> SymbolResult<bool> {
        match self.symbols.get(name) {
            Some(Symbol::Function { is_initialized, .. }) => Ok(*is_initialized),
            Some(Symbol::Variable { is_initialized, .. }) => Ok(*is_initialized),
            Some(Symbol::StructType { .. }) => Err(SymbolError::TypeMismatch {
                name: name.to_string(),
                message: "Struct types do not have an initialization state".to_string(),
            }),
            None => Err(SymbolError::SymbolNotFound { name: name.to_string() }),
        }
    }

    pub fn set_initialized(&mut self, name: &str, initialized: bool) -> SymbolResult<()> {
        match self.symbols.get_mut(name) {
            Some(Symbol::Function {
                is_initialized, ..
            }) => {
                *is_initialized = initialized;
                Ok(())
            }
            Some(Symbol::Variable {
                is_initialized, ..
            }) => {
                *is_initialized = initialized;
                Ok(())
            }
            Some(Symbol::StructType { .. }) => Err(SymbolError::TypeMismatch {
                name: name.to_string(),
                message: "Struct types do not have an initialization state".to_string(),
            }),
            None => Err(SymbolError::SymbolNotFound { name: name.to_string() }),
        }
    }

    pub fn initialize(
        &mut self,
        name: &str,
        return_type: Option<Type>,
        parameters: Option<Vec<(Type, String)>>,
    ) -> SymbolResult<()> {
        match self.symbols.get_mut(name) {
            Some(Symbol::Function {
                return_type: rt,
                parameters: p,
                is_initialized: ii,
                ..
            }) => {
                if let Some(ref new_return_type) = return_type {
                    if rt != new_return_type {
                        return Err(SymbolError::TypeMismatch {
                            name: name.to_string(),
                            message: "Return type does not match declaration".to_string(),
                        });
                    }
                }
                if let Some(ref new_parameters) = parameters {
                    if p.len() != new_parameters.len() {
                        return Err(SymbolError::TypeMismatch {
                            name: name.to_string(),
                            message: "Parameter count does not match declaration".to_string(),
                        });
                    }
                    for (i, (new_param_type, _)) in new_parameters.iter().enumerate() {
                        if &p[i].0 != new_param_type {
                            return Err(SymbolError::TypeMismatch {
                                name: name.to_string(),
                                message: format!("Parameter {} type does not match declaration", i + 1),
                            });
                        }
                    }
                }
                *ii = true;
                Ok(())
            }
            Some(Symbol::Variable {
                is_initialized, ..
            }) => {
                *is_initialized = true;
                Ok(())
            }
            Some(Symbol::StructType { .. }) => Err(SymbolError::TypeMismatch {
                name: name.to_string(),
                message: "Struct types do not have an initialization state".to_string(),
            }),
            None => Err(SymbolError::SymbolNotFound { name: name.to_string() }),
        }
    }

    pub fn contains(&self, name: &str) -> bool {
        self.symbols.contains_key(name)
    }

    pub fn get_struct_members(&self, name: &str) -> SymbolResult<&Vec<AstNode>> {
        match self.symbols.get(name) {
            Some(Symbol::StructType { members, .. }) => Ok(members),
            Some(_) => Err(SymbolError::TypeMismatch {
                name: name.to_string(),
                message: "Symbol is not a struct type".to_string(),
            }),
            None => Err(SymbolError::SymbolNotFound { name: name.to_string() }),
        }
    }
    
    pub fn get_all(&self) -> Vec<&Symbol> {
        self.symbols.values().collect()
    }

    pub fn clear(&mut self) {
        self.symbols.clear();
    }
}