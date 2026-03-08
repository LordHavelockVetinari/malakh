use std::collections::VecDeque;
use std::io::Read;
use std::rc::Rc;

use crate::parse::error::ParseError;
use crate::parse::location::Location;
use crate::parse::normalized_token_stream::NormalizedTokenStream;
use crate::parse::token::TokenType::*;
use crate::parse::token::{Token, TokenType};
use crate::parse::tree::{
    Argument, ArgumentType, Assignment, AssignmentTarget, AssignmentType, BinaryOperator, Block,
    CodeFile, Condition, ConstantLiteral, Expr, ExprType, GlobalDeclaration, ImportDeclaration,
    JumpType, RaiseType, Stmt, StmtType, SwitchCase, TryBlock, UnaryOperator,
};

mod error;
pub mod location;
mod normalized_token_stream;
mod token;
pub mod tree;

enum ExprTypeRestrictions {
    Any,
    NoSend,
}

pub struct Parser {
    source: NormalizedTokenStream,
    token_buf: VecDeque<Token>,
}

impl Parser {
    pub fn new(filename: &str, source: Box<dyn Read>) -> Self {
        Self {
            source: NormalizedTokenStream::new(filename, source),
            token_buf: VecDeque::new(),
        }
    }

    fn token_at(&mut self, i: usize) -> Result<&Token, ParseError> {
        while self.token_buf.len() <= i {
            let tok = self.source.next().unwrap();
            self.token_buf.push_back(tok?);
        }
        Ok(&self.token_buf[i])
    }

    fn skip(&mut self, i: usize) {
        assert!(
            i <= self.token_buf.len(),
            "cannot skip more characters than buffer size"
        );
        self.token_buf.drain(..i);
    }

    fn consume_if<F>(&mut self, f: F) -> Result<Option<Token>, ParseError>
    where
        F: FnOnce(&TokenType) -> bool,
    {
        if !f(&self.token_at(0)?.0) {
            return Ok(None);
        }
        Ok(self.token_buf.pop_front())
    }

    fn err<S: Into<String>, R>(&self, message: S, location: Location) -> Result<R, ParseError> {
        Err(ParseError::Parse {
            location,
            message: message.into(),
        })
    }

    fn err_here<S: Into<String>, R>(&mut self, message: S) -> Result<R, ParseError> {
        let location = self.token_at(0)?.1.clone();
        self.err(message, location)
    }

    fn parse_ident(&mut self) -> Result<Option<String>, ParseError> {
        let Some(ident) = self.consume_if(|t| matches!(t, Identifier(..)))? else {
            return Ok(None);
        };
        let Identifier(ident) = ident.0 else {
            unreachable!();
        };
        Ok(Some(ident))
    }

    // Matches "::Foo::Bar::Baz".
    fn parse_path_extension(&mut self, output: &mut Vec<String>) -> Result<(), ParseError> {
        while let Some(colon_colon) = self.consume_if(|t| matches!(t, ColonColon))? {
            let Some(part) = self.parse_ident()? else {
                return self.err("expected an identifier after `::`", colon_colon.1);
            };
            output.push(part);
        }
        Ok(())
    }

    fn parse_path(&mut self) -> Result<Vec<String>, ParseError> {
        let Some(first) = self.parse_ident()? else {
            return self.err_here("expected an identifier");
        };
        let mut result = vec![first];
        self.parse_path_extension(&mut result)?;
        Ok(result)
    }

