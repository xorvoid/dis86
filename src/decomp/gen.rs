use crate::decomp::ast::*;
use std::fmt;

pub enum Flavor {
  Standard,
  Hydra,
}

impl Flavor {
  fn instantiate(&self) -> Box<dyn FlavorImpl> {
    match self {
      Flavor::Standard => Box::new(Standard{}),
      Flavor::Hydra => Box::new(Hydra{}),
    }
  }
}

trait FlavorImpl {
  fn func_sig(&self, g: &mut Gen<'_>, name: &str) -> fmt::Result;
  fn ret(&self, g: &mut Gen<'_>, ret: &Return) -> fmt::Result;
  fn call(&self, g: &mut Gen<'_>, name: &Expr, args: &[Expr], level: usize) -> fmt::Result;
}

struct Standard {}
impl FlavorImpl for Standard {
  fn func_sig(&self, g: &mut Gen<'_>, name: &str) -> fmt::Result {
    g.text(&format!("void {}(void)", name))
  }

  fn ret(&self, g: &mut Gen<'_>, ret: &Return) -> fmt::Result {
    match ret {
      Return::Far => g.text("return; /* FAR */")?,
      Return::Near => g.text("return; /* NEAR */")?,
    }
    Ok(())
  }

  fn call(&self, g: &mut Gen<'_>, name: &Expr, args: &[Expr], level: usize) -> fmt::Result {
    g.expr(name, level+1, self)?;
    g.text("(")?;
    for (i, arg) in args.iter().enumerate() {
      if i != 0 { g.text(", ")?; }
      g.expr(arg, 0, self)?;
    }
    g.text(")")?;
    Ok(())
  }
}

struct Hydra {}
impl FlavorImpl for Hydra {
  fn func_sig(&self, g: &mut Gen<'_>, name: &str) -> fmt::Result {
    let name = if name.starts_with("F_") { &name[2..] } else { name };
    g.text(&format!("HOOK_FUNC(H_{})", name))
  }

  fn ret(&self, g: &mut Gen<'_>, ret: &Return) -> fmt::Result {
    match ret {
      Return::Far => g.text("RETURN_FAR();")?,
      Return::Near => g.text("RETURN_NEAR();")?,
    }
    Ok(())
  }

  fn call(&self, g: &mut Gen<'_>, name: &Expr, args: &[Expr], level: usize) -> fmt::Result {
    g.expr(name, level+1, self)?;
    g.text("(m")?;
    for arg in args.iter() {
      g.text(", ")?;
      g.expr(arg, 0, self)?;
    }
    g.text(")")?;
    Ok(())
  }
}

struct Gen<'a> {
  f: &'a mut dyn fmt::Write,
  indent_level: usize,
  newline: bool,
}

impl<'a> Gen<'a> {
  fn new(f: &'a mut dyn fmt::Write) -> Self {
    Self { f, indent_level: 0, newline: true }
  }

  fn endline(&mut self) -> fmt::Result {
    self.f.write_str("\n")?;
    self.newline = true;
    Ok(())
  }

  fn text(&mut self, txt: &str) -> fmt::Result {
    if self.newline {
      write!(self.f, "{:indent$}", "", indent=2*self.indent_level)?;
      self.newline = false;
    }
    self.f.write_str(txt)
  }

  fn suppress_indent(&mut self) {
    self.newline = false;
  }

  fn enter_block(&mut self) -> fmt::Result {
    self.text("{")?;
    self.indent_level += 1;
    Ok(())
  }

  fn leave_block(&mut self) -> fmt::Result {
    self.indent_level -= 1;
    self.text("}")?;
    Ok(())
  }

  fn unary_oper(&mut self, oper: &UnaryOperator) -> fmt::Result {
    let s = match oper {
      UnaryOperator::Addr => "(u8*)&",
      UnaryOperator::Not => "!",
    };
    self.text(s)
  }

  fn binary_oper(&mut self, oper: &BinaryOperator) -> fmt::Result {
    let s = match oper {
      BinaryOperator::Add => "+",
      BinaryOperator::Sub => "-",
      BinaryOperator::UDiv => "/",
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
    self.text(s)
  }

  fn expr(&mut self, expr: &Expr, level: usize, imp: &dyn FlavorImpl) -> fmt::Result {
    match expr {
      Expr::Unary(u) => {
        self.unary_oper(&u.op)?;
        self.expr(&u.rhs, level+1, imp)?;
      }
      Expr::Binary(b) => {
        if level > 0 {
          self.text("(")?;
        }
        self.expr(&b.lhs, level+1, imp)?;
        self.text(" ")?;
        self.binary_oper(&b.op)?;
        self.text(" ")?;
        self.expr(&b.rhs, level+1, imp)?;
        if level > 0 {
          self.text(")")?;
        }
      }
      Expr::Const(k) => {
        let s = if *k > 128 || *k < -128 {
          let k = *k as u32;
          format!("0x{:x}", k)
        } else {
          format!("{}", k)
        };
        self.text(&s)?;
      }
      Expr::Name(n) => {
        self.text(n)?;
      }
      Expr::Cast(typ, expr) => {
        self.text(&format!("({})", typ))?;
        self.expr(expr, level+1, imp)?;
      }
      Expr::Deref(expr) => {
        self.text("*")?;
        self.expr(expr, level+1, imp)?;
      }
      Expr::Call(name, args) => {
        imp.call(self, name, args, level)?;
      }
      Expr::Abstract(name, args) => {
        self.text(&format!("{}(", name))?;
        for (i, arg) in args.iter().enumerate() {
          if i != 0 { self.text(", ")?; }
          self.expr(arg, 0, imp)?;
        }
        self.text(")")?;
      }
      _ => self.text(&format!("UNIMPL_EXPR /* {:?} */", expr))?,
    }
    Ok(())
  }

  fn goto(&mut self, label: &Label) -> fmt::Result {
    self.text("goto ")?;
    self.text(&label.0)?;
    self.text(";")
  }

  fn stmt(&mut self, stmt: &Stmt, imp: &dyn FlavorImpl) -> fmt::Result {
    match stmt {
      Stmt::Label(l) => {
        self.suppress_indent();
        self.text(&format!("{}:;", l.0))?;
        self.endline()?;
      }
      Stmt::VarDecl(typ, name) => {
        self.text(&format!("{} ", typ))?;
        self.text(&name)?;
        self.text(";")?;
        self.endline()?;
      }
      Stmt::Expr(expr) => {
        self.expr(expr, 0, imp)?;
        self.text(";")?;
        self.endline()?;
      }
      Stmt::Assign(s) => {
        self.expr(&s.lhs, 0, imp)?;
        self.text(" = ")?;
        self.expr(&s.rhs, 0, imp)?;
        self.text(";")?;
        self.endline()?;
      }
      Stmt::Goto(g) => {
        self.goto(&g.label)?;
        self.endline()?;
      }
      Stmt::CondGoto(g) => {
        self.text("if (")?;
        self.expr(&g.cond, 0, imp)?;
        self.text(") ")?;
        self.goto(&g.label_true)?;
        self.text("else ")?;
        self.goto(&g.label_false)?;
        self.endline()?;
      }
      Stmt::Return(r) => {
        imp.ret(self, r)?;
        self.endline()?;
      }
      Stmt::Loop(lp) => {
        self.text("while (1) ")?;
        self.enter_block()?;
        self.endline()?;
        self.block(&lp.body, imp)?;
        self.leave_block()?;
        self.endline()?;
      }
      Stmt::If(ifstmt) => {
        self.text("if (")?;
        self.expr(&ifstmt.cond, 0, imp)?;
        self.text(") ")?;
        self.enter_block()?;
        self.endline()?;
        self.block(&ifstmt.then_body, imp)?;
        self.leave_block()?;
        self.endline()?;
      }
      _ => {
        self.text(&format!("UNIMPL_STMT; /* {:?} */", stmt))?;
        self.endline()?;
      }
    }
    Ok(())
  }

  fn block(&mut self, blk: &Block, imp: &dyn FlavorImpl) -> fmt::Result {
    for stmt in &blk.0 {
      self.stmt(stmt, imp)?;
    }
    Ok(())
  }

  fn func(&mut self, func: &Function, imp: &dyn FlavorImpl) -> fmt::Result {
    imp.func_sig(self, &func.name)?;
    self.endline()?;
    self.enter_block()?;
    self.endline()?;
    self.block(&func.decls, imp)?;
    self.endline()?;
    self.block(&func.body, imp)?;
    self.leave_block()?;
    Ok(())
  }
}

pub fn generate_generic(func: &Function, f: &mut dyn fmt::Write, flavor: Flavor) -> fmt::Result {
  let mut g = Gen::new(f);
  let imp = flavor.instantiate();
  g.func(func, imp.as_ref())
}

pub fn generate(func: &Function, flavor: Flavor) -> Result<String, fmt::Error> {
  let mut buf = String::new();
  generate_generic(func, &mut buf, flavor)?;
  Ok(buf)
}
