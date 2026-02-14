#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Opcode {
  Nop,
  Pin,
  Ref,
  Phi,
  Unimpl,

  Add,
  Sub,
  Shl,
  Shr,    // signed
  UShr,   // unsigned
  And,
  Or,
  Xor,
  IMul,  // signed
  UMul,  // unsigned
  IDiv,  // signed
  UDiv,  // unsigned

  Neg,
  Not,  // bitwise

  SignExtTo16,
  SignExtTo32,

  Load8,
  Load16,
  Load32,
  Store8,
  Store16,
  Store32,
  ReadVar8,
  ReadVar16,
  ReadVar32,
  WriteVar8,
  WriteVar16,
  WriteVar32,
  ReadArr8,
  ReadArr16,
  ReadArr32,
  WriteArr8,
  WriteArr16,
  Lower16,     // |n: u32| => n as u16
  Upper16,     // |n: u32| => (n >> 16) as u16
  Make32,      // |high: u16, low: u16| => (high as u32) << 16 | (low as u32)

  UpdateFlags,
  EqFlags,     // Maps to: JE  / JZ
  NeqFlags,    // Maps to: JNE / JNZ
  GtFlags,     // Maps to: JG  / JNLE
  GeqFlags,    // Maps to: JGE / JNL
  LtFlags,     // Maps to: JL  / JNGE
  LeqFlags,    // Maps to: JLE / JNG
  UGtFlags,    // Maps to: JA  / JNBE
  UGeqFlags,   // Maps to: JAE / JNB  / JNC
  ULtFlags,    // Maps to: JB  / JNAE / JC
  ULeqFlags,   // Maps to: JBE / JNA
  SignFlags,   // Maps to: JS and inverted for JNS,

  Eq,          // Operator: == (any sign)
  Neq,         // Operator: != (any sign)
  Gt,          // Operator: >  (signed)
  Geq,         // Operator: >= (signed)
  Lt,          // Operator: <  (signed)
  Leq,         // Operator: <= (signed)
  UGt,         // Operator: >  (unsigned)
  UGeq,        // Operator: >= (unsigned)
  ULt,         // Operator: <  (unsigned)
  ULeq,        // Operator: <=  (unsigned)
  Sign,        // Is Signed?
  NotSign,     // Is not signed?

  CallFar,
  CallNear,
  CallPtr,
  CallArgs,
  Int,

  RetFar,
  RetNear,

  Jmp,
  Jne,
  JmpTbl,

  // TODO: HMMM.... Better Impl?
  AssertEven,
  AssertPos,
}


impl Opcode {
  pub fn as_str(&self) -> &'static str {
    match self {
      Opcode::Nop         => "nop",
      Opcode::Pin         => "pin",
      Opcode::Ref         => "ref",
      Opcode::Phi         => "phi",
      Opcode::Unimpl      => "unimpl",
      Opcode::Sub         => "sub",
      Opcode::Add         => "add",
      Opcode::Shl         => "shl",
      Opcode::Shr         => "shr",
      Opcode::UShr        => "ushr",
      Opcode::And         => "and",
      Opcode::Or          => "or",
      Opcode::Xor         => "xor",
      Opcode::IMul        => "imul",
      Opcode::UMul        => "umul",
      Opcode::IDiv        => "idiv",
      Opcode::UDiv        => "udiv",
      Opcode::Neg         => "neg",
      Opcode::Not         => "not",
      Opcode::SignExtTo16 => "signext16",
      Opcode::SignExtTo32 => "signext32",
      //Opcode::AddrOf      => "addrof",
      Opcode::Load8       => "load8",
      Opcode::Load16      => "load16",
      Opcode::Load32      => "load32",
      Opcode::Store8      => "store8",
      Opcode::Store16     => "store16",
      Opcode::Store32     => "store32",
      Opcode::ReadVar8    => "readvar8",
      Opcode::ReadVar16   => "readvar16",
      Opcode::ReadVar32   => "readvar32",
      Opcode::WriteVar8   => "writevar8",
      Opcode::WriteVar16  => "writevar16",
      Opcode::WriteVar32  => "writevar32",
      Opcode::ReadArr8    => "readarr8",
      Opcode::ReadArr16   => "readarr16",
      Opcode::ReadArr32   => "readarr32",
      Opcode::WriteArr8   => "writearr8",
      Opcode::WriteArr16  => "writearr16",
      Opcode::Lower16     => "lower16",
      Opcode::Upper16     => "upper16",
      Opcode::Make32      => "make32",
      Opcode::UpdateFlags => "updf",
      Opcode::EqFlags     => "eqf",
      Opcode::NeqFlags    => "neqf",
      Opcode::GtFlags     => "gtf",
      Opcode::GeqFlags    => "geqf",
      Opcode::LtFlags     => "ltf",
      Opcode::LeqFlags    => "leqf",
      Opcode::UGtFlags    => "ugtf",
      Opcode::UGeqFlags   => "ugeqf",
      Opcode::ULtFlags    => "ultf",
      Opcode::ULeqFlags   => "uleqf",
      Opcode::SignFlags   => "signf",
      Opcode::Eq          => "eq",
      Opcode::Neq         => "neq",
      Opcode::Gt          => "gt",
      Opcode::Geq         => "geq",
      Opcode::Lt          => "lt",
      Opcode::Leq         => "leq",
      Opcode::UGt         => "ugt",
      Opcode::UGeq        => "ugeq",
      Opcode::ULt         => "ult",
      Opcode::ULeq        => "uleq",
      Opcode::Sign        => "sign",
      Opcode::NotSign     => "notsign",
      Opcode::CallFar     => "callfar",
      Opcode::CallNear    => "callnear",
      Opcode::CallPtr     => "callptr",
      Opcode::CallArgs    => "callargs",
      Opcode::Int         => "int",
      Opcode::RetFar      => "retf",
      Opcode::RetNear     => "retn",
      Opcode::Jmp         => "jmp",
      Opcode::Jne         => "jne",
      Opcode::JmpTbl      => "jmptbl",

      Opcode::AssertEven => "assert_even",
      Opcode::AssertPos  => "assert_pos",
    }
  }
}