    fn parse_simple_expr(&mut self) -> Result<Rc<Expr>, ParseError> {
        if let Some(int) = self.consume_if(|typ| matches!(typ, Integer(_)))? {
            let Token(Integer(n), location) = int else {
                unreachable!();
            };
            Ok(Rc::new(Expr(
                ExprType::ConstantLiteral(ConstantLiteral::Integer(n)),
                location,
            )))
        } else if let Some(float) = self.consume_if(|typ| matches!(typ, Float(_)))? {
            let Token(Float(x), location) = float else {
                unreachable!();
            };
            Ok(Rc::new(Expr(
                ExprType::ConstantLiteral(ConstantLiteral::Float(x)),
                location,
            )))
        } else if let Some(str_token) = self.consume_if(|t| matches!(t, StringLiteral(_)))? {
            let Token(StringLiteral(s), location) = str_token else {
                unreachable!();
            };
            Ok(Rc::new(Expr(
                ExprType::ConstantLiteral(ConstantLiteral::String(s)),
                location,
            )))
        } else if let Some(sym) = self.consume_if(|typ| matches!(typ, Symbol(_)))? {
            let Token(Symbol(name), location) = sym else {
                unreachable!();
            };
            Ok(Rc::new(Expr(
                ExprType::ConstantLiteral(ConstantLiteral::Symbol(name)),
                location,
            )))
        } else if let Some(ident) = self.consume_if(|typ| matches!(typ, Identifier(_)))? {
            let Token(Identifier(s), location) = ident else {
                unreachable!();
            };
            if !matches!(self.token_at(0)?.0, ColonColon) {
                return Ok(Rc::new(Expr(ExprType::Identifier(s), location)));
            }
            let mut parts = vec![s];
            self.parse_path_extension(&mut parts)?;
            Ok(Rc::new(Expr(ExprType::Path(parts), location)))
        } else if let Some(in_token) = self.consume_if(|typ| matches!(typ, KeywordIn))? {
            Ok(Rc::new(Expr(ExprType::In, in_token.1)))
        } else if let Some(lparen) = self.consume_if(|typ| matches!(typ, LParen))? {
            let inner = self.parse_expr()?;
            if self.consume_if(|typ| matches!(typ, RParen))?.is_none() {
                self.err_here("expected a right parenthesis (')')")?;
            }
            Ok(Rc::new(Expr(
                ExprType::Parenthesized(inner),
                lparen.1.clone(),
            )))
        } else if let Some(lparen) = self.consume_if(|typ| matches!(typ, LBracket))? {
            let inner = self.parse_expr()?;
            if self.consume_if(|typ| matches!(typ, RBracket))?.is_none() {
                self.err_here("expected a right bracket (']')")?;
            }
            Ok(Rc::new(Expr(ExprType::Receive(inner), lparen.1.clone())))
        } else if matches!(self.token_at(0)?.0, LBrace) {
            let body = self.parse_block()?;
            Ok(Rc::new(Expr(
                ExprType::ProcessLiteral(body.stmts),
                body.location,
            )))
        } else {
            self.err_here("expected an expression")
        }
    }

    fn parse_unary_expr(&mut self) -> Result<Rc<Expr>, ParseError> {
        let Some(op_token) = self.consume_if(|typ| matches!(typ, Plus | Minus | KeywordNot))?
        else {
            return self.parse_simple_expr();
        };
        let un_op = match op_token.0 {
            Plus => UnaryOperator::Plus,
            Minus => UnaryOperator::Minus,
            KeywordNot => UnaryOperator::Not,
            _ => unreachable!(),
        };
        let rhs = self.parse_unary_expr()?;
        Ok(Rc::new(Expr(ExprType::Unary(un_op, rhs), op_token.1)))
    }

    fn parse_op_expr1(&mut self, start: Option<Rc<Expr>>) -> Result<Rc<Expr>, ParseError> {
        let lhs = match start {
            Some(start) => start,
            None => self.parse_unary_expr()?,
        };
        if let Some(op_token) = self.consume_if(|typ| matches!(typ, Caret))? {
            let rhs = self.parse_op_expr1(None)?;
            Ok(Rc::new(Expr(
                ExprType::Binary(BinaryOperator::Power, lhs, rhs),
                op_token.1,
            )))
        } else {
            Ok(lhs)
        }
    }

    fn parse_op_expr2(&mut self, start: Option<Rc<Expr>>) -> Result<Rc<Expr>, ParseError> {
        let mut lhs = self.parse_op_expr1(start)?;
        while let Some(op_token) =
            self.consume_if(|typ| matches!(typ, Asterisk | Slash | Percent))?
        {
            let rhs = self.parse_op_expr1(None)?;
            let op = match op_token.0 {
                Asterisk => BinaryOperator::Multiply,
                Slash => BinaryOperator::Divide,
                Percent => BinaryOperator::Remainder,
                _ => unreachable!(),
            };
            lhs = Rc::new(Expr(ExprType::Binary(op, lhs, rhs), op_token.1));
        }
        Ok(lhs)
    }

