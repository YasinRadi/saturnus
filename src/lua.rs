use crate::{
    code::{self},
    parser::ast::{self},
};

pub struct LuaEmitter;

impl code::Visitor<code::Builder> for LuaEmitter {
    fn visit_return(
        &self,
        ctx: code::Builder,
        stmt: &ast::Return,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = ctx.line().put("return ");
        let ctx = self.visit_expression(ctx, &stmt.value)?;
        Ok(ctx.put(";"))
    }

    fn visit_class(
        &self,
        ctx: code::Builder,
        stmt: &ast::Class,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = ctx
            .line()
            .put(format!("local {} = {{}};", stmt.name.0.clone()))
            .line()
            .put(format!("{}.__meta__ = {{}};", stmt.name.0.clone()))
            .line()
            .put(format!(
                "{}.__meta__.__call = function(self, struct)",
                stmt.name.0.clone()
            ))
            .push()
            .line()
            .put("return setmetatable(struct, self.prototype.__meta__);")
            .pop()
            .unwrap()
            .line()
            .put("end;")
            .line()
            .put(format!("{}.prototype = {{}};", stmt.name.0.clone()))
            .line()
            .put(format!(
                "{}.prototype.__meta__ = {{}};",
                stmt.name.0.clone()
            ))
            .line()
            .put(format!(
                "{}.prototype.__meta__.__index = {}.prototype;",
                stmt.name.0.clone(),
                stmt.name.0.clone()
            ))
            .line()
            .put(format!(
                "setmetatable({}, {}.__meta__);",
                stmt.name.0.clone(),
                stmt.name.0.clone()
            ));
        let ctx = stmt.fields.iter().fold(Ok(ctx), |ctx, field| {
            let ctx = ctx?.line();
            let ctx = match field {
                ast::ClassField::Method(f) => {
                    let level = if let Some(first) = f.arguments.first() {
                        if first.name.0 == "self" {
                            ".prototype."
                        } else {
                            "."
                        }
                    } else {
                        "."
                    }
                    .to_string();
                    let ctx = ctx
                        .put(stmt.name.0.clone())
                        .put(level)
                        .put(f.name.0.clone())
                        .put(" = ");
                    let ctx = self.visit_lambda(
                        ctx,
                        &ast::Lambda {
                            arguments: f.arguments.clone(),
                            body: ast::ScriptOrExpression::Script(f.body.clone()),
                        },
                    )?;
                    ctx.put(";")
                }
                ast::ClassField::Let(f) => {
                    let ctx = ctx.put(format!(
                        "{}.prototype.{} = ",
                        stmt.name.0.clone(),
                        f.target.0.clone()
                    ));
                    let ctx = if let Some(value) = f.value.as_ref() {
                        self.visit_expression(ctx, value)?
                    } else {
                        ctx.put("nil")
                    };
                    ctx.put(";")
                }
                ast::ClassField::Operator(f) => {
                    let target = match f.operator {
                        ast::Operator::Plus => "__add",
                        ast::Operator::Minus => "__sub",
                        ast::Operator::Product => "__mul",
                        ast::Operator::Quotient => "__div",
                        ast::Operator::Remainder => "__mod",
                        ast::Operator::Power => "__pow",
                        ast::Operator::Equal => "__eq",
                        ast::Operator::Less => "__lt",
                        ast::Operator::LessEqual => "__le",
                        ast::Operator::Concat => "__concat",
                        ast::Operator::Count => "__len",
                        _ => todo!(
                            "Operator overload for {:?} operator not supported",
                            f.operator.clone()
                        ),
                    };
                    let ctx = ctx.put(format!(
                        "{}.prototype.__meta__.{} = ",
                        stmt.name.0.clone(),
                        target
                    ));
                    let ctx = self.visit_lambda(
                        ctx,
                        &ast::Lambda {
                            arguments: f.arguments.clone(),
                            body: ast::ScriptOrExpression::Script(f.body.clone()),
                        },
                    )?;
                    ctx.put(";")
                }
            };
            let ctx = stmt.decorators.iter().fold(Ok(ctx), |ctx, dec| {
                let ctx = ctx?.line();
                let ctx = self.visit_expression(ctx, &dec.target)?;
                let ctx = ctx.put(format!(
                    "({}, \"{}\");",
                    stmt.name.0.clone(),
                    stmt.name.0.clone()
                ));
                Ok(ctx)
            })?;
            Ok(ctx)
        })?;
        Ok(ctx)
    }

