use crate::parser::{
    ast::{self, IdentifierOrCall, Statement},
    Script,
};

pub struct MacroData {
    pub statement: Statement,
    pub target: IdentifierOrCall,
}

pub struct MacroExpander {
    pub macros: Vec<MacroData>,
}

#[derive(Debug)]
pub enum MacroExpansionError {
    Unknown,
}

impl MacroExpander {
    pub fn new() -> MacroExpander {
        MacroExpander { macros: vec![] }
    }
    pub fn compile_macros(&mut self, script: &Script) -> Result<Script, MacroExpansionError> {
        let statements = script
            .statements
            .iter()
            .filter(|stmt| {
                if let Statement::MacroDecorator(stmt) = stmt {
                    for target in stmt.macros.iter() {
                        self.macros.push(MacroData {
                            statement: stmt.target.clone(),
                            target: target.clone(),
                        });
                    }
                    false
                } else {
                    true
                }
            })
            .map(|c| c.clone())
            .collect::<Vec<Statement>>();
        Ok(Script { statements })
    }
    pub fn expand_macros(&self, script: &Script) -> Result<Script, MacroExpansionError> {
        let statements = script
            .statements
            .iter()
            .map(|stmt| {})
            .collect::<Vec<Statement>>();
        todo!()
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