    fn parse_op_expr3(&mut self, start: Option<Rc<Expr>>) -> Result<Rc<Expr>, ParseError> {
        let mut lhs = self.parse_op_expr2(start)?;
        while let Some(op_token) = self.consume_if(|typ| matches!(typ, Plus | Minus))? {
            let rhs = self.parse_op_expr2(None)?;
            let op = match op_token.0 {
                Plus => BinaryOperator::Add,
                Minus => BinaryOperator::Subtract,
                _ => unreachable!(),
            };
            lhs = Rc::new(Expr(ExprType::Binary(op, lhs, rhs), op_token.1));
        }
        Ok(lhs)
    }

    fn parse_op_expr4(&mut self, start: Option<Rc<Expr>>) -> Result<Rc<Expr>, ParseError> {
        let mut lhs = self.parse_op_expr3(start)?;
        while let Some(op_token) = self.consume_if(|typ| {
            matches!(
                typ,
                EqualsEquals | BangEquals | Less | LessEquals | Greater | GreaterEquals
            )
        })? {
            let rhs = self.parse_op_expr3(None)?;
            let op = match op_token.0 {
                EqualsEquals => BinaryOperator::Equals,
                BangEquals => BinaryOperator::NotEquals,
                Less => BinaryOperator::Less,
                Greater => BinaryOperator::Greater,
                LessEquals => BinaryOperator::LessOrEqual,
                GreaterEquals => BinaryOperator::GreaterOrEqual,
                _ => unreachable!(),
            };
            lhs = Rc::new(Expr(ExprType::Binary(op, lhs, rhs), op_token.1));
        }
        Ok(lhs)
    }

    fn parse_op_expr5(&mut self, start: Option<Rc<Expr>>) -> Result<Rc<Expr>, ParseError> {
        let mut lhs = self.parse_op_expr4(start)?;
        while let Some(op_token) = self.consume_if(|typ| matches!(typ, KeywordAnd))? {
            let rhs = self.parse_op_expr4(None)?;
            lhs = Rc::new(Expr(
                ExprType::Binary(BinaryOperator::And, lhs, rhs),
                op_token.1,
            ));
        }
        Ok(lhs)
    }

    fn parse_op_expr6(&mut self, start: Option<Rc<Expr>>) -> Result<Rc<Expr>, ParseError> {
        let mut lhs = self.parse_op_expr5(start)?;
        while let Some(op_token) = self.consume_if(|typ| matches!(typ, KeywordXor))? {
            let rhs = self.parse_op_expr5(None)?;
            lhs = Rc::new(Expr(
                ExprType::Binary(BinaryOperator::Xor, lhs, rhs),
                op_token.1,
            ));
        }
        Ok(lhs)
    }

    fn parse_op_expr7(&mut self, start: Option<Rc<Expr>>) -> Result<Rc<Expr>, ParseError> {
        let mut lhs = self.parse_op_expr6(start)?;
        while let Some(op_token) = self.consume_if(|typ| matches!(typ, KeywordOr))? {
            let rhs = self.parse_op_expr6(None)?;
            lhs = Rc::new(Expr(
                ExprType::Binary(BinaryOperator::Or, lhs, rhs),
                op_token.1,
            ));
        }
        Ok(lhs)
    }

    fn parse_op_expr_after(&mut self, start: Option<Rc<Expr>>) -> Result<Rc<Expr>, ParseError> {
        self.parse_op_expr7(start)
    }

    fn parse_op_expr(&mut self) -> Result<Rc<Expr>, ParseError> {
        self.parse_op_expr_after(None)
    }

    fn parse_expr(&mut self) -> Result<Rc<Expr>, ParseError> {
        if Self::is_operator(self.token_at(0)?) {
            return self.parse_op_expr();
        }
        let start = self.parse_simple_expr()?;
        if Self::is_operator(self.token_at(0)?) {
            self.parse_op_expr_after(Some(start))
        } else {
            let mut result = start;
            for arg in self.parse_args()? {
                let location = result.1.clone();
                result = Rc::new(Expr(ExprType::Send(result, arg), location));
            }
            Ok(result)
        }
    }

    fn parse_arg(&mut self) -> Result<Argument, ParseError> {
        let expr = self.parse_simple_expr()?;
        if let Some(ellipsis_token) = self.consume_if(|t| matches!(t, Ellipsis))? {
            match &expr.0 {
                ExprType::In => Ok(Argument {
                    typ: ArgumentType::InLoop,
                    expr: Rc::clone(&expr),
                    location: expr.1.clone(),
                }),
                ExprType::Receive(inner) => Ok(Argument {
                    typ: ArgumentType::ReceiveLoop,
                    expr: Rc::clone(inner),
                    location: expr.1.clone(),
                }),
                _ => self.err(
                    "an ellipsis is not allowed after this expression",
                    ellipsis_token.1,
                ),
            }
        } else {
            let location = expr.1.clone();
            Ok(Argument {
                typ: ArgumentType::Single,
                expr,
                location,
            })
        }
    }