    fn visit_fn(
        &self,
        ctx: code::Builder,
        stmt: &ast::Function,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = ctx
            .line()
            .put("local function ")
            .put(stmt.name.0.clone())
            .put("(");
        let ctx = if let Some(first) = stmt.arguments.first() {
            ctx.put(first.name.0.clone())
        } else {
            ctx
        };
        let ctx = stmt
            .arguments
            .iter()
            .skip(1)
            .fold(ctx, |ctx, ident| ctx.put(", ").put(ident.name.0.clone()));
        let ctx = ctx.put(")").push();
        let ctx = self.visit_script(ctx, &stmt.body)?;
        let ctx = ctx.pop().unwrap().line().put("end");
        let ctx = stmt.decorators.iter().fold(Ok(ctx), |ctx, dec| {
            let ctx = ctx?.line();
            let ctx = self.visit_expression(ctx, &dec.target)?;
            let ctx = ctx.put(format!(
                "({}, \"{}\");",
                stmt.name.0.clone(),
                stmt.name.0.clone()
            ));
            Ok(ctx)
        })?;
        Ok(ctx)
    }

    fn visit_assignment(
        &self,
        ctx: code::Builder,
        stmt: &ast::Assignment,
    ) -> Result<code::Builder, code::VisitError> {
        let segment = self.visit_reference(ctx.clone_like(), &stmt.target)?.collect();
        let ctx = ctx.line().put(segment).put(" = ");
        let ctx = if let Some(extra) = stmt.extra.as_ref() {
            let ast::Assignment { target, value, .. } = stmt.clone();
            self.visit_binary(
                ctx,
                &ast::BinaryExpression {
                    left: ast::Expression::Reference(target),
                    operator: extra.clone(),
                    right: value,
                },
            )
        } else {
            self.visit_expression(ctx, &stmt.value)
        }?;
        let ctx = ctx.put(";");
        Ok(ctx)
    }

    fn visit_declaration(
        &self,
        ctx: code::Builder,
        stmt: &ast::Let,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = ctx
            .line()
            .put("local ")
            .put(stmt.target.0.clone())
            .put(" = ");
        let ctx = self.visit_expression(ctx, stmt.value.as_ref().unwrap())?;
        let ctx = ctx.put(";");
        Ok(ctx)
    }

    fn visit_expression_statement(
        &self,
        ctx: code::Builder,
        stmt: &ast::Expression,
    ) -> Result<code::Builder, code::VisitError> {
        Ok(self.visit_expression(ctx.line(), stmt)?.put(";"))
    }

    fn visit_lambda(
        &self,
        ctx: code::Builder,
        expr: &ast::Lambda,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = ctx.put("function(");
        let ctx = if let Some(first) = expr.arguments.first() {
            ctx.put(first.name.0.clone())
        } else {
            ctx
        };
        let ctx = expr
            .arguments
            .iter()
            .skip(1)
            .fold(ctx, |ctx, ident| ctx.put(", ").put(ident.name.0.clone()));
        let ctx = ctx.put(")").push();
        let ctx = match &expr.body {
            ast::ScriptOrExpression::Script(e) => self.visit_script(ctx, e)?,
            ast::ScriptOrExpression::Expression(e) => self
                .visit_expression(ctx.line().put("return "), e)
                .map(|b| b.put(";"))?,
        };
        Ok(ctx.pop().unwrap().line().put("end"))
    }

