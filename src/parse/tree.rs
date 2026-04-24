use std::rc::Rc;

use malachite::Integer;

use crate::parse::location::Location;

#[derive(Debug)]
pub enum ConstantLiteral {
    Integer(Integer),
    Float(f64),
    String(Vec<u8>),
    Symbol(String),
}

#[derive(Clone, Copy, Debug)]
pub enum UnaryOperator {
    Plus,
    Minus,
    Not,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOperator {
    Power,
    Multiply,
    Divide,
    Remainder,
    Add,
    Subtract,
    Equals,
    NotEquals,
    Less,
    Greater,
    LessOrEqual,
    GreaterOrEqual,
    And,
    Xor,
    Or,
}

#[derive(Clone, Copy, Debug)]
pub enum ArgumentType {
    Single,
    InLoop,
    ReceiveLoop,
}

#[derive(Debug)]
pub struct Argument {
    pub typ: ArgumentType,
    pub expr: Rc<Expr>,
    pub location: Location,
}

#[derive(Debug)]
pub enum InputType {
    Normal,
    Fork,
}

#[derive(Debug)]
pub enum ExprType {
    ConstantLiteral(ConstantLiteral),
    ProcessLiteral(Vec<Rc<Stmt>>),
    Identifier(String),
    Path(Vec<String>),
    In(InputType),
    Parenthesized(Rc<Expr>),
    Unary(UnaryOperator, Rc<Expr>),
    Binary(BinaryOperator, Rc<Expr>, Rc<Expr>),
    Send(Rc<Expr>, Argument),
    Receive(Rc<Expr>),
}

#[derive(Debug)]
pub struct Expr(pub ExprType, pub Location);

#[derive(Clone, Copy, Debug)]
pub enum JumpType {
    Stop,
    Break,
    Continue,
}

#[derive(Debug)]
pub enum Condition {
    Boolean(Rc<Expr>),
    Assignment(Rc<Stmt>),
}

#[derive(Debug)]
pub struct SwitchCase {
    pub values: Option<Vec<Rc<Expr>>>,
    pub body: Vec<Rc<Stmt>>,
    pub location: Location,
}

#[derive(Debug)]
pub enum RaiseType {
    Err,
    Throw,
}

#[derive(Debug)]
pub struct TryBlock {
    pub body: Vec<Rc<Stmt>>,
    pub cases: Vec<SwitchCase>,
}

#[derive(Debug)]
pub enum StmtType {
    Expr(Rc<Expr>),
    Declaration(Vec<Rc<AssignmentTarget>>),
    Assignment(Rc<Assignment>),
    AssignmentElse(Rc<Assignment>, Vec<Rc<Stmt>>),
    Debug(Rc<Expr>),
    Out(Vec<Argument>),
    Raise(RaiseType, Vec<Argument>),
    Jump(JumpType),
    If(Condition, Vec<Rc<Stmt>>, Vec<Rc<Stmt>>),
    Switch(Rc<Expr>, Vec<SwitchCase>),
    Loop(Option<Condition>, Vec<Rc<Stmt>>),
    Try(TryBlock),
    BareBlock(Vec<Rc<Stmt>>),
}

#[derive(Debug)]
pub struct Stmt(pub StmtType, pub Location);

#[derive(Debug)]
pub struct Block {
    pub stmts: Vec<Rc<Stmt>>,
    pub location: Location,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssignmentType {
    Declaration,
    Assignment,
    AugmentedAssignment(BinaryOperator),
    Constructor,
    Discard,
}

#[derive(Debug)]
pub struct AssignmentTarget {
    pub typ: AssignmentType,
    pub name: String,
    pub location: Location,
}

#[derive(Debug)]
pub struct Assignment {
    pub targets: Vec<Rc<AssignmentTarget>>,
    pub values: Vec<Rc<Expr>>,
    pub location: Location,
}

#[derive(Debug)]
pub struct ImportDeclaration {
    pub module: Vec<String>,
    pub items: Vec<(String, Option<String>)>, // Optional renaming.
    pub location: Location,
}

#[derive(Clone, Debug)]
pub enum GlobalDeclaration {
    Assignment(Rc<Assignment>),
    Import(Rc<ImportDeclaration>),
}

#[derive(Debug)]
pub struct CodeFile {
    pub declarations: Vec<GlobalDeclaration>,
}

pub trait CodeVisitor {
    type Output;

