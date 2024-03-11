use crate::decomp::ir;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ElemId(usize);

#[derive(Debug)]
struct Elem {
  entry: ElemId,
  exits: Vec<ElemId>,
  detail: Detail,
}

#[derive(Debug)]
enum Detail {
  BasicBlock(BasicBlock),
  Loop(Loop),
  If(If),
}

#[derive(Debug)]
struct BasicBlock {
  blkref: ir::BlockRef,
}

#[derive(Debug)]
struct Loop {
  entry: ElemId,
  exits: Vec<ElemId>,
  backedges: HashSet<ElemId>,
  body: Body,
}

#[derive(Debug)]
struct If {
  entry: ElemId,
  exit: ElemId,
  then_body: Body,
  else_body: Body,
}

#[derive(Debug)]
struct Body {
  elems: HashSet<ElemId>,
}

#[derive(Debug)]
pub struct Function {
  all_elems: Vec<Elem>,
  entry: ElemId,
  body: Body,
}

impl Loop {
  fn new(entry: ElemId) -> Self {
    Self {
      entry,
      exits: vec![],
      backedges: HashSet::new(),
      body: Body::new(),
    }
  }
}

impl Elem {
  // HAX HAX HAX THIS IS USELESS FOR ANYTHING EXCEPT LOOPS!
  fn body(&self) -> Option<&Body> {
    match &self.detail {
      Detail::BasicBlock(_) => None,
      Detail::Loop(x) => Some(&x.body),
      Detail::If(_) => None,
    }
  }
}

impl Body {
  fn new() -> Self {
    Self { elems: HashSet::new() }
  }

  // Insert sub-elem, removing any elems it's captured
  fn insert_sub(&mut self, sub: ElemId, sub_body: &Body) {
    if !sub_body.elems.is_subset(&self.elems) {
      panic!("An inserted sub-elem must only use a subset of elems");
    }
    self.elems = self.elems.difference(&sub_body.elems).cloned().collect();
    self.elems.insert(sub);
  }
}

impl Function {
  fn from_ir_naive(ir: &ir::IR) -> Self {
    let mut func = Function {
      all_elems: vec![],
      entry: ElemId(0),
      body: Body::new(),
    };

    for b in 0..ir.blocks.len() {
      let instr = ir.blocks[b].instrs.last().unwrap();
      let mut exits = vec![];
      match instr.opcode {
        ir::Opcode::Ret => (),
        ir::Opcode::Jmp => {
          exits.push(ElemId(instr.operands[0].unwrap_block().0));
        }
        ir::Opcode::Jne => {
          exits.push(ElemId(instr.operands[1].unwrap_block().0));
          exits.push(ElemId(instr.operands[2].unwrap_block().0));
        }
        _ => panic!("Expected last instruction to be a branching instruction: {:?}", instr),
      }

      func.all_elems.push(Elem {
        entry: ElemId(b),
        exits,
        detail: Detail::BasicBlock(BasicBlock { blkref: ir::BlockRef(b) }),
      });
      func.body.elems.insert(ElemId(b));
    }

    func
  }

  pub fn from_ir(ir: &ir::IR) -> Self {
    let mut func = Self::from_ir_naive(ir);
    infer_loop(&mut func);
    func
  }
}

enum DFSAction {
  Cycle { from: ElemId, to: ElemId },
  Next(ElemId),
  Exit(ElemId),
  Done,
}

struct DFS<'a> {
  body: &'a Body,
  all_elems: &'a [Elem],
  visited: Vec<bool>,
  path: Vec<ElemId>,
  exit_idx: Vec<usize>,
  pending: Option<ElemId>,
}

impl<'a> DFS<'a> {
  fn new(entry: ElemId, body: &'a Body, all_elems: &'a [Elem]) -> Self {
    let mut visited = vec![];
    visited.resize(all_elems.len(), false);
    visited[entry.0] = true;

    Self {
      body,
      all_elems,
      visited,
      path: vec![entry],
      exit_idx: vec![0],
      pending: None,
    }
  }

  fn apply_pending(&mut self) {
    let Some(next) = self.pending.take() else { return };
    self.visited[next.0] = true;
    self.path.push(next);
    self.exit_idx.push(0);
  }

  fn path(&self) -> &[ElemId] {
    &self.path
  }

  fn next(&mut self) -> DFSAction {
    self.apply_pending();

    if self.path.len() == 0 {
      return DFSAction::Done;
    }
    let idx = self.path.len() - 1;

    let node = self.path[idx];
    let exit_idx = self.exit_idx[idx];
    self.exit_idx[idx] += 1;

    let exits = &self.all_elems[node.0].exits;
    if exit_idx >= exits.len() {
      self.visited[node.0] = false;
      self.path.pop();
      self.exit_idx.pop();
      return self.next();
    }

    let next = exits[exit_idx];
    if self.visited[next.0] {
      return DFSAction::Cycle { from: node, to: next };
    }

    if self.body.elems.get(&next).is_none() {
      return DFSAction::Exit(next);
    }

    self.pending = Some(next);
    DFSAction::Next(next)
  }
}

fn find_exits(entry: ElemId, body: &Body, all_elems: &[Elem]) -> Vec<ElemId> {
  let mut dfs = DFS::new(entry, body, all_elems);
  let mut exits = HashSet::new();
  loop {
    match dfs.next() {
      DFSAction::Done => break,
      DFSAction::Exit(exit) => { exits.insert(exit); }
      _ => (),
    }
  }
  exits.into_iter().collect()
}

fn infer_loop(f: &mut Function) -> bool {
  let mut dfs = DFS::new(f.entry, &f.body, &f.all_elems);
  let mut lp: Option<Loop> = None;
  loop {
    match dfs.next() {
      DFSAction::Done => break,
      DFSAction::Cycle { from, to } => {
        if lp.is_none() {
          lp = Some(Loop::new(to));
        }
        let lp = lp.as_mut().unwrap();

        // Only work on one loop at a time, ignore any others
        if lp.entry != to {
          continue;
        }

        // Update this loop and add all blocks in the loop body path
        lp.backedges.insert(from);
        for elem in dfs.path().iter().cloned().rev() {
          lp.body.elems.insert(elem);
          if elem == lp.entry { break; }
        }
      }
      DFSAction::Exit(exit) => panic!("EXIT UMIMPL: {:?}", exit),
      DFSAction::Next(_) => (),
    }
  }

  let Some(mut lp) = lp else { return false };

  // Successfully inferred a loop, we just need to finalize it up into
  // a proper elem and then insert it into the structure.

  // Step 1: We need to find the exits
  lp.exits = find_exits(lp.entry, &lp.body, &f.all_elems);

  // Step 2: Wrap up into a proper elem
  let loop_elem = Elem {
    entry: lp.entry,
    exits: lp.exits.clone(),
    detail: Detail::Loop(lp),
  };

  // Step 3: Insert it into the body, replacing the old elems
  let id = ElemId(f.all_elems.len());
  f.all_elems.push(loop_elem);
  let body = f.all_elems[id.0].body().unwrap();
  f.body.insert_sub(id, body);
  //println!("{:#?}", loop_elem);

  true
}