    fn visit_reference(
        &self,
        ctx: code::Builder,
        expr: &ast::DotExpression,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = if let Some(first) = expr.0.first() {
            match first {
                ast::DotSegment::Identifier(e) => 
                ctx.put(e.0.clone()),
                ast::DotSegment::Expression(_) => panic!("Found an array access segment as first item! Perhaps you forgot to wrap [] between parens? ([])"),
            }
        } else {
            ctx
        };
        let ctx = expr
            .0
            .iter()
            .skip(1)
            .fold(Ok(ctx), |ctx, target| {
                match target {
                    ast::DotSegment::Identifier(e) =>
                        Ok(ctx?.put(".").put(e.0.clone())),
                    ast::DotSegment::Expression(e) => {
                        let ctx = ctx?.put("[");
                        let ctx = self.visit_expression(ctx, e)?;
                        Ok(ctx.put("]"))
                    },
                }
            })?;
        Ok(ctx)
    }

    fn visit_call(
        &self,
        ctx: code::Builder,
        expr: &ast::CallExpression,
    ) -> Result<code::Builder, code::VisitError> {
        let segment = self.visit_reference(ctx.clone_like(),
            &ast::DotExpression(
                expr.target
                    .0
                    .iter()
                    .rev()
                    .skip(1)
                    .rev()
                    .map(|x| x.clone())
                    .collect(),
            ))?.collect();
        let ctx = ctx.put(segment.clone());
        let last = expr.target.0.last();
        let ctx = if let Some(sc) = expr.static_target.as_ref() {
            let ctx = if let Some(last) = last {
                let ctx = if segment.len() > 0 {
                    ctx.put(".")
                } else {ctx};
                self.visit_reference(ctx, &ast::DotExpression(vec![last.clone()]))?
            } else {ctx};
            ctx.put(".").put(sc.0.clone())
        } else {
            if let Some(last) = last {
                let ctx = if segment.len() > 0 {
                    ctx.put(":")
                } else {ctx};
                self.visit_reference(ctx, &ast::DotExpression(vec![last.clone()]))?
            } else {
                ctx
            }
        };
        let ctx = ctx.put("(");
        let ctx = if let Some(first) = expr.arguments.first() {
            self.visit_expression(ctx, first)?
        } else {
            ctx
        };
        let ctx = expr.arguments.iter().skip(1).fold(Ok(ctx), |ctx, expr| {
            self.visit_expression(ctx.map(|b| b.put(", "))?, expr)
        })?;
        Ok(ctx.put(")"))
    }

    fn visit_tuple(
        &self,
        ctx: code::Builder,
        expr: &ast::Tuple,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = ctx.put("{");
        let ctx = if let Some(first) = expr.0.first().as_ref() {
            let ctx = ctx.put(format!("_0 = "));
            self.visit_expression(ctx, first)?
        } else {
            ctx
        };
        let ctx = expr
            .0
            .iter()
            .skip(1)
            .fold(Ok((ctx, 1_u16)), |ctx, value| {
                let (ctx, i) = ctx?;
                let ctx = ctx.put(format!(", _{} = ", i));
                let ctx = self.visit_expression(ctx, value)?;
                Ok((ctx, i + 1))
            })?
            .0;
        let ctx = ctx.put("}");
        Ok(ctx)
    }

    fn visit_number(
        &self,
        ctx: code::Builder,
        expr: &ast::Number,
    ) -> Result<code::Builder, code::VisitError> {
        let repr = match expr {
            ast::Number::Float(e) => e.to_string(),
            ast::Number::Integer(e) => e.to_string(),
        };
        Ok(ctx.put(repr))
    }

    fn visit_string(
        &self,
        ctx: code::Builder,
        expr: &String,
    ) -> Result<code::Builder, code::VisitError> {
        Ok(ctx.put("\"").put(expr.clone()).put("\""))
    }

