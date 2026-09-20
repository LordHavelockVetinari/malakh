use std::mem;
use std::rc::Rc;

use crate::compile::error::CompilationError;
use crate::parse::tree::{
    CodeFile, CodeVisitor, Condition, DefaultCodeVisitor, Expr, ExprType, InputType,
    ResultCodeVisitor, Stmt, StmtType,
};
use crate::util::ptr_set::PtrSet;

struct ForkingLoopDetector {
    forking_loops: PtrSet<Stmt>,
    current_non_forking_loops: PtrSet<Stmt>,
}

impl DefaultCodeVisitor for ForkingLoopDetector {
    type Error = CompilationError;

    fn visit_in(&mut self, expr: &Rc<Expr>) -> Result<(), Self::Error> {
        let Expr(ExprType::In(typ), _) = &**expr else {
            unreachable!();
        };
        if *typ == InputType::Fork {
            for loop_ in self.current_non_forking_loops.drain() {
                self.forking_loops.insert(loop_);
            }
        }
        Ok(())
    }

    fn visit_process_literal(&mut self, expr: &Rc<Expr>) -> Result<(), Self::Error> {
        let Expr(ExprType::ProcessLiteral(body), _) = &**expr else {
            unreachable!();
        };
        let outer_loops = mem::take(&mut self.current_non_forking_loops);
        self.visit_many(body)?;
        assert!(
            self.current_non_forking_loops.is_empty(),
            "all loops should be removed from the set"
        );
        self.current_non_forking_loops = outer_loops;
        Ok(())
    }

    fn visit_loop(&mut self, stmt: &Rc<Stmt>) -> Result<(), Self::Error> {
        let Stmt(StmtType::Loop(cond, body), _) = &**stmt else {
            unreachable!();
        };
        let mut reevaluated_conditions = Vec::new();
        match cond {
            None => {}
            Some(Condition::Boolean(boolean)) => {
                reevaluated_conditions.push(boolean);
            }
            Some(Condition::Assignment(asgn)) => {
                let Stmt(StmtType::Assignment(assignment), _) = &**asgn else {
                    unreachable!();
                };
                for value in &assignment.values {
                    if matches!(value.0, ExprType::Receive(_)) {
                        self.visit(value)?;
                    } else {
                        reevaluated_conditions.push(value);
                    }
                }
            }
        }
        self.current_non_forking_loops.insert(Rc::clone(stmt));
        self.visit_many(&reevaluated_conditions)?;
        self.visit_many(body)?;
        self.current_non_forking_loops.remove(stmt);
        Ok(())
    }
}

pub fn find_all_forking_loops(code: &Rc<CodeFile>) -> Result<PtrSet<Stmt>, CompilationError> {
    let mut detector = ForkingLoopDetector {
        forking_loops: PtrSet::new(),
        current_non_forking_loops: PtrSet::new(),
    };
    detector.visit(code)?;
    Ok(detector.forking_loops)
}