    fn visit_constant_literal(&mut self, expr: &Rc<Expr>) -> Self::Output;
    fn visit_process_literal(&mut self, expr: &Rc<Expr>) -> Self::Output;
    fn visit_identifier(&mut self, expr: &Rc<Expr>) -> Self::Output;
    fn visit_path(&mut self, expr: &Rc<Expr>) -> Self::Output;
    fn visit_in(&mut self, expr: &Rc<Expr>) -> Self::Output;
    fn visit_parenthesized(&mut self, expr: &Rc<Expr>) -> Self::Output;
    fn visit_unary(&mut self, expr: &Rc<Expr>) -> Self::Output;
    fn visit_binary(&mut self, expr: &Rc<Expr>) -> Self::Output;
    fn visit_send(&mut self, expr: &Rc<Expr>) -> Self::Output;
    fn visit_receive(&mut self, expr: &Rc<Expr>) -> Self::Output;
    fn visit_assignment_target(&mut self, target: &Rc<AssignmentTarget>) -> Self::Output;
    fn visit_expr_stmt(&mut self, stmt: &Rc<Stmt>) -> Self::Output;
    fn visit_declaration(&mut self, stmt: &Rc<Stmt>) -> Self::Output;
    fn visit_assignment_stmt(&mut self, stmt: &Rc<Stmt>) -> Self::Output;
    fn visit_assignment_else(&mut self, stmt: &Rc<Stmt>) -> Self::Output;
    fn visit_out(&mut self, stmt: &Rc<Stmt>) -> Self::Output;
    fn visit_raise(&mut self, stmt: &Rc<Stmt>) -> Self::Output;
    fn visit_jump(&mut self, stmt: &Rc<Stmt>) -> Self::Output;
    fn visit_debug(&mut self, stmt: &Rc<Stmt>) -> Self::Output;
    fn visit_switch(&mut self, stmt: &Rc<Stmt>) -> Self::Output;
    fn visit_if(&mut self, stmt: &Rc<Stmt>) -> Self::Output;
    fn visit_loop(&mut self, stmt: &Rc<Stmt>) -> Self::Output;
    fn visit_try(&mut self, stmt: &Rc<Stmt>) -> Self::Output;
    fn visit_bare_block(&mut self, stmt: &Rc<Stmt>) -> Self::Output;
    fn visit_global_assignment(&mut self, decl: &Rc<Assignment>) -> Self::Output;
    fn visit_import_declaration(&mut self, decl: &Rc<ImportDeclaration>) -> Self::Output;
    fn visit_code_file(&mut self, file: &Rc<CodeFile>) -> Self::Output;

    fn visit<C: VisitableCode>(&mut self, code: &C) -> Self::Output
    where
        Self: Sized,
    {
        code.accept_visitor(self)
    }
}

pub trait ResultCodeVisitor<E>
where
    Self: Sized + CodeVisitor<Output = Result<(), E>>,
{
    #[must_use]
    fn visit_many<C: VisitableCode>(&mut self, codes: &[C]) -> Self::Output {
        for code in codes {
            self.visit(code)?;
        }
        Ok(())
    }
}

impl<T, E> ResultCodeVisitor<E> for T where T: CodeVisitor<Output = Result<(), E>> {}

pub trait VisitableCode {
    fn accept_visitor<V: CodeVisitor>(&self, visitor: &mut V) -> V::Output;
}

impl VisitableCode for Rc<Expr> {
    fn accept_visitor<V: CodeVisitor>(&self, visitor: &mut V) -> V::Output {
        match &self.0 {
            ExprType::ConstantLiteral(..) => visitor.visit_constant_literal(self),
            ExprType::ProcessLiteral(..) => visitor.visit_process_literal(self),
            ExprType::Identifier(..) => visitor.visit_identifier(self),
            ExprType::Path(..) => visitor.visit_path(self),
            ExprType::In(..) => visitor.visit_in(self),
            ExprType::Parenthesized(..) => visitor.visit_parenthesized(self),
            ExprType::Unary(..) => visitor.visit_unary(self),
            ExprType::Binary(..) => visitor.visit_binary(self),
            ExprType::Send(..) => visitor.visit_send(self),
            ExprType::Receive(..) => visitor.visit_receive(self),
        }
    }
}

impl VisitableCode for Rc<AssignmentTarget> {
    fn accept_visitor<V: CodeVisitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_assignment_target(self)
    }
}

impl VisitableCode for Condition {
    fn accept_visitor<V: CodeVisitor>(&self, visitor: &mut V) -> V::Output {
        match self {
            Condition::Boolean(expr) => visitor.visit(expr),
            Condition::Assignment(stmt) => visitor.visit(stmt),
        }
    }
}

impl VisitableCode for Rc<Stmt> {
    fn accept_visitor<V: CodeVisitor>(&self, visitor: &mut V) -> V::Output {
        match &self.0 {
            StmtType::Expr(..) => visitor.visit_expr_stmt(self),
            StmtType::Declaration(..) => visitor.visit_declaration(self),
            StmtType::Assignment(..) => visitor.visit_assignment_stmt(self),
            StmtType::AssignmentElse(..) => visitor.visit_assignment_else(self),
            StmtType::Out(..) => visitor.visit_out(self),
            StmtType::Raise(..) => visitor.visit_raise(self),
            StmtType::Jump(..) => visitor.visit_jump(self),
            StmtType::Debug(..) => visitor.visit_debug(self),
            StmtType::If(..) => visitor.visit_if(self),
            StmtType::Switch(..) => visitor.visit_switch(self),
            StmtType::Loop(..) => visitor.visit_loop(self),
            StmtType::Try(..) => visitor.visit_try(self),
            StmtType::BareBlock(..) => visitor.visit_bare_block(self),
        }
    }
}

impl VisitableCode for Argument {
    fn accept_visitor<V: CodeVisitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit(&self.expr)
    }
}

