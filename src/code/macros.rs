use std::collections::HashMap;

use crate::parser::{
    ast::{self, Function, IdentifierOrCall, Statement},
    Script,
};

#[derive(Debug)]
pub enum MacroExpansionError {
    Unknown,
    InvalidCode,
    MacroNotDefined(String),
    ExpansionTimeError(Option<Box<dyn std::error::Error>>),
}

pub struct MacroData {
    pub func: Function,
    pub runtime_code: String,
}
impl MacroData {
    pub fn compiled(func: Function) -> Result<MacroData, MacroExpansionError> {
        use crate::code::{ast_visitor::Visitor, builder::Builder};
        use crate::lua::visitor::LuaEmitter;
        let ctx = Builder::new("  ");
        let runtime_code = LuaEmitter::new()
            .visit_fn(ctx, &func)
            .map_err(|_| MacroExpansionError::InvalidCode)?
            .collect();
        Ok(MacroData { func, runtime_code })
    }
}

pub struct MacroExpander {
    pub macros: HashMap<String, MacroData>,
}

impl MacroExpander {
    pub fn new() -> MacroExpander {
        MacroExpander {
            macros: HashMap::new(),
        }
    }
    /// Grabs all macro declarations and compiles them away.
    ///
    /// This operation strips macro declarations from the AST.
    ///
    /// ```rs
    /// macro test(ast) {/* Example code */}
    /// ```
    pub fn compile_macros(&mut self, script: &Script) -> Result<Script, MacroExpansionError> {
        let statements = script
            .statements
            .iter()
            .filter(|stmt| {
                if let Statement::MacroDeclare(func) = stmt {
                    self.macros.insert(
                        func.name.0.clone(),
                        MacroData::compiled(func.clone()).unwrap(),
                    );
                    false
                } else {
                    true
                }
            })
            .map(|c| c.clone())
            .collect::<Vec<Statement>>();
        Ok(Script { statements })
    }

    /// Takes all macro invocations, either by explicit invocation or by implicit
    /// decoration, and applies them the compiled macro code.
    pub fn expand_macros(&self, script: &Script) -> Result<Script, MacroExpansionError> {
        let rt = rlua::Lua::new();
        let statements: Vec<Statement> = vec![];
        for stmt in script.statements.iter() {
            if let Statement::MacroDecorator(dec) = stmt {
                for expander in dec.macros.iter() {
                    if let Some(found) = self.macros.get(&expander.target.0) {
                        rt.context(|ctx| {
                            ctx.load(&found.runtime_code).eval().unwrap();
                        });
                    } else {
                        Err(MacroExpansionError::MacroNotDefined(
                            expander.target.0.clone(),
                        ))?;
                    }
                }
            } else {
                statements.push(stmt);
            }
        }
        Ok(Script { statements })
    }
}

pub fn serialize_statement(ast: &ast::Statement) -> serde_json::Result<ast::Table> {
    // serde_json::
    todo!()
}

pub mod builder {
    type KeyValue = (TableKeyExpression, Option<Expression>);

    use crate::parser::ast::{
        CallExpression, CallSubExpression, Expression, Identifier, MemberExpression, StringLiteral,
        Table, TableKeyExpression,
    };

    pub fn table_key(key: String) -> TableKeyExpression {
        TableKeyExpression::Identifier(Identifier(key))
    }

    pub fn key_value(key: String, expr: Expression) -> KeyValue {
        (table_key(key), Some(expr))
    }

    pub fn string(value: String) -> Expression {
        Expression::String(StringLiteral::Double(value))
    }

    pub fn table(key_values: Vec<KeyValue>) -> Table {
        Table { key_values }
    }

    pub fn create_simple_call(target: String, arguments: Vec<Expression>) -> CallExpression {
        CallExpression {
            head: CallSubExpression {
                callee: Some(MemberExpression {
                    head: Expression::Identifier(Identifier(target)),
                    tail: vec![],
                }),
                arguments,
            },
            tail: vec![],
        }
    }
}
