use crate::decomp::ir;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ElemId(pub usize);

#[derive(Debug)]
pub struct Elem {
  pub entry: ElemId,
  pub exits: Vec<ElemId>,
  pub detail: Detail,
}

#[derive(Debug)]
pub enum Detail {
  BasicBlock(BasicBlock),
  Loop(Loop),
  If(If),
}

#[derive(Debug)]
pub struct BasicBlock {
  pub blkref: ir::BlockRef,
}

#[derive(Debug)]
pub struct Loop {
  pub entry: ElemId,
  pub exits: Vec<ElemId>,
  pub backedges: HashSet<ElemId>,
  pub body: Body,
}

#[derive(Debug)]
pub struct If {
  pub entry: ElemId,
  pub exit: ElemId,
  pub then_body: Body,
}

#[derive(Debug, Clone)]
pub struct Body {
  pub elems: HashSet<ElemId>,
  pub remap: HashMap<ElemId, ElemId>,
}

#[derive(Debug)]
pub struct Function {
  pub all_elems: Vec<Elem>,
  pub entry: ElemId,
  pub body: Body,
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

impl Body {
  fn new() -> Self {
    Self {
      elems: HashSet::new(),
      remap: HashMap::new(),
    }
  }

  fn remove_elem(&mut self, remove: ElemId) {
    if !self.elems.remove(&remove) {
      panic!("Cannot remove elems that are not part of this body!");
    }
  }

  fn remove_elems(&mut self, remove: &HashSet<ElemId>) {
    if !remove.is_subset(&self.elems) {
      panic!("Cannot remove elems that are not part of this body!");
    }
    self.elems = self.elems.difference(remove).cloned().collect();
  }

  // Insert loop, removing any elems it's captured
  fn insert_loop(&mut self, lp: Loop, all_elems: &mut Vec<Elem>) {
    // Step 0: Save some data
    let loop_entry = lp.entry;
    let loop_body = lp.body.clone();

    // Step 1: Wrap up into a proper elem
    let new_elem = Elem {
      entry: lp.entry,
      exits: lp.exits.clone(),
      detail: Detail::Loop(lp),
    };

    // Step 2: Insert it into "all_elems", assigning an ElemId
    let loop_id = ElemId(all_elems.len());
    all_elems.push(new_elem);

    // Step 3: Remove any captured elems
    self.remove_elems(&loop_body.elems);
    self.elems.insert(loop_id);
    self.remap.insert(loop_entry, loop_id);
  }

  // Insert ifstmt, removing any elems it's captured
  fn insert_ifstmt(&mut self, ifstmt: If, all_elems: &mut Vec<Elem>) {
    // Step 0: Save some data
    let ifstmt_entry = ifstmt.entry;
    let ifstmt_then = ifstmt.then_body.clone();

    // Step 1: Wrap up into a proper elem
    let new_elem = Elem {
      entry: ifstmt.entry,
      exits: vec![ifstmt.exit],
      detail: Detail::If(ifstmt),
    };

    // Step 2: Insert it into "all_elems", assigning an ElemId
    let ifstmt_id = ElemId(all_elems.len());
    all_elems.push(new_elem);

    // Step 3: Remove any captured elems
    self.remove_elems(&ifstmt_then.elems);
    self.remove_elem(ifstmt_entry);
    self.elems.insert(ifstmt_id);
    self.remap.insert(ifstmt_entry, ifstmt_id);
  }

  fn exit(&self, node: ElemId, exit_idx: usize, all_elems: &[Elem]) -> Option<ElemId> {
    let next = *all_elems[node.0].exits.get(exit_idx)?;
    self.remap.get(&next).cloned().or(Some(next))
  }

  fn exits(&self, node: ElemId, all_elems: &[Elem]) -> Vec<ElemId> {
    assert!(self.elems.get(&node).is_some());
    let mut exits = all_elems[node.0].exits.clone();
    for exit in &mut exits {
      if let Some(remap) = self.remap.get(exit) {
        *exit = *remap;
      }
    }
    exits
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
    infer_structure(func.entry, &mut func.body, &mut func.all_elems);
    func
  }
}

enum DFSAction<'a> {
  Cycle { from: ElemId, to: ElemId },
  Next(ElemId),
  Exit(ElemId),
  Backtrack(&'a [ElemId]),
  Done,
}

enum DFSPending {
  Expand(ElemId),
  Backtrack,
}

struct DFS<'a> {
  body: &'a Body,
  all_elems: &'a [Elem],
  visited: Vec<bool>,
  path: Vec<ElemId>,
  exit_idx: Vec<usize>,
  pending: Option<DFSPending>,
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
    let Some(action) = self.pending.take() else { return };
    match action {
      DFSPending::Expand(next) => {
        self.visited[next.0] = true;
        self.path.push(next);
        self.exit_idx.push(0);
      }
      DFSPending::Backtrack => {
        let node = self.path.pop().unwrap();
        self.exit_idx.pop();
        self.visited[node.0] = false;
      }
    }
  }

  fn path(&self) -> &[ElemId] {
    &self.path
  }

  fn next(&mut self) -> DFSAction<'_> {
    self.apply_pending();

    if self.path.len() == 0 {
      return DFSAction::Done;
    }
    let idx = self.path.len() - 1;

    let node = self.path[idx];
    let exit_idx = self.exit_idx[idx];
    self.exit_idx[idx] += 1;

    let Some(next) = self.body.exit(node, exit_idx, self.all_elems) else {
      self.pending = Some(DFSPending::Backtrack);
      return DFSAction::Backtrack(&self.path);
    };

    if self.body.elems.get(&next).is_none() {
      return DFSAction::Exit(next);
    }

    if self.visited[next.0] {
      return DFSAction::Cycle { from: node, to: next };
    }


    self.pending = Some(DFSPending::Expand(next));
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