    // true if we should stop reading arguments when encountering this token.
    pub fn is_argument_terminator(token: &Token) -> bool {
        matches!(
            token.0,
            RParen | RBracket | RBrace | Colon | Comma | Semicolon | Eof
        )
    }

    pub fn is_operator(token: &Token) -> bool {
        matches!(
            token.0,
            KeywordNot
                | Caret
                | Asterisk
                | Slash
                | Percent
                | Plus
                | Minus
                | EqualsEquals
                | BangEquals
                | Less
                | Greater
                | LessEquals
                | GreaterEquals
                | KeywordAnd
                | KeywordXor
                | KeywordOr
        )
    }

    fn parse_args(&mut self) -> Result<Vec<Argument>, ParseError> {
        let mut args = Vec::new();
        loop {
            let token = self.token_at(0)?;
            if Self::is_argument_terminator(token) {
                break;
            }
            if Self::is_operator(token) {
                return self.err_here("operators are not allowed in this context (hint: wrap the operator expression in parentheses)");
            }
            args.push(self.parse_arg()?);
        }
        Ok(args)
    }

    fn parse_assignment_target(&mut self) -> Result<AssignmentTarget, ParseError> {
        if let Some(target) = self.consume_if(|typ| matches!(typ, Identifier(_)))? {
            let Token(Identifier(name), location) = target else {
                unreachable!();
            };
            Ok(AssignmentTarget {
                typ: AssignmentType::Declaration,
                name,
                location,
            })
        } else if let Some(lparen) = self.consume_if(|typ| matches!(typ, LParen))? {
            let Some(name) = self.parse_ident()? else {
                return self.err_here("expected a variable name");
            };
            if self.consume_if(|typ| matches!(typ, RParen))?.is_none() {
                return self.err_here("expected a closing parenthesis");
            };
            Ok(AssignmentTarget {
                typ: AssignmentType::Assignment,
                name,
                location: lparen.1,
            })
        } else {
            self.err_here("expected a variable name")
        }
    }

    fn parse_assignment_operator(&mut self) -> Result<AssignmentType, ParseError> {
        if self.consume_if(|typ| matches!(typ, Equals))?.is_some() {
            Ok(AssignmentType::Assignment)
        } else if self.consume_if(|typ| matches!(typ, ColonEquals))?.is_some() {
            Ok(AssignmentType::Declaration)
        } else if self.consume_if(|typ| matches!(typ, RArrow))?.is_some() {
            Ok(AssignmentType::Constructor)
        } else if let Some(tok) = self.consume_if(|typ| {
            matches!(
                typ,
                CaretEquals
                    | AsteriskEquals
                    | SlashEquals
                    | PercentEquals
                    | PlusEquals
                    | MinusEquals
            )
        })? {
            let op = match tok.0 {
                CaretEquals => BinaryOperator::Power,
                AsteriskEquals => BinaryOperator::Multiply,
                SlashEquals => BinaryOperator::Divide,
                PercentEquals => BinaryOperator::Remainder,
                PlusEquals => BinaryOperator::Add,
                MinusEquals => BinaryOperator::Subtract,
                _ => unreachable!(),
            };
            Ok(AssignmentType::AugmentedAssignment(op))
        } else {
            self.err_here("expected an assignment operator")
        }
    }

    fn parse_comma_separated_exprs(
        &mut self,
        restrictions: ExprTypeRestrictions,
    ) -> Result<Vec<Rc<Expr>>, ParseError> {
        let mut exprs = Vec::new();
        loop {
            let expr = match restrictions {
                ExprTypeRestrictions::Any => self.parse_expr()?,
                ExprTypeRestrictions::NoSend => self.parse_op_expr()?,
            };
            exprs.push(expr);
            if self.consume_if(|typ| matches!(typ, Comma))?.is_none() {
                break;
            }
        }
        Ok(exprs)
    }

