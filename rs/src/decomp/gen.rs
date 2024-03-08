use crate::decomp::ast::*;
use std::fmt;

struct Gen<'a> {
  f: &'a mut dyn fmt::Write,
}

impl Gen<'_> {
  fn gen_uoper(&mut self, oper: &UnaryOperator) -> fmt::Result {
    let s = match oper {
      UnaryOperator::Addr => "(u8*)&",
    };
    write!(self.f, "{}", s)
  }

  fn gen_boper(&mut self, oper: &BinaryOperator) -> fmt::Result {
    let s = match oper {
      BinaryOperator::Add => "+",
      BinaryOperator::Sub => "-",
      BinaryOperator::And => "&",
      BinaryOperator::Or  => "|",
      BinaryOperator::Xor => "^",
      BinaryOperator::Shl => "<<",
      BinaryOperator::Shr => ">>",
      BinaryOperator::Eq  => "==",
      BinaryOperator::Neq => "!=",
      BinaryOperator::Gt  => ">",
      BinaryOperator::Geq => ">=",
      BinaryOperator::Lt  => "<",
      BinaryOperator::Leq => "<=",
    };
    write!(self.f, " {} ", s)
  }

  fn gen_expr(&mut self, expr: &Expr) -> fmt::Result {
    match expr {
      Expr::Unary(u) => {
        self.gen_uoper(&u.op)?;
        self.gen_expr(&u.rhs)?;
      }
      Expr::Binary(b) => {
        write!(self.f, "(")?;
        self.gen_expr(&b.lhs)?;
        self.gen_boper(&b.op)?;
        self.gen_expr(&b.rhs)?;
        write!(self.f, ")")?;
      }
      Expr::Const(k) => {
        if *k > 128 {
          write!(self.f, "0x{:x}", k)?;
        } else {
          write!(self.f, "{}", k)?;
        }
      }
      Expr::Name(n) => {
        write!(self.f, "{}", n)?;
      }
      Expr::Ptr16(seg, off) => {
        write!(self.f, "PTR_16(")?;
        self.gen_expr(seg)?;
        write!(self.f, ", ")?;
        self.gen_expr(off)?;
        write!(self.f, ")")?;
      }
      Expr::Ptr32(seg, off) => {
        write!(self.f, "PTR_32(")?;
        self.gen_expr(seg)?;
        write!(self.f, ", ")?;
        self.gen_expr(off)?;
        write!(self.f, ")")?;
      }
      Expr::Cast(typ, expr) => {
        write!(self.f, "({})", typ)?;
        self.gen_expr(expr)?;
      }
      Expr::Deref(expr) => {
        write!(self.f, "*")?;
        self.gen_expr(expr)?;
      }
      Expr::Call(name, args) => {
        self.gen_expr(name)?;
        write!(self.f, "(")?;
        for (i, arg) in args.iter().enumerate() {
          if i != 0 { write!(self.f, ", ")?; }
          self.gen_expr(arg)?;
        }
        write!(self.f, ")")?;
      }
      _ => write!(self.f, "UNIMPL_EXPR /* {:?} */", expr)?,
    }
    Ok(())
  }

  fn gen_stmt(&mut self, stmt: &Stmt) -> fmt::Result {
    match stmt {
      Stmt::None => (),
      Stmt::Assign(s) => {
        self.gen_expr(&s.lhs)?;
        write!(self.f, " = ")?;
        self.gen_expr(&s.rhs)?;
        writeln!(self.f, ";")?;
      }
      Stmt::Goto(g) => {
        writeln!(self.f, "goto {};", g.tgt.0)?;
        writeln!(self.f, "")?;
      }
      Stmt::CondGoto(g) => {
        write!(self.f, "if (")?;
        self.gen_expr(&g.cond)?;
        writeln!(self.f, ") goto {};", g.tgt_true.0)?;
        writeln!(self.f, "else goto {};", g.tgt_false.0)?;
        writeln!(self.f, "")?;
      }
      _ => writeln!(self.f, "UNIMPL_STMT;")?,
    }
    Ok(())
  }

  fn gen(&mut self, func: &Function) -> fmt::Result {
    for stmt in &func.body {
      self.gen_stmt(stmt)?;
    }
    Ok(())
  }
}

pub fn generate(func: &Function, f: &mut dyn fmt::Write) -> fmt::Result {
  let mut g = Gen { f };
  g.gen(func)
}