fn infer_loop(entry: ElemId, body: &mut Body, all_elems: &mut Vec<Elem>) -> bool {
  // println!("Starting loop infer");
  // println!("  Body: {:?}", f.body);

  let mut dfs = DFS::new(entry, body, all_elems);
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
      _ => (),
    }
  }

  let Some(mut lp) = lp else { return false };

  // Successfully inferred a loop, we just need to finalize it up into
  // a proper elem and then insert it into the structure.

  // Step 1: We need to find the exits
  lp.exits = find_exits(lp.entry, &lp.body, all_elems);

  // Step 2: Insert it
  body.insert_loop(lp, all_elems);

  true
}

fn infer_if(body: &mut Body, all_elems: &mut Vec<Elem>) -> bool {
  // Consider each basic block as an if-stmt header
  let mut found: Option<(ElemId, ElemId, ElemId)> = None;
  for id in &body.elems {
    let elem = &all_elems[id.0];

    // If-stmt header needs to be a basic block
    if !matches!(elem.detail, Detail::BasicBlock(_)) { continue; }

    // If-stmt header needs to have exactly two exits
    let exits = body.exits(*id, &all_elems);
    if exits.len() != 2 { continue; }

    //println!("Candidate Block: {} => {:?}", id.0, exits);

    // Check for: {A, B}, A -> B
    {
      let (a, b) = (exits[0], exits[1]);
      let a_exits = body.exits(a, &all_elems);
      //println!("a_exits: {:?}", a_exits);
      if a_exits.len() == 1 && a_exits[0] == b {
        //println!("Found case 1");
        found = Some((*id, a, b));
        break;
      }
    }

    // Check for: {A, B}, B -> A
    {
      let (a, b) = (exits[0], exits[1]);
      let b_exits = body.exits(b, &all_elems);
      //println!("b_exits: {:?}", b_exits);
      if b_exits.len() == 1 && b_exits[0] == a {
        //println!("Found case 2");
        found = Some((*id, b, a));
        break;
      }
    }
  }

  let Some((entry, then, join)) = found else { return false };

  // Successfully inferred an if-stmt, we just need to finalize it up into
  // a proper elem and then insert it into the structure.

  // Step 1: Build If struct
  let mut ifstmt = If {
    entry,
    exit: join,
    then_body: Body::new(),
  };
  ifstmt.then_body.elems.insert(then);

  // Step 2: Insert it
  body.insert_ifstmt(ifstmt, all_elems);

  true
}

fn infer_structure(entry: ElemId, body: &mut Body, all_elems: &mut Vec<Elem>) {
  // infer at the top-level
  while infer_loop(entry, body, all_elems) {}
  while infer_if(body, all_elems) {}

  // TODO!
  // // recurse and infer at lower-levels
  // for id in &body.elems {

  // }
}

pub fn print(func: &Function) {
  print_recurse(&func.body, &func.all_elems, 0)
}

fn print_recurse(body: &Body, all_elems: &[Elem], indent_level: usize) {
  for id in itertools::sorted(body.elems.iter().cloned()) {
    print!("{:indent$}{:?} | ", "", id, indent=2*indent_level);
    let elem = &all_elems[id.0];
    let exits: Vec<_> = elem.exits.iter().map(|x| x.0).collect();
    match &elem.detail {
      Detail::BasicBlock(b) => println!("BasicBlock({})", b.blkref.0),
      Detail::Loop(lp) => {
        let backedges: Vec<_> = lp.backedges.iter().cloned().map(|x| x.0).collect();
        println!("Loop [entry={}, exits={:?}, backedges={:?}]", elem.entry.0, exits, backedges);
        print_recurse(&lp.body, all_elems, indent_level+1);
      }
      Detail::If(i) => {
        println!("If [entry={}, exits={:?}]", elem.entry.0, exits);
        print_recurse(&i.then_body, all_elems, indent_level+1);
      }
    }
  }
}