impl Opcode {
  pub fn is_load(&self) -> bool {
    match self {
      Opcode::Load8 => true,
      Opcode::Load16 => true,
      Opcode::Load32 => true,
      _ => false,
    }
  }

  pub fn is_store(&self) -> bool {
    match self {
      Opcode::Store8 => true,
      Opcode::Store16 => true,
      Opcode::Store32 => true,
      _ => false,
    }
  }

  pub fn is_mem_op(&self) -> bool {
    match self {
      Opcode::Load8 => true,
      Opcode::Load16 => true,
      Opcode::Load32 => true,
      Opcode::Store8 => true,
      Opcode::Store16 => true,
      Opcode::Store32 => true,
      Opcode::ReadVar8 => true,
      Opcode::ReadVar16 => true,
      Opcode::ReadVar32 => true,
      Opcode::WriteVar8 => true,
      Opcode::WriteVar16 => true,
      Opcode::WriteVar32 => true,
      _ => false,
    }
  }

  pub fn is_call(&self) -> bool {
    match self {
      Opcode::CallFar | Opcode::CallNear | Opcode::CallPtr | Opcode::CallArgs => true,
      _ => false,
    }
  }

  pub fn operation_size(&self) -> u16 {
    match self {
      Opcode::Load8 => 1,
      Opcode::Load16 => 2,
      Opcode::Load32 => 4,
      Opcode::Store8 => 1,
      Opcode::Store16 => 2,
      _ => panic!("invalid"),
    }
  }

  pub fn has_no_result(&self) -> bool {
    match self {
      Opcode::Nop => true,
      //Opcode::Pin => true,
      Opcode::Store8 => true,
      Opcode::Store16 => true,
      Opcode::WriteVar8 => true,
      Opcode::WriteVar16 => true,
      Opcode::WriteVar32 => true,
      Opcode::RetFar => true,
      Opcode::RetNear => true,
      Opcode::Jmp => true,
      Opcode::Jne => true,
      Opcode::JmpTbl => true,
      _ => false,
    }
  }

  pub fn maybe_unused(&self) -> bool {
    match self {
      Opcode::Nop => true,
      Opcode::Pin => true,
      Opcode::Store8 => true,
      Opcode::Store16 => true,
      Opcode::WriteVar8 => true,
      Opcode::WriteVar16 => true,
      Opcode::WriteVar32 => true,
      Opcode::CallFar => true,
      Opcode::CallNear => true,
      Opcode::CallPtr => true,
      Opcode::CallArgs => true,
      Opcode::Int => true,
      Opcode::RetFar => true,
      Opcode::RetNear => true,
      Opcode::Jmp => true,
      Opcode::Jne => true,
      Opcode::JmpTbl => true,
      _ => false,
    }
  }

  pub fn has_side_effects(&self) -> bool {
    match self {
      Opcode::Pin => true,
      Opcode::Store8 => true,
      Opcode::Store16 => true,
      Opcode::WriteVar8 => true,
      Opcode::WriteVar16 => true,
      Opcode::WriteVar32 => true,
      Opcode::CallFar => true,
      Opcode::CallNear => true,
      Opcode::CallPtr => true,
      Opcode::CallArgs => true,
      Opcode::Int => true,
      Opcode::RetFar => true,
      Opcode::RetNear => true,
      Opcode::Jmp => true,
      Opcode::Jne => true,
      Opcode::JmpTbl => true,
      Opcode::AssertEven => true,
      Opcode::AssertPos => true,
      _ => false,
    }
  }
}