    fn visit_unit(&self, ctx: code::Builder) -> Result<code::Builder, code::VisitError> {
        Ok(ctx.put("nil"))
    }

    fn visit_binary(
        &self,
        ctx: code::Builder,
        expr: &ast::BinaryExpression,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = self.visit_expression(ctx, &expr.left)?.put(" ");
        let ctx = match expr.operator.clone() {
            // Basic math
            ast::Operator::Plus => ctx.put("+"),
            ast::Operator::Minus => ctx.put("-"),
            ast::Operator::Product => ctx.put("*"),
            ast::Operator::Quotient => ctx.put("/"),
            ast::Operator::Remainder => ctx.put("%"),
            ast::Operator::Power => ctx.put("**"),
            ast::Operator::Concat => ctx.put(".."),
            // Comparison
            ast::Operator::Greater => ctx.put(">"),
            ast::Operator::GreaterEqual => ctx.put(">="),
            ast::Operator::Less => ctx.put("<"),
            ast::Operator::LessEqual => ctx.put("<="),
            ast::Operator::Equal => ctx.put("=="),
            ast::Operator::NotEqual => ctx.put("~="),
            // Logic
            ast::Operator::LogicNot => ctx.put("not"),
            ast::Operator::LogicAnd => ctx.put("or"),
            ast::Operator::LogicOr => ctx.put("and"),
            op => todo!("Binary operator {:?} not supported!", op),
        };
        let ctx = self.visit_expression(ctx.put(" "), &expr.right)?;
        Ok(ctx)
    }

    fn visit_unary(
        &self,
        ctx: code::Builder,
        expr: &ast::UnaryExpression,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = match expr.operator {
            ast::Operator::Minus => ctx.put("-"),
            _ => todo!("Unary operator not supported!"),
        };
        let ctx = self.visit_expression(ctx, &expr.expression)?;
        Ok(ctx)
    }

    fn visit_if(
        &self,
        ctx: code::Builder,
        expr: &ast::If,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = ctx.line().put("if ");
        let ctx = self.visit_expression(ctx, &expr.condition)?;
        let ctx = ctx.put(" then").push();
        let ctx = self.visit_script(ctx, &expr.body)?;
        let ctx = expr.branches.iter().fold(Ok(ctx), |ctx, (c, s)| {
            let ctx = ctx?.pop().unwrap().line().put("elseif ");
            let ctx = self.visit_expression(ctx, c)?;
            let ctx = ctx.put(" then").push();
            let ctx = self.visit_script(ctx, s)?;
            Ok(ctx)
        })?;
        let ctx = if let Some(eb) = expr.else_branch.as_ref() {
            let ctx = ctx.pop().unwrap().line().put("else").push();
            self.visit_script(ctx, eb)?
        } else {
            ctx
        };
        let ctx = ctx.pop().unwrap().line().put("end");
        Ok(ctx)
    }