    fn parse_assignment_or_declaration(
        &mut self,
        value_restrictions: ExprTypeRestrictions,
    ) -> Result<Rc<Stmt>, ParseError> {
        let mut targets = Vec::new();
        loop {
            targets.push(self.parse_assignment_target()?);
            if self.consume_if(|typ| matches!(typ, Comma))?.is_none() {
                break;
            }
        }
        let location = self.token_at(0)?.1.clone();
        if self.consume_if(|t| matches!(t, Colon))?.is_some() {
            let mut vars = Vec::new();
            for target in targets {
                match target.typ {
                    AssignmentType::Declaration => {}
                    AssignmentType::Assignment => {
                        return self.err(
                            "declared variable name may not be parenthesized",
                            target.location.clone(),
                        );
                    }
                    AssignmentType::Constructor | AssignmentType::AugmentedAssignment(_) => {
                        unreachable!()
                    }
                }
                vars.push((target.name, target.location));
            }
            return Ok(Rc::new(Stmt(StmtType::Declaration(vars), location)));
        }
        let assignment_type = self.parse_assignment_operator()?;
        if matches!(
            assignment_type,
            AssignmentType::Constructor | AssignmentType::AugmentedAssignment(_)
        ) && targets.len() > 1
        {
            return self.err(
                "multiple assignment targets are not allowed here",
                targets[1].location.clone(),
            );
        }
        if matches!(
            assignment_type,
            AssignmentType::Assignment
                | AssignmentType::Constructor
                | AssignmentType::AugmentedAssignment(_)
        ) && let Some(bad_target) = targets
            .iter()
            .find(|target| target.typ == AssignmentType::Assignment)
        {
            let message = if assignment_type == AssignmentType::Constructor {
                "parentheses are not allowed around a constructor name"
            } else {
                "redundant parentheses in an assignment statement are forbidden"
            };
            return self.err(message, bad_target.location.clone());
        }
        if matches!(
            assignment_type,
            AssignmentType::Assignment
                | AssignmentType::AugmentedAssignment(_)
                | AssignmentType::Constructor
        ) {
            for target in &mut targets {
                target.typ = assignment_type;
            }
        }
        let values = self.parse_comma_separated_exprs(value_restrictions)?;
        let targets = targets.into_iter().map(Rc::new).collect::<Vec<_>>();
        Ok(Rc::new(Stmt(
            StmtType::Assignment(Rc::new(Assignment {
                targets,
                values,
                location: location.clone(),
            })),
            location,
        )))
    }

    fn is_before_assignment_or_declaration(&mut self) -> Result<bool, ParseError> {
        let index = if matches!(self.token_at(0)?.0, Identifier(_)) {
            1
        } else if matches!(self.token_at(0)?.0, LParen)
            && matches!(self.token_at(1)?.0, Identifier(_))
            && matches!(self.token_at(2)?.0, RParen)
        {
            3
        } else {
            return Ok(false);
        };
        Ok(matches!(
            self.token_at(index)?.0,
            Comma
                | Equals
                | ColonEquals
                | RArrow
                | CaretEquals
                | AsteriskEquals
                | SlashEquals
                | PercentEquals
                | PlusEquals
                | MinusEquals
                | Colon
        ))
    }

    fn skip_redundant_semicolon(&mut self) -> Result<(), ParseError> {
        while matches!(self.token_at(0)?.0, Semicolon) && matches!(self.token_at(1)?.0, Semicolon) {
            self.skip(1);
        }
        Ok(())
    }

    fn parse_condition(&mut self) -> Result<Condition, ParseError> {
        if !self.is_before_assignment_or_declaration()? {
            return Ok(Condition::Boolean(self.parse_op_expr()?));
        }
        let stmt = self.parse_assignment_or_declaration(ExprTypeRestrictions::NoSend)?;
        let StmtType::Assignment(assignment) = &stmt.0 else {
            return self.err("unexpected colon in a condition", stmt.1.clone());
        };
        let location = assignment.location.clone();
        for target in &assignment.targets {
            match target.typ {
                AssignmentType::Assignment | AssignmentType::Declaration => {}
                AssignmentType::AugmentedAssignment(_) => {
                    return self.err(
                        "augmented assignment is not allowed in a condition",
                        location,
                    );
                }
                AssignmentType::Constructor => {
                    return self.err("cannot declare a constructor in a condition", location);
                }
            }
        }
        for value in &assignment.values {
            if !matches!(value.0, ExprType::In | ExprType::Receive(_)) {
                return self.err(
                    "assigned value must be an `in` expression or a receive expression",
                    assignment.location.clone(),
                );
            }
        }
        Ok(Condition::Assignment(stmt))
    }

