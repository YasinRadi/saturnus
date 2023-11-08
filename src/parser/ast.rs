use super::grammar::Script;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decorator {
    pub target: CallExpression,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Argument {
    pub name: Identifier,
    pub decorators: Vec<Decorator>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub name: Identifier,
    pub arguments: Vec<Argument>,
    pub decorators: Vec<Decorator>,
    pub body: Script,
    pub native: Option<Vec<(Identifier, StringLiteral)>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lambda {
    pub arguments: Vec<Argument>,
    pub body: ScriptOrExpression,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Do {
    pub body: Script,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tuple(pub Vec<Expression>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identifier(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemberSegment {
    Computed(Expression),
    IdentifierDynamic(Identifier),
    IdentifierStatic(Identifier),
}
impl Into<CallExpressionVariant> for MemberSegment {
    fn into(self) -> CallExpressionVariant {
        CallExpressionVariant::Member(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberExpression {
    pub head: Expression,
    pub tail: Vec<MemberSegment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DestructureOrigin {
    Tuple,
    Array,
    Table,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Destructuring(pub Vec<Identifier>, pub DestructureOrigin);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssignmentTarget {
    Destructuring(Destructuring),
    Identifier(Identifier),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Let {
    pub target: AssignmentTarget,
    pub value: Option<Expression>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assignment {
    pub target: MemberExpression,
    pub value: Expression,
    pub extra: Option<Operator>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Class {
    pub name: Identifier,
    pub decorators: Vec<Decorator>,
    pub fields: Vec<ClassField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallSubExpression {
    pub callee: Option<MemberExpression>,
    pub arguments: Vec<Expression>,
}
impl Into<CallExpressionVariant> for CallSubExpression {
    fn into(self) -> CallExpressionVariant {
        CallExpressionVariant::Call(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CallExpressionVariant {
    Call(CallSubExpression),
    Member(MemberSegment),
}

// TODO: Implement macros!
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MacroCallArguments {
    FunctionLike(Vec<Expression>),
    BlockLike(Script),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentifierOrCall {
    pub target: Identifier,
    pub arguments: Vec<Expression>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroDecorator {
    pub macros: Vec<IdentifierOrCall>,
    pub target: Statement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroCallExpression {
    pub target: Identifier,
    pub arguments: MacroCallArguments,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallExpression {
    pub head: CallSubExpression,
    pub tail: Vec<CallExpressionVariant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Return {
    pub value: Expression,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Number {
    Float(f64),
    Integer(i64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct If {
    pub condition: Expression,
    pub body: Script,
    pub branches: Vec<(Expression, Script)>,
    pub else_branch: Option<Script>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct For {
    pub handler: AssignmentTarget,
    pub target: Expression,
    pub body: Script,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExpressionOrLet {
    Expression(Expression),
    Let(Let),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct While {
    pub condition: ExpressionOrLet,
    pub body: Script,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Loop {
    pub body: Script,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScriptOrExpression {
    Script(Script),
    Expression(Expression),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Match {
    pub target: Expression,
    pub branches: Vec<(Expression, ScriptOrExpression)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Statement {
    MacroDecorator(Box<MacroDecorator>),
    If(If),
    Match(Match),
    For(For),
    Loop(Loop),
    While(While),
    Return(Return),
    Class(Class),
    Function(Function),
    MacroDeclare(Function),
    Assignment(Assignment),
    Let(Let),
    Expression(Expression),
    Use(Identifier),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClassField {
    Method(Function),
    Let(Let),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operator(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryExpression {
    pub left: Expression,
    pub right: Expression,
    pub operator: Operator,
}
impl Into<Expression> for BinaryExpression {
    fn into(self) -> Expression {
        Expression::Binary(Box::new(self))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnaryExpression {
    pub expression: Expression,
    pub operator: Operator,
}
impl Into<Expression> for UnaryExpression {
    fn into(self) -> Expression {
        Expression::Unary(Box::new(self))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vector {
    pub expressions: Vec<Expression>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub key_values: Vec<(TableKeyExpression, Option<Expression>)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TableKeyExpression {
    Identifier(Identifier),
    Expression(Expression),
    Implicit(Identifier),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StringLiteral {
    Double(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expression {
    Lambda(Box<Lambda>),
    Reference(Box<MemberExpression>),
    Identifier(Identifier),
    Call(Box<CallExpression>),
    Tuple(Tuple),
    Tuple1(Box<Expression>),
    Table(Table),
    Do(Do),
    Use(Identifier),
    Vector(Vector),
    Number(Number),
    String(StringLiteral),
    Binary(Box<BinaryExpression>),
    Unary(Box<UnaryExpression>),
    Unit,
}