impl VisitableCode for Rc<CodeFile> {
    fn accept_visitor<V: CodeVisitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_code_file(self)
    }
}

impl VisitableCode for GlobalDeclaration {
    fn accept_visitor<V: CodeVisitor>(&self, visitor: &mut V) -> V::Output {
        match self {
            GlobalDeclaration::Assignment(decl) => visitor.visit_global_assignment(decl),
            GlobalDeclaration::Import(decl) => visitor.visit_import_declaration(decl),
        }
    }
}

pub trait DefaultCodeVisitor: Sized {
    type Error;

    fn visit_constant_literal(&mut self, expr: &Rc<Expr>) -> Result<(), Self::Error> {
        let _ = expr;
        Ok(())
    }

    fn visit_process_literal(&mut self, expr: &Rc<Expr>) -> Result<(), Self::Error> {
        let Expr(ExprType::ProcessLiteral(stmts), _) = &**expr else {
            panic!("visitor method got wrong variant")
        };
        self.visit_many(stmts)
    }

    fn visit_identifier(&mut self, expr: &Rc<Expr>) -> Result<(), Self::Error> {
        let _ = expr;
        Ok(())
    }

    fn visit_path(&mut self, expr: &Rc<Expr>) -> Result<(), Self::Error> {
        let _ = expr;
        Ok(())
    }

    fn visit_in(&mut self, expr: &Rc<Expr>) -> Result<(), Self::Error> {
        let _ = expr;
        Ok(())
    }

    fn visit_parenthesized(&mut self, expr: &Rc<Expr>) -> Result<(), Self::Error> {
        let Expr(ExprType::Parenthesized(inner), _) = &**expr else {
            panic!("visitor method got wrong variant");
        };
        self.visit(inner)
    }

    fn visit_unary(&mut self, expr: &Rc<Expr>) -> Result<(), Self::Error> {
        let Expr(ExprType::Unary(_, rhs), _) = &**expr else {
            panic!("visitor method got wrong variant");
        };
        self.visit(rhs)
    }

    fn visit_binary(&mut self, expr: &Rc<Expr>) -> Result<(), Self::Error> {
        let Expr(ExprType::Binary(_, lhs, rhs), _) = &**expr else {
            panic!("visitor method got wrong variant");
        };
        self.visit(lhs)?;
        self.visit(rhs)
    }

    fn visit_send(&mut self, expr: &Rc<Expr>) -> Result<(), Self::Error> {
        let Expr(ExprType::Send(proc, arg), _) = &**expr else {
            panic!("visitor method got wrong variant");
        };
        self.visit(proc)?;
        self.visit(&arg.expr)
    }

    fn visit_receive(&mut self, expr: &Rc<Expr>) -> Result<(), Self::Error> {
        let Expr(ExprType::Receive(inner), _) = &**expr else {
            panic!("visitor method got wrong variant");
        };
        self.visit(inner)
    }

    fn visit_assignment_target(
        &mut self,
        target: &Rc<AssignmentTarget>,
    ) -> Result<(), Self::Error> {
        let _ = target;
        Ok(())
    }

    fn visit_expr_stmt(&mut self, stmt: &Rc<Stmt>) -> Result<(), Self::Error> {
        let Stmt(StmtType::Expr(expr), _) = &**stmt else {
            panic!("visitor method got wrong variant");
        };
        self.visit(expr)
    }

    fn visit_declaration(&mut self, stmt: &Rc<Stmt>) -> Result<(), Self::Error> {
        let Stmt(StmtType::Declaration(targets), _) = &**stmt else {
            panic!("visitor method got wrong variant");
        };
        self.visit_many(targets)
    }