    fn parse_after_if(&mut self, location: Location) -> Result<Rc<Stmt>, ParseError> {
        let cond = self.parse_condition()?;
        self.ignore_semicolons()?;
        let then = self.parse_block()?.stmts;
        self.skip_redundant_semicolon()?;
        let has_else = if self.consume_if(|typ| matches!(typ, KeywordElse))?.is_some() {
            true
        } else if matches!(self.token_at(0)?.0, Semicolon)
            && matches!(self.token_at(1)?.0, KeywordElse)
        {
            self.skip(2);
            true
        } else {
            false
        };
        let else_ =
            if has_else && let Some(if_token) = self.consume_if(|typ| matches!(typ, KeywordIf))? {
                let stmt = self.parse_after_if(if_token.1)?;
                vec![stmt]
            } else if has_else {
                self.parse_block()?.stmts
            } else {
                Vec::new()
            };
        Ok(Rc::new(Stmt(StmtType::If(cond, then, else_), location)))
    }

    fn parse_case_body(&mut self) -> Result<Vec<Rc<Stmt>>, ParseError> {
        let mut result = Vec::new();
        loop {
            self.ignore_semicolons()?;
            if matches!(self.token_at(0)?.0, KeywordCase | KeywordDefault | RBrace) {
                break;
            }
            result.push(self.parse_stmt()?);
            if self.consume_if(|t| matches!(t, Semicolon))?.is_none()
                && !matches!(self.token_at(0)?.0, RBrace)
            {
                return self.err_here("expected a newline or a semicolon after a statement");
            }
        }
        Ok(result)
    }

    fn parse_case(&mut self) -> Result<SwitchCase, ParseError> {
        if let Some(case_token) = self.consume_if(|t| matches!(t, KeywordCase))? {
            let values = self.parse_comma_separated_exprs(ExprTypeRestrictions::Any)?;
            if self.consume_if(|t| matches!(t, Colon))?.is_none() {
                return self.err_here("expected a colon after `case`");
            }
            let body = self.parse_case_body()?;
            Ok(SwitchCase {
                values: Some(values),
                body,
                location: case_token.1,
            })
        } else if let Some(default_token) = self.consume_if(|t| matches!(t, KeywordDefault))? {
            if self.consume_if(|t| matches!(t, Colon))?.is_none() {
                return self.err_here("expected a colon after `default`");
            }
            let body = self.parse_case_body()?;
            Ok(SwitchCase {
                values: None,
                body,
                location: default_token.1,
            })
        } else {
            self.err_here("expected `case` or `default`")
        }
    }

