use crate::ast::*;
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
  fn func_sig(&self, g: &mut Gen<'_>, func: &Function) -> fmt::Result;
  fn ret(&self, g: &mut Gen<'_>, ret: &Return) -> fmt::Result;
  fn call(&self, g: &mut Gen<'_>, name: &Expr, args: &[Expr], level: usize) -> fmt::Result;
}

struct Standard {}
impl FlavorImpl for Standard {
  fn func_sig(&self, g: &mut Gen<'_>, func: &Function) -> fmt::Result {
    let ret_str = match &func.ret {
      Some(ret) => format!("{}", ret),
      None => "_unknown_return_type".to_string(),
    };
    g.text(&format!("{} {}(void)", ret_str, func.name))
  }

  fn ret(&self, g: &mut Gen<'_>, ret: &Return) -> fmt::Result {
    g.text("return")?;

    match ret.vals.len() {
      0 => (),
      1 => {
        g.text(" ")?;
        g.expr(&ret.vals[0], 0, self)?;
      }
      2 => {
        g.text(" MAKE_32(")?;
        g.expr(&ret.vals[1], 0, self)?;
        g.text(", ")?;
        g.expr(&ret.vals[0], 0, self)?;
        g.text(")")?;
      }
      _ => panic!("Unsupported return values"),
    }

    match &ret.rt {
      ReturnType::Far => g.text("; /* FAR */")?,
      ReturnType::Near => g.text("; /* NEAR */")?,
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
  fn func_sig(&self, g: &mut Gen<'_>, func: &Function) -> fmt::Result {
    let name = &func.name;
    let name = if name.starts_with("F_") { &name[2..] } else { name };
    g.text(&format!("HYDRA_FUNC(H_{})", name))
  }

  fn ret(&self, g: &mut Gen<'_>, ret: &Return) -> fmt::Result {
    match ret.vals.len() {
      0 => (),
      1 => {
        g.text("AX = ")?;
        g.expr(&ret.vals[0], 0, self)?;
        g.text(";")?;
        g.endline()?;
      }
      2 => {
        g.text("DX = ")?;
        g.expr(&ret.vals[1], 0, self)?;
        g.text(";")?;
        g.endline()?;
        g.text("AX = ")?;
        g.expr(&ret.vals[0], 0, self)?;
        g.text(";")?;
        g.endline()?;
      }
      _ => panic!("Unsupported return values"),
    }
    match &ret.rt {
      ReturnType::Far => g.text("RETURN_FAR();")?,
      ReturnType::Near => g.text("RETURN_NEAR();")?,
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

  fn enter_indent(&mut self) {
    self.indent_level += 1;
  }

  fn leave_indent(&mut self) {
    assert!(self.indent_level >= 1);
    self.indent_level -= 1;
  }

  fn enter_block(&mut self) -> fmt::Result {
    self.text("{")?;
    self.enter_indent();
    Ok(())
  }

  fn leave_block(&mut self) -> fmt::Result {
    self.leave_indent();
    self.text("}")?;
    Ok(())
  }

  fn unary_oper(&mut self, oper: &UnaryOperator) -> fmt::Result {
    self.text(oper.as_operator_str())
  }

  fn binary_oper(&mut self, oper: &BinaryOperator) -> fmt::Result {
    self.text(oper.as_operator_str())
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
      Expr::HexConst(k) => {
        self.text(&format!("0x{:x}", k))?
      }
      Expr::DecimalConst(k) => {
        self.text(&format!("{}", k))?
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
      Expr::ArrayAccess(lhs, idx) => {
        self.expr(lhs, level+1, imp)?;
        self.text("[")?;
        self.expr(idx, level+1, imp)?;
        self.text("]")?;
      }
      Expr::StructAccess(lhs, mbr) => {
        self.expr(lhs, level+1, imp)?;
        self.text(".")?;
        self.expr(mbr, level+1, imp)?;
      }
      _ => {
        panic!("UNIMPL EXPR: {:?}", expr);
        //self.text(&format!("UNIMPL_EXPR /* {:?} */", expr))?;
      }

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
        // label is unindented one level
        self.leave_indent();
        self.text("")?;
        self.enter_indent();
        self.text(&format!("{}:;", l.0))?;
        self.endline()?;
      }
      Stmt::Expr(expr) => {
        self.expr(expr, 0, imp)?;
        self.text(";")?;
        self.endline()?;
      }
      Stmt::Assign(s) => {
        if let Some(typ) = &s.decltype {
          self.text(&format!("{} ", typ))?;
        }
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
        self.endline()?;
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
      Stmt::Switch(sw) => {
        self.text("switch (")?;
        self.expr(&sw.switch_val, 0, imp)?;
        self.text(") ")?;
        self.enter_block()?;
        self.endline()?;
        for case in &sw.cases {
          for (i, case_expr) in case.cases.iter().enumerate() {
            self.text("case ")?;
            self.expr(case_expr, 0, imp)?;
            self.text(":")?;
            if i+1 != case.cases.len() {
              self.endline()?;
            } else {
              self.text(" ")?;
            }
          }
          self.enter_block()?;
          self.endline()?;
          self.block(&case.body, imp)?;
          self.leave_block()?;
          self.endline()?;
        }
        if let Some(body) = &sw.default {
          self.text("default: ")?;
          self.enter_block()?;
          self.endline()?;
          self.block(body, imp)?;
          self.leave_block()?;
          self.endline()?;
        }
        self.leave_block()?;
        self.endline()?;
      }
      Stmt::Unreachable => {
        self.text("assert(0 && \"unreachable\");")?;
        self.endline()?;
      }
      _ => {
        panic!("UNIMPL STMT: {:?}", stmt);
        // self.text(&format!("UNIMPL_STMT; /* {:?} */", stmt))?;
        // self.endline()?;
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

  fn varmaps_def(&mut self, maps: &[VarMap], imp: &dyn FlavorImpl) -> fmt::Result {
    for d in maps {
      self.text(&format!("#define {} ", d.name))?;
      self.expr(&d.mapping_expr, 0, imp)?;
      self.endline()?;
    }
    Ok(())
  }

  fn varmaps_undef(&mut self, maps: &[VarMap], _imp: &dyn FlavorImpl) -> fmt::Result {
    for d in maps {
      self.text(&format!("#undef {}", d.name))?;
      self.endline()?;
    }
    Ok(())
  }

  fn vardecls(&mut self, decls: &[VarDecl], _imp: &dyn FlavorImpl) -> fmt::Result {
    for d in decls {
      self.text(&format!("{} ", d.typ))?;
      for (i, name) in d.names.iter().enumerate() {
        if i != 0 { self.text(", ")?; }
        self.text(name)?;
      }
      self.text(";")?;
      self.endline()?;
    }
    Ok(())
  }

  fn func(&mut self, func: &Function, imp: &dyn FlavorImpl) -> fmt::Result {
    imp.func_sig(self, func)?;
    self.endline()?;
    self.enter_block()?;
    self.endline()?;
    self.varmaps_def(&func.varmaps, imp)?;
    self.endline()?;
    self.vardecls(&func.vardecls, imp)?;
    self.endline()?;
    self.block(&func.body, imp)?;
    self.endline()?;
    self.varmaps_undef(&func.varmaps, imp)?;
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