    fn visit_assignment_stmt(&mut self, stmt: &Rc<Stmt>) -> Result<(), Self::Error> {
        let Stmt(StmtType::Assignment(assignment), _) = &**stmt else {
            panic!("visitor method got wrong variant");
        };
        self.visit_many(&assignment.targets)?;
        self.visit_many(&assignment.values)
    }

    fn visit_assignment_else(&mut self, stmt: &Rc<Stmt>) -> Result<(), Self::Error> {
        let Stmt(StmtType::AssignmentElse(assignment, else_), _) = &**stmt else {
            panic!("visitor method got wrong variant");
        };
        self.visit_many(&assignment.targets)?;
        self.visit_many(&assignment.values)?;
        self.visit_many(else_)
    }

    fn visit_out(&mut self, stmt: &Rc<Stmt>) -> Result<(), Self::Error> {
        let Stmt(StmtType::Out(args), _) = &**stmt else {
            panic!("visitor method got wrong variant");
        };
        self.visit_many(args)
    }

    fn visit_raise(&mut self, stmt: &Rc<Stmt>) -> Result<(), Self::Error> {
        let Stmt(StmtType::Raise(_, args), _) = &**stmt else {
            panic!("visitor method got wrong variant");
        };
        self.visit_many(args)
    }

    fn visit_jump(&mut self, stmt: &Rc<Stmt>) -> Result<(), Self::Error> {
        let _ = stmt;
        Ok(())
    }

    fn visit_debug(&mut self, stmt: &Rc<Stmt>) -> Result<(), Self::Error> {
        let Stmt(StmtType::Debug(expr), _) = &**stmt else {
            panic!("visitor method got wrong variant");
        };
        self.visit(expr)
    }

    fn visit_if(&mut self, stmt: &Rc<Stmt>) -> Result<(), Self::Error> {
        let Stmt(StmtType::If(cond, then, else_), _) = &**stmt else {
            panic!("visitor method got wrong variant");
        };
        self.visit(cond)?;
        self.visit_many(then)?;
        self.visit_many(else_)
    }

    fn visit_switch(&mut self, stmt: &Rc<Stmt>) -> Result<(), Self::Error> {
        let Stmt(StmtType::Switch(expr, cases), _) = &**stmt else {
            panic!("visitor method got wrong variant");
        };
        self.visit(expr)?;
        for case in cases {
            if let Some(values) = &case.values {
                self.visit_many(values)?;
            }
            self.visit_many(&case.body)?;
        }
        Ok(())
    }

    fn visit_loop(&mut self, stmt: &Rc<Stmt>) -> Result<(), Self::Error> {
        let Stmt(StmtType::Loop(cond, stmts), _) = &**stmt else {
            panic!("visitor method got wrong variant");
        };
        if let Some(cond) = cond {
            self.visit(cond)?;
        }
        self.visit_many(stmts)
    }

    fn visit_try(&mut self, stmt: &Rc<Stmt>) -> Result<(), Self::Error> {
        let Stmt(StmtType::Try(TryBlock { body, cases }), _) = &**stmt else {
            panic!("visitor method got wrong variant");
        };
        self.visit_many(body)?;
        for case in cases {
            if let Some(values) = &case.values {
                self.visit_many(values)?;
            }
            self.visit_many(&case.body)?;
        }
        Ok(())
    }

    fn visit_bare_block(&mut self, stmt: &Rc<Stmt>) -> Result<(), Self::Error> {
        let Stmt(StmtType::BareBlock(stmts), _) = &**stmt else {
            panic!("visitor method got wrong variant");
        };
        self.visit_many(stmts)
    }

    fn visit_global_assignment(&mut self, decl: &Rc<Assignment>) -> Result<(), Self::Error> {
        self.visit_many(&decl.values)
    }

    fn visit_import_declaration(
        &mut self,
        import: &Rc<ImportDeclaration>,
    ) -> Result<(), Self::Error> {
        let _ = import;
        Ok(())
    }

    fn visit_code_file(&mut self, file: &Rc<CodeFile>) -> Result<(), Self::Error> {
        for decl in &file.declarations {
            match decl {
                GlobalDeclaration::Assignment(assignment) => {
                    self.visit_global_assignment(assignment)?
                }
                GlobalDeclaration::Import(import) => self.visit_import_declaration(import)?,
            }
        }
        Ok(())
    }
}

impl<U: DefaultCodeVisitor> CodeVisitor for U {
    type Output = Result<(), U::Error>;