    fn parse_stmt(&mut self) -> Result<Rc<Stmt>, ParseError> {
        if let Some(out_token) = self.consume_if(|t| matches!(t, KeywordOut))? {
            let args = self.parse_args()?;
            if matches!(self.token_at(0)?.0, Comma) {
                return self.err_here(
                    "unnecessary comma in `out` statement (outputs are separated with spaces)",
                );
            }
            Ok(Rc::new(Stmt(StmtType::Out(args), out_token.1)))
        } else if let Some(raise_token) =
            self.consume_if(|t| matches!(t, KeywordErr | KeywordThrow))?
        {
            let kind = match raise_token.0 {
                TokenType::KeywordErr => RaiseType::Err,
                TokenType::KeywordThrow => RaiseType::Throw,
                _ => unreachable!(),
            };
            let args = self.parse_args()?;
            if matches!(self.token_at(0)?.0, Comma) {
                return self.err_here(
                    "unnecessary comma in error (error values are separated with spaces)",
                );
            }
            Ok(Rc::new(Stmt(StmtType::Raise(kind, args), raise_token.1)))
        } else if let Some(break_token) = self.consume_if(|typ| matches!(typ, KeywordBreak))? {
            Ok(Rc::new(Stmt(
                StmtType::Jump(JumpType::Break),
                break_token.1,
            )))
        } else if let Some(continue_token) =
            self.consume_if(|typ| matches!(typ, KeywordContinue))?
        {
            Ok(Rc::new(Stmt(
                StmtType::Jump(JumpType::Continue),
                continue_token.1,
            )))
        } else if let Some(stop_token) = self.consume_if(|typ| matches!(typ, KeywordStop))? {
            Ok(Rc::new(Stmt(StmtType::Jump(JumpType::Stop), stop_token.1)))
        } else if let Some(debug_token) =
            self.consume_if(|typ| matches!(typ, InternalKeywordDebug))?
        {
            let expr = self.parse_simple_expr()?;
            Ok(Rc::new(Stmt(StmtType::Debug(expr), debug_token.1)))
        } else if let Some(if_token) = self.consume_if(|typ| matches!(typ, KeywordIf))? {
            self.parse_after_if(if_token.1)
        } else if let Some(switch_token) = self.consume_if(|t| matches!(t, KeywordSwitch))? {
            let expr = self.parse_op_expr()?;
            if self.consume_if(|t| matches!(t, LBrace))?.is_none() {
                return self.err_here("expected a block");
            };
            let mut cases = Vec::new();
            loop {
                self.ignore_semicolons()?;
                if self.consume_if(|t| matches!(t, RBrace))?.is_some() {
                    break;
                }
                cases.push(self.parse_case()?);
            }
            if cases.is_empty() {
                return self.err(
                    "a `switch` statement must have at least one case (or `default`)",
                    switch_token.1,
                );
            }
            for case in &cases[..cases.len() - 1] {
                if case.values.is_none() {
                    return self.err(
                        "`default` must be the last case in a `switch` statement",
                        case.location.clone(),
                    );
                }
            }
            Ok(Rc::new(Stmt(StmtType::Switch(expr, cases), switch_token.1)))
        } else if let Some(loop_token) = self.consume_if(|typ| matches!(typ, KeywordLoop))? {
            let stmts = self.parse_block()?.stmts;
            Ok(Rc::new(Stmt(StmtType::Loop(None, stmts), loop_token.1)))
        } else if let Some(while_token) = self.consume_if(|typ| matches!(typ, KeywordWhile))? {
            let cond = self.parse_condition()?;
            let stmts = self.parse_block()?.stmts;
            Ok(Rc::new(Stmt(
                StmtType::Loop(Some(cond), stmts),
                while_token.1,
            )))
        } else if let Some(try_token) = self.consume_if(|typ| matches!(typ, KeywordTry))? {
            if self.consume_if(|t| matches!(t, LBrace))?.is_none() {
                return self.err_here("expected a block");
            };
            let body = self.parse_case_body()?;
            let mut cases = Vec::new();
            loop {
                self.ignore_semicolons()?;
                if self.consume_if(|t| matches!(t, RBrace))?.is_some() {
                    break;
                }
                cases.push(self.parse_case()?);
            }
            if cases.is_empty() {
                return self.err(
                    "a `try` statement must have at least one case (or `default`)",
                    try_token.1,
                );
            }
            for case in &cases[..cases.len() - 1] {
                if case.values.is_none() {
                    return self.err(
                        "`default` must be the last case in a `try` statement",
                        case.location.clone(),
                    );
                }
            }
            Ok(Rc::new(Stmt(
                StmtType::Try(TryBlock { body, cases }),
                try_token.1,
            )))
        } else if self.is_before_assignment_or_declaration()? {
            let init_stmt = self.parse_assignment_or_declaration(ExprTypeRestrictions::Any)?;
            let assignment = match &init_stmt.0 {
                StmtType::Assignment(assignment) => Rc::clone(assignment),
                StmtType::Declaration(_) => return Ok(init_stmt),
                _ => unreachable!(),
            };
            let location = assignment.location.clone();
            self.skip_redundant_semicolon()?;
            if matches!(self.token_at(0)?.0, Semicolon)
                && matches!(self.token_at(1)?.0, KeywordElse)
            {
                let else_location = self.token_at(0)?.1.clone();
                self.skip(2);
                let else_ = self.parse_block()?.stmts;
                for target in &assignment.targets {
                    if target.typ == AssignmentType::Constructor {
                        return self.err(
                            "an `else` clause is not allowed after a constructor declaration",
                            else_location,
                        );
                    }
                    if matches!(target.typ, AssignmentType::AugmentedAssignment(_)) {
                        return self.err(
                            "an `else` clause is not allowed after an augmented assignment statement",
                            else_location
                        );
                    }
                }
                Ok(Rc::new(Stmt(
                    StmtType::AssignmentElse(assignment, else_),
                    location,
                )))
            } else {
                Ok(Rc::new(Stmt(StmtType::Assignment(assignment), location)))
            }
        } else {
            let location = self.token_at(0)?.1.clone();
            let expr = self.parse_expr()?;
            if let ExprType::ProcessLiteral(stmts) = &expr.0 {
                return Ok(Rc::new(Stmt(StmtType::BareBlock(stmts.to_vec()), expr.1.clone())));
            }
            Ok(Rc::new(Stmt(StmtType::Expr(expr), location)))
        }
    }