    fn visit_table(
        &self,
        ctx: code::Builder,
        expr: &ast::Table,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = ctx.put("{");
        let ctx = if let Some((k, v)) = expr.key_values.first() {
            match k {
                ast::TableKeyExpression::Identifier(k) => {
                    let ctx = ctx.put(k.0.clone()).put(" = ");
                    self.visit_expression(ctx, v)
                }
                ast::TableKeyExpression::Expression(k) => {
                    let ctx = self.visit_expression(ctx, k)?.put(" = ");
                    self.visit_expression(ctx, v)
                }
                ast::TableKeyExpression::Implicit(k) => {
                    let ctx = ctx.put(k.0.clone()).put(" = ");
                    self.visit_expression(
                        ctx,
                        &ast::Expression::Reference(
                            ast::DotExpression(vec![ast::DotSegment::Identifier(k.clone())])),
                    )
                }
            }?
        } else {
            ctx
        };
        let ctx = expr
            .key_values
            .iter()
            .skip(1)
            .fold(Ok(ctx), |ctx, (k, v)| {
                let ctx = ctx?.put(", ");
                match k {
                    ast::TableKeyExpression::Identifier(k) => {
                        let ctx = ctx.put(k.0.clone()).put(" = ");
                        self.visit_expression(ctx, v)
                    }
                    ast::TableKeyExpression::Expression(k) => {
                        let ctx = ctx.put("[");
                        let ctx = self.visit_expression(ctx, k)?.put("] = ");
                        self.visit_expression(ctx, v)
                    }
                    ast::TableKeyExpression::Implicit(k) => {
                        let ctx = ctx.put(k.0.clone()).put(" = ");
                        self.visit_expression(
                            ctx,
                            &ast::Expression::Reference(
                                ast::DotExpression(vec![ast::DotSegment::Identifier(k.clone())])),
                        )
                    }
                }
            })?;
        Ok(ctx.put("}"))
    }

    fn visit_vector(
        &self,
        ctx: code::Builder,
        expr: &ast::Vector,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = ctx.put("{");
        let ctx = if let Some(first) = expr.expressions.first() {
            self.visit_expression(ctx, first)?
        } else {
            ctx
        };
        let ctx = expr.expressions.iter().skip(1).fold(Ok(ctx), |ctx, v| {
            let ctx = ctx?.put(", ");
            let ctx = self.visit_expression(ctx, v)?;
            Ok(ctx)
        })?;
        Ok(ctx.put("}"))
    }

    fn visit_for(
        &self,
        ctx: code::Builder,
        expr: &ast::For,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = ctx
            .line()
            .put(format!("for {} in ", expr.handler.0.clone()));
        let ctx = self.visit_expression(ctx, &expr.target)?;
        let ctx = ctx.put(" do").push();
        let ctx = self.visit_script(ctx, &expr.body)?;
        let ctx = ctx.pop().unwrap().line().put("end");
        Ok(ctx)
    }

    fn visit_while(
        &self,
        ctx: code::Builder,
        expr: &ast::While,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = match &expr.condition {
            ast::ExpressionOrLet::Expression(e) => {
                let ctx = ctx.line().put("while ");
                let ctx = self.visit_expression(ctx, e)?;
                let ctx = ctx.put(" do").push();
                let ctx = self.visit_script(ctx, &expr.body)?;
                ctx.pop().unwrap().line().put("end")
            }
            ast::ExpressionOrLet::Let(e) => {
                let ctx = ctx.line().put("do").push();
                let ctx = self.visit_declaration(ctx, e)?;
                let ctx = ctx
                    .line()
                    .put(format!("while {} do", e.target.0.clone()))
                    .push();
                let ctx = self.visit_script(ctx, &expr.body)?;
                let ctx = self.visit_assignment(
                    ctx,
                    &ast::Assignment {
                        target: ast::DotExpression(vec![ast::DotSegment::Identifier(e.target.clone())]),
                        value: e.value.clone().unwrap(),
                        extra: None,
                    },
                )?;
                ctx.pop()
                    .unwrap()
                    .line()
                    .put("end")
                    .pop()
                    .unwrap()
                    .line()
                    .put("end")
            }
        };
        Ok(ctx)
    }

    fn visit_loop(
        &self,
        ctx: code::Builder,
        expr: &ast::Loop,
    ) -> Result<code::Builder, code::VisitError> {
        let ctx = ctx.line().put("while true do").push();
        let ctx = self.visit_script(ctx, &expr.body)?;
        let ctx = ctx.pop().unwrap().line().put("end");
        Ok(ctx)
    }

    fn visit_match(
        &self,
        ctx: code::Builder,
        expr: &ast::Match,
    ) -> Result<code::Builder, code::VisitError> {
        todo!()
    }
}