    fn visit_constant_literal(&mut self, expr: &Rc<Expr>) -> Self::Output {
        DefaultCodeVisitor::visit_constant_literal(self, expr)
    }

    fn visit_process_literal(&mut self, expr: &Rc<Expr>) -> Self::Output {
        DefaultCodeVisitor::visit_process_literal(self, expr)
    }

    fn visit_identifier(&mut self, expr: &Rc<Expr>) -> Self::Output {
        DefaultCodeVisitor::visit_identifier(self, expr)
    }

    fn visit_path(&mut self, expr: &Rc<Expr>) -> Self::Output {
        DefaultCodeVisitor::visit_path(self, expr)
    }

    fn visit_in(&mut self, expr: &Rc<Expr>) -> Self::Output {
        DefaultCodeVisitor::visit_in(self, expr)
    }

    fn visit_parenthesized(&mut self, expr: &Rc<Expr>) -> Self::Output {
        DefaultCodeVisitor::visit_parenthesized(self, expr)
    }

    fn visit_unary(&mut self, expr: &Rc<Expr>) -> Self::Output {
        DefaultCodeVisitor::visit_unary(self, expr)
    }

    fn visit_binary(&mut self, expr: &Rc<Expr>) -> Self::Output {
        DefaultCodeVisitor::visit_binary(self, expr)
    }

    fn visit_send(&mut self, expr: &Rc<Expr>) -> Self::Output {
        DefaultCodeVisitor::visit_send(self, expr)
    }

    fn visit_receive(&mut self, expr: &Rc<Expr>) -> Self::Output {
        DefaultCodeVisitor::visit_receive(self, expr)
    }

    fn visit_assignment_target(&mut self, target: &Rc<AssignmentTarget>) -> Self::Output {
        DefaultCodeVisitor::visit_assignment_target(self, target)
    }

    fn visit_expr_stmt(&mut self, stmt: &Rc<Stmt>) -> Self::Output {
        DefaultCodeVisitor::visit_expr_stmt(self, stmt)
    }

    fn visit_declaration(&mut self, stmt: &Rc<Stmt>) -> Self::Output {
        DefaultCodeVisitor::visit_declaration(self, stmt)
    }

    fn visit_assignment_stmt(&mut self, stmt: &Rc<Stmt>) -> Self::Output {
        DefaultCodeVisitor::visit_assignment_stmt(self, stmt)
    }

    fn visit_assignment_else(&mut self, stmt: &Rc<Stmt>) -> Self::Output {
        DefaultCodeVisitor::visit_assignment_else(self, stmt)
    }

    fn visit_out(&mut self, stmt: &Rc<Stmt>) -> Self::Output {
        DefaultCodeVisitor::visit_out(self, stmt)
    }

    fn visit_raise(&mut self, stmt: &Rc<Stmt>) -> Self::Output {
        DefaultCodeVisitor::visit_raise(self, stmt)
    }

    fn visit_jump(&mut self, stmt: &Rc<Stmt>) -> Self::Output {
        DefaultCodeVisitor::visit_jump(self, stmt)
    }

    fn visit_debug(&mut self, stmt: &Rc<Stmt>) -> Self::Output {
        DefaultCodeVisitor::visit_debug(self, stmt)
    }

    fn visit_if(&mut self, stmt: &Rc<Stmt>) -> Self::Output {
        DefaultCodeVisitor::visit_if(self, stmt)
    }

    fn visit_switch(&mut self, stmt: &Rc<Stmt>) -> Self::Output {
        DefaultCodeVisitor::visit_switch(self, stmt)
    }

    fn visit_loop(&mut self, stmt: &Rc<Stmt>) -> Self::Output {
        DefaultCodeVisitor::visit_loop(self, stmt)
    }

    fn visit_try(&mut self, stmt: &Rc<Stmt>) -> Self::Output {
        DefaultCodeVisitor::visit_try(self, stmt)
    }

    fn visit_bare_block(&mut self, stmt: &Rc<Stmt>) -> Self::Output {
        DefaultCodeVisitor::visit_bare_block(self, stmt)
    }

    fn visit_global_assignment(&mut self, decl: &Rc<Assignment>) -> Self::Output {
        DefaultCodeVisitor::visit_global_assignment(self, decl)
    }

    fn visit_import_declaration(&mut self, decl: &Rc<ImportDeclaration>) -> Self::Output {
        DefaultCodeVisitor::visit_import_declaration(self, decl)
    }

    fn visit_code_file(&mut self, file: &Rc<CodeFile>) -> Self::Output {
        DefaultCodeVisitor::visit_code_file(self, file)
    }
}