    fn ignore_semicolons(&mut self) -> Result<(), ParseError> {
        while self.consume_if(|t| matches!(t, Semicolon))?.is_some() {}
        Ok(())
    }

    fn parse_block(&mut self) -> Result<Block, ParseError> {
        let Some(Token(_, location)) = self.consume_if(|t| matches!(t, LBrace))? else {
            return self.err_here("expected a block");
        };
        let mut stmts = Vec::new();
        loop {
            self.ignore_semicolons()?;
            if self.consume_if(|t| matches!(t, RBrace))?.is_some() {
                break;
            }
            stmts.push(self.parse_stmt()?);
            if self.consume_if(|t| matches!(t, Semicolon))?.is_none()
                && !matches!(self.token_at(0)?.0, RBrace)
            {
                return self.err_here("expected a newline or a semicolon after a statement");
            }
        }
        Ok(Block { stmts, location })
    }

    fn parse_import_renaming(&mut self) -> Result<Option<String>, ParseError> {
        if self.consume_if(|t| matches!(t, KeywordAs))?.is_none() {
            return Ok(None);
        }
        let Some(ident) = self.parse_ident()? else {
            return self.err_here("expected an identifier after `as`");
        };
        Ok(Some(ident))
    }

    fn parse_import(&mut self) -> Result<Rc<ImportDeclaration>, ParseError> {
        let Some(Token(_, location)) = self.consume_if(|t| matches!(t, KeywordImport))? else {
            return self.err_here("expected keyword `import`");
        };
        let (module, mut items) = {
            let mut path = self.parse_path()?;
            let item = path.pop().expect("path after import shouldn't be empty");
            let rename = self.parse_import_renaming()?;
            (path, vec![(item, rename)])
        };
        while self.consume_if(|t| matches!(t, Comma))?.is_some() {
            let Some(ident) = self.parse_ident()? else {
                return self.err_here("expected an identifier after comma");
            };
            let rename = self.parse_import_renaming()?;
            items.push((ident, rename));
        }
        if module.is_empty() && items.len() > 1 {
            return self.err(
                "cannot import multiple modules in one `import` statement",
                location,
            );
        }
        Ok(Rc::new(ImportDeclaration {
            module,
            items,
            location,
        }))
    }

    fn parse_global_assignment(&mut self) -> Result<Rc<Assignment>, ParseError> {
        let stmt = self.parse_stmt()?;
        let assignment = match &stmt.0 {
            StmtType::Assignment(assignment) => Rc::clone(assignment),
            _ => {
                return self.err(
                    "the top level of the program may only contain declarations",
                    stmt.1.clone(),
                );
            }
        };
        for target in &assignment.targets {
            if matches!(
                target.typ,
                AssignmentType::Assignment | AssignmentType::AugmentedAssignment(_)
            ) {
                return self.err(
                    "assignment statements are only allowed within a process",
                    target.location.clone(),
                );
            }
        }
        Ok(assignment)
    }

    pub fn parse_all(&mut self) -> Result<Rc<CodeFile>, ParseError> {
        let mut declarations = Vec::new();
        loop {
            self.ignore_semicolons()?;
            if self.consume_if(|t| matches!(t, Eof))?.is_some() {
                break;
            }
            if matches!(self.token_at(0)?, Token(KeywordImport, _)) {
                declarations.push(GlobalDeclaration::Import(self.parse_import()?));
            } else {
                declarations.push(GlobalDeclaration::Assignment(
                    self.parse_global_assignment()?,
                ));
            }
            if self.consume_if(|t| matches!(t, Semicolon))?.is_none()
                && !matches!(self.token_at(0)?.0, Eof)
            {
                return self
                    .err_here("expected a newline or a semicolon after a global declaration");
            }
        }
        Ok(Rc::new(CodeFile { declarations }))
    }
}
