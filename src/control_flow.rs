use crate::ir;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Write;

// FIXME: THE 'remap' HERE IS REALLY CLUNKY AND FRAGILE: IT NEEDS TO BE COMPLETELY RETHOUGHT

const LABEL_BLOCKS_ALWAYS: bool = false;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ElemId(pub usize);

#[derive(Debug, Clone)]
pub enum Jump {
  None,                           // no jump, e.g. infinite loop, return, etc
  UncondFallthrough,              // jmp elided, fallthrough
  UncondTarget(ElemId),           // jmp not elided
  CondTargetTrue(ElemId),         // jne true tgt needed, false elided, fallthrough
  CondTargetFalse(ElemId),        // jne false tgt needed, true elided, fallthrough
  CondTargetBoth(ElemId, ElemId), // jne not elided
  Table(Vec<ElemId>),             // jmptbl not elided
  //Continue,                       // uncond "continue" loop backedge
}

impl Jump {
  pub fn cond_inverted(&self) -> bool { matches!(self, Jump::CondTargetFalse(_)) }
}

#[derive(Debug)]
pub struct Elem {
  pub entry: ElemId,
  pub exits: Vec<ElemId>,
  pub jump: Option<Jump>,  // should only be None during early construction
  pub detail: Detail,
}

#[derive(Debug)]
pub enum Detail {
  BasicBlock(BasicBlock),
  Goto(Goto),
  ElemBlock(ElemBlock),
  Loop(Loop),
  If(If),
  Switch(Switch),
}

#[derive(Debug)]
pub struct BasicBlock {
  pub labeled: bool,
  pub jump_table: bool,
  pub preds: Vec<ElemId>,
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
  pub inverted: bool,
  pub then_body: Body,
}

#[derive(Debug)]
pub struct Switch {
  pub entry: ElemId,
  pub exits: Vec<ElemId>,
  pub cases: Vec<ElemId>,
  pub body: Body,
  //pub cases: Vec<Case>,
}

#[derive(Debug)]
pub struct Goto {
  pub target: ElemId,
}

#[derive(Debug)]
pub struct ElemBlock {
  pub entry: ElemId,
  pub exits: Vec<ElemId>,
  pub body: Body,
}

// #[derive(Debug)]
// pub struct Case {
//   pub entry: ElemId,
//   pub exits: Vec<ElemId>,
//   pub case: CaseVariant,
// }

// #[derive(Debug, Clone)]
// pub enum Case {
//   Goto(Goto),
//   Body(Body),
// }

#[derive(Debug, Clone)]
pub struct Body {
  pub entry: ElemId,
  pub elems: HashSet<ElemId>,
  pub remap: HashMap<ElemId, ElemId>,
  pub layout: Vec<ElemId>,
}

// Option needed so we can deal with a mut overlap by temporarily take'ing an element
// Compiler can't deduce the mut-splitting here so we are effectively doing that manually
#[derive(Debug)]
pub struct ControlFlowData(Vec<Option<Elem>>);

impl ControlFlowData {
  fn new() -> Self { Self(vec![]) }
  fn len(&self) -> usize { self.0.len() }
  fn append(&mut self, elem: Elem) -> ElemId { let id = ElemId(self.len()); self.0.push(Some(elem)); id }
  fn get(&self, id: ElemId) -> &Elem { self.0[id.0].as_ref().unwrap() }
  fn get_mut(&mut self, id: ElemId) -> &mut Elem { self.0[id.0].as_mut().unwrap() }
  fn checkout(&mut self, id: ElemId) -> Elem { self.0[id.0].take().unwrap() }
  fn release(&mut self, id: ElemId, elem: Elem) { assert!(self.0[id.0].is_none()); self.0[id.0] = Some(elem); }
  fn next_id(&self) -> ElemId { ElemId(self.0.len()) }

  fn append_with_id(&mut self, id: ElemId, elem: Elem) {
    assert!(id.0 >= self.len());
    while self.len() < id.0 {
      self.0.push(None);
    }
    assert!(id.0 == self.len());
    self.append(elem);
  }

  #[allow(unused)]
  fn debug_dump(&self) {
    for (i, elem) in self.0.iter().enumerate() {
      if let Some(elem) = elem {
        println!("{:3} | {:?}", i, elem);
      }
    }
  }
}

#[derive(Debug)]
pub struct Function {
  pub entry: ElemId,
  pub body: Body,
}

#[derive(Debug)]
pub struct ControlFlow {
  data: ControlFlowData,
  pub func: Function,
}

impl ControlFlow {
  #[allow(unused)]
  fn debug_dump(&self) {
    self.data.debug_dump();
    println!("{:?}", self.func.body);
  }
}

impl Loop {
  fn new(entry: ElemId) -> Self {
    Self {
      entry,
      exits: vec![],
      backedges: HashSet::new(),
      body: Body::new(entry),
    }
  }
}

impl Body {
  fn new(entry: ElemId) -> Self {
    Self {
      entry,
      elems: HashSet::new(),
      remap: HashMap::new(),
      layout: vec![],
    }
  }

  fn elem_is_movable_from(&self, id: ElemId) -> bool {
    self.elems.get(&id).is_some() && id != self.entry
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
  fn insert_loop(&mut self, lp: Loop, data: &mut ControlFlowData) {
    // Step 0: Save some data
    let loop_entry = lp.entry;
    let loop_body = lp.body.clone();

    // Step 1: Wrap up into a proper elem
    let new_elem = Elem {
      entry: lp.entry,
      exits: lp.exits.clone(),
      jump: None,
      detail: Detail::Loop(lp),
    };

    // Step 2: Insert it into "data", assigning an ElemId
    let loop_id = data.append(new_elem);

    // Step 3: Remove any captured elems
    self.remove_elems(&loop_body.elems);
    self.elems.insert(loop_id);
    self.remap.insert(loop_entry, loop_id);
  }

  // Insert ifstmt, removing any elems it's captured
  fn insert_ifstmt(&mut self, ifstmt: If, data: &mut ControlFlowData) {
    // Step 0: Save some data
    let ifstmt_entry = ifstmt.entry;
    let ifstmt_then = ifstmt.then_body.clone();

    // Step 1: Wrap up into a proper elem
    let new_elem = Elem {
      entry: ifstmt.entry,
      exits: vec![ifstmt.exit],
      jump: None,
      detail: Detail::If(ifstmt),
    };

    // Step 2: Insert it into "data", assigning an ElemId
    let ifstmt_id = data.append(new_elem);

    // Step 3: Remove any captured elems
    self.remove_elems(&ifstmt_then.elems);
    self.remove_elem(ifstmt_entry);
    self.elems.insert(ifstmt_id);
    self.remap.insert(ifstmt_entry, ifstmt_id);
  }

  // Insert switch, removing any elems it's captured
  fn insert_switch(&mut self, sw: Switch, data: &mut ControlFlowData) {
    // Step 0: Save some data
    let switch_entry = sw.entry;
    //let switch_cases = sw.cases.clone();

    // Step 1: Wrap up into a proper elem
    let new_elem = Elem {
      entry: sw.entry,
      exits: sw.exits.clone(),
      jump: None,
      detail: Detail::Switch(sw),
    };

    // Step 2: Insert it into "data", assigning an ElemId
    let switch_id = data.append(new_elem);

    // Step 3: Remove any captured elems

    // FIXME!!!!
    // for case in &switch_cases {
    //   match case {
    //     Case::Goto(_) => (),
    //     Case::Body(body) => self.remove_elems(&body.elems),
    //   }
    // }

    self.remove_elem(switch_entry);
    self.elems.insert(switch_id);
    self.remap.insert(switch_entry, switch_id);
  }

  // Insert goto
  fn insert_goto(&mut self, goto: Goto, data: &mut ControlFlowData) -> ElemId {
    // Step 1: Wrap up into a proper elem
    let new_elem = Elem {
      entry: data.next_id(),
      exits: vec![goto.target],
      jump: None,
      detail: Detail::Goto(goto),
    };

    // Step 2: Insert it into "data", assigning an ElemId
    let goto_id = data.append(new_elem);

    // Step 3: Remove any captured elems
    // NOTE: Goto captures NO elems
    self.elems.insert(goto_id);

    goto_id
  }

  // Insert block
  fn insert_elem_block(&mut self, blk: ElemBlock, data: &mut ControlFlowData) -> ElemId {
    // Step 0: Save some data
    let blk_entry = blk.entry;
    //let blk_body = blk.body.clone();

    // Step 1: Wrap up into a proper elem
    let new_elem = Elem {
      entry: blk.entry,
      exits: blk.exits.clone(),
      jump: None,
      detail: Detail::ElemBlock(blk),
    };

    // Step 2: Insert it into "data", assigning an ElemId
    let blk_id = data.append(new_elem);

    // Step 3: Remove any captured elems
    self.elems.insert(blk_id);
    self.remap.insert(blk_entry, blk_id);
    // FIXME: NOT REMOVING "blk_body" ELEMS BEACUSE THE CONSTRUCTION TAKES THEM FROM THE
    // PARENT'S PARENT .... HMMMM

    blk_id
  }

  fn exit(&self, node: ElemId, exit_idx: usize, data: &ControlFlowData) -> Option<ElemId> {
    let next = *data.get(node).exits.get(exit_idx)?;
    self.remap.get(&next).cloned().or(Some(next))
  }

  // returns None if some of the exits would escape the body
  fn exits(&self, node: ElemId, data: &ControlFlowData) -> Option<Vec<ElemId>> {
    _ = self.elems.get(&node)?;
    let mut exits = data.get(node).exits.clone();
    for exit in &mut exits {
      if let Some(remap) = self.remap.get(exit) {
        *exit = *remap;
      }
    }
    Some(exits)
  }

  pub fn lookup_from_blkref(&self, blkref: ir::BlockRef) -> Option<ElemId> {
    let bb_id = ElemId(blkref.0);
    let id = self.remap.get(&bb_id).unwrap_or(&bb_id);
    self.elems.get(id).cloned()
  }

  pub fn lookup_from_id(&self, id: ElemId) -> Option<ElemId> {
    let id = self.remap.get(&id).unwrap_or(&id);
    self.elems.get(id).cloned()
  }
}

impl ControlFlow {
  fn from_ir_naive(ir: &ir::IR) -> Self {
    let entry = ElemId(0);

    let mut cf = ControlFlow {
      data: ControlFlowData::new(),
      func: Function {
        entry,
        body: Body::new(entry),
      }
    };

    for b in ir.iter_blocks() {
      let exits = ir.block(b).exits().into_iter().map(|x| ElemId(x.0)).collect();
      let preds = ir.block(b).preds.iter().map(|x| ElemId(x.0)).collect();
      let jump_table = ir.block(b).instrs.last().unwrap().opcode == ir::Opcode::JmpTbl;

      cf.data.append_with_id(ElemId(b.0), Elem {
        entry: ElemId(b.0),
        exits,
        jump: None,
        detail: Detail::BasicBlock(BasicBlock {
          blkref: b,
          labeled: false,
          jump_table,
          preds,
        }),
      });
      cf.func.body.elems.insert(ElemId(b.0));
    }

    cf
  }

  pub fn from_ir(ir: &ir::IR) -> Self {
    let mut cf = Self::from_ir_naive(ir);
    infer_structure(&mut cf.func.body, None, &mut cf.data);
    //cf.debug_dump();
    schedule_layout(&mut cf.func.body, &mut cf.data);
    label_blocks(&mut cf);
    cf
  }

  pub fn elem(&self, id: ElemId) -> &Elem {
    self.data.get(id)
  }

  pub fn iter(&self) -> ControlFlowIter<'_> {
    ControlFlowIter::new(self)
  }
}

pub struct ControlFlowIter<'a> {
  cf: &'a ControlFlow,
  state: Vec<(&'a Body, usize)>,
}

pub struct ControlFlowIterElem<'a> {
  pub id: ElemId,
  pub elem: &'a Elem,
  pub cfdata: &'a ControlFlowData,
  pub depth: usize,
}

impl<'a> ControlFlowIter<'a> {
  fn new(cf: &'a ControlFlow) -> Self {
    Self { cf, state: vec![(&cf.func.body, 0)] }
  }
}

impl<'a> Iterator for ControlFlowIter<'a> {
  type Item = ControlFlowIterElem<'a>;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      if self.state.len() == 0 {
        return None;
      }
      let (body, idx) = self.state.as_mut_slice().last_mut().unwrap();
      let Some(id) = body.layout.get(*idx) else {
        self.state.pop();
        continue;
      };
      *idx += 1;
      let elem = self.cf.data.get(*id);
      let depth = self.state.len() - 1;
      match &elem.detail {
        Detail::BasicBlock(_) => (),
        Detail::Goto(_) => (),
        Detail::ElemBlock(blk) => self.state.push((&blk.body, 0)),
        Detail::Loop(lp) => self.state.push((&lp.body, 0)),
        Detail::If(ifstmt) => self.state.push((&ifstmt.then_body, 0)),
        Detail::Switch(sw) => self.state.push((&sw.body, 0)),
        // _ => panic!("Unknown detail type: {:?}", elem.detail),
      }

      return Some(ControlFlowIterElem {
        id: *id,
        cfdata: &self.cf.data,
        elem,
        depth,
      });
    }
  }
}

#[allow(dead_code)]
enum DFSAction<'a> {
  Cycle { from: ElemId, to: ElemId },
  Next(ElemId),
  Exit(ElemId),
  Exclude(ElemId),
  Backtrack(&'a [ElemId]),
  Done,
}

enum DFSPending {
  Expand(ElemId),
  Backtrack,
}

struct DFS<'a> {
  body: &'a Body,
  exclude: Option<&'a HashSet<ElemId>>,
  data: &'a ControlFlowData,
  visited: Vec<bool>,
  path: Vec<ElemId>,
  exit_idx: Vec<usize>,
  pending: Option<DFSPending>,
}

impl<'a> DFS<'a> {
  fn new(entry: ElemId, body: &'a Body, exclude: Option<&'a HashSet<ElemId>>, data: &'a ControlFlowData) -> Self {
    let mut visited = vec![];
    visited.resize(data.len(), false);
    visited[entry.0] = true;

    Self {
      body,
      exclude,
      data,
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

    let Some(next) = self.body.exit(node, exit_idx, self.data) else {
      self.pending = Some(DFSPending::Backtrack);
      return DFSAction::Backtrack(&self.path);
    };

    if self.body.elems.get(&next).is_none() {
      return DFSAction::Exit(next);
    }

    if let Some(exclude) = &self.exclude {
      if exclude.get(&next).is_some() {
        return DFSAction::Exclude(next);
      }
    }

    if self.visited[next.0] {
      return DFSAction::Cycle { from: node, to: next };
    }


    self.pending = Some(DFSPending::Expand(next));
    DFSAction::Next(next)
  }
}

fn find_loop_exits(entry: ElemId, body: &Body, data: &ControlFlowData) -> Vec<ElemId> {
  let mut dfs = DFS::new(entry, body, None, data);
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

fn infer_loop(body: &mut Body, exclude: Option<&HashSet<ElemId>>, data: &mut ControlFlowData) -> bool {
  let mut dfs = DFS::new(body.entry, body, exclude, data);
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
  lp.exits = find_loop_exits(lp.entry, &lp.body, data);

  // Step 2: Insert it
  body.insert_loop(lp, data);

  true
}

// src -> ... -> dst (sequentially via single exits)
fn sequentially_reaching(src: ElemId, dst: ElemId, body: &Body, data: &ControlFlowData) -> Option<Vec<ElemId>> {
  let mut cur = src;
  let mut blks = vec![];
  loop {
    blks.push(cur);
    let exits = body.exits(cur, data)?;
    if exits.len() != 1 { return None; }
    if exits[0] == dst { return Some(blks); }
    cur = exits[0];
  }
}

fn infer_if(body: &mut Body, data: &mut ControlFlowData) -> bool {
  // Consider each basic block as an if-stmt header
  let mut found: Option<(ElemId, Vec<ElemId>, ElemId, bool)> = None;
  for id in itertools::sorted(body.elems.iter()) {
    let elem = data.get(*id);

    // If-stmt header needs to be a basic block
    if !matches!(elem.detail, Detail::BasicBlock(_)) { continue; }

    // If-stmt header needs to have exactly two exits inside the body
    let Some(exits) = body.exits(*id, data) else { continue };
    if exits.len() != 2 { continue; }

    //println!("Candidate Block: {} => {:?}", id.0, exits);

    // Check for: {A, B}, A -> ... -> B
    {
      let (a, b) = (exits[0], exits[1]);
      if let Some(blks) = sequentially_reaching(a, b, body, data) {
        found = Some((*id, blks, b, false));
        break;
      }
    }

    // Check for: {A, B}, B -> ... -> A
    {
      let (a, b) = (exits[0], exits[1]);
      if let Some(blks) = sequentially_reaching(b, a, body, data) {
        found = Some((*id, blks, a, true));
        break;
      }
    }
  }

  let Some((entry, then_blks, join, inverted)) = found else { return false };

  // Successfully inferred an if-stmt, we just need to finalize it up into
  // a proper elem and then insert it into the structure.

  // Step 1: Build If struct
  let mut ifstmt = If {
    entry,
    exit: join,
    inverted,
    then_body: Body::new(then_blks[0]),
  };
  for id in then_blks {
    ifstmt.then_body.elems.insert(id);
    let elem = data.get(id);
    if elem.entry != id {
      ifstmt.then_body.remap.insert(elem.entry, id);
    }
  }

  // Step 2: Insert it
  body.insert_ifstmt(ifstmt, data);

  true
}

fn try_infer_case_body(entry: ElemId, parent_body: &mut Body, data: &ControlFlowData) -> Option<ElemBlock> {
  // Can we move the target block inside the switch??
  if !parent_body.elem_is_movable_from(entry) {
    return None;
  }

  // Follow and capture all exits as long as they are in the parrent body
  let mut inner = HashSet::new();
  let mut exits = HashSet::new();
  let mut search_queue = VecDeque::new();

  inner.insert(entry);
  search_queue.push_back(entry);

  while let Some(id) = search_queue.pop_front() {
    for exit in data.get(id).exits.iter().cloned() {
      if inner.get(&exit).is_some() { continue; }
      if !parent_body.elem_is_movable_from(exit) {
        exits.insert(exit);
        continue;
      }

      //println!("exit: {:?}", exit);

      // HEURISTIC: elems with a very large number of preds are probably some join-block
      // FIXME: We only have preds on BasicBlocks, so we make that restriction
      if let Detail::BasicBlock(bb) = &data.get(exit).detail {
        //println!("preds: {:?}", bb.preds);
        if bb.preds.len() > 10 { // 10 is arbitrarily chosen
          exits.insert(exit);
          continue;
        }
      }

      inner.insert(exit);
      search_queue.push_back(exit);
    }
  }

  // Move blocks to new body
  let mut body = Body::new(entry);
  for id in inner.into_iter() {
    parent_body.remove_elem(id);
    body.elems.insert(id);
  }

  let exits = itertools::sorted(exits.into_iter()).collect();
  let blk = ElemBlock {
    entry,
    exits,
    body,
  };

  Some(blk)
}

fn infer_switch(body: &mut Body, data: &mut ControlFlowData) -> bool {
  // Consider each basic block as a switch-stmt header
  //let mut found: Option<(ElemId, Vec<ElemId>, ElemId, bool)> = None;
  for id in itertools::sorted(body.elems.iter()) {
    let elem = data.get(*id);

    // Switch-stmt header needs to be a basic block
    let Detail::BasicBlock(bb) = &elem.detail else { continue };

    // Switch-stmt header should be a jump_table
    if !bb.jump_table { continue; }

    let sw_entry = *id;
    let mut sw_body = Body::new(sw_entry);
    let mut sw_cases = vec![];
    let mut sw_exits = HashSet::new();
    for tgt in &elem.exits.clone() {
      // Try to build up block for the case
      let case_id = if let Some(blk) = try_infer_case_body(*tgt, body, data) {
        for exit in &blk.exits {
          sw_exits.insert(*exit);
        }
        sw_body.insert_elem_block(blk, data)
      } else {
        // Can't move the target into a sub-block, use a goto..
        sw_exits.insert(*tgt);
        let goto = Goto {
          target: *tgt,
        };
        sw_body.insert_goto(goto, data)
      };
      sw_cases.push(case_id);
    }

    let sw_exits = itertools::sorted(sw_exits.iter().cloned()).collect();
    let switch = Switch {
      entry: sw_entry,
      exits: sw_exits,
      cases: sw_cases,
      body:  sw_body,
    };

    body.insert_switch(switch, data);
    return true;
  }
  false
}

fn infer_structure(body: &mut Body, exclude: Option<&HashSet<ElemId>>, data: &mut ControlFlowData) {
  // infer at the top-level
  while infer_loop(body, exclude, data) {}
  while infer_switch(body, data) {}
  while infer_if(body, data) {}

  // recurse and infer at lower-levels
  for id in &body.elems {
    let mut elem = data.checkout(*id);
    match &mut elem.detail {
      Detail::BasicBlock(_) => (),
      Detail::Goto(_) => (),
      Detail::ElemBlock(blk) => infer_structure(&mut blk.body, None, data),
      Detail::Loop(lp) => infer_structure(&mut lp.body, Some(&lp.backedges), data),
      Detail::If(_ifstmt) => (), // TODO!!!
      Detail::Switch(sw) => infer_structure(&mut sw.body, None, data),
      //_ => panic!("Unknown detail type: {:?}", elem.detail),
    }
    data.release(*id, elem);
  }
}

fn schedule_layout(body: &mut Body, data: &mut ControlFlowData) {
  let _ = schedule_layout_body(body, None, data);
}

struct Parent<'a> {
  body: &'a Body,
  remain: &'a HashSet<ElemId>,
  next: Box<Option<&'a Parent<'a>>>,
}

impl<'a> Parent<'a> {
  fn elem_avail(&self, id: ElemId) -> Option<ElemId> {
    // Search immediate parent
    if self.remain.len() > 0 {
      if let Some(id) = self.body.lookup_from_id(id) {
        if self.remain.get(&id).is_some() { return Some(id); }
      }
    }

    // No blocks remaining immediate parent, search parent's parent (if possible)
    if let Some(next) = self.next.as_ref() {
      next.elem_avail(id)
    } else {
      None
    }
  }
}

#[must_use]
fn schedule_layout_basic_block(elem: &mut Elem, parent: &Parent, _data: &mut ControlFlowData) -> Option<ElemId> {
  let Detail::BasicBlock(_) = &elem.detail else { panic!("Expected basic block") };
  let exits = &elem.exits;

  let (next, jump) = if exits.len() == 0 {
    (None, Jump::None)
  } else if exits.len() == 1 {
    let tgt = exits[0];
    if let Some(tgt) = parent.elem_avail(tgt) {
      (Some(tgt), Jump::UncondFallthrough)
    } else {
      (None, Jump::UncondTarget(tgt))
    }
  } else if exits.len() == 2 {
    let tgt_true = exits[0];
    let tgt_false = exits[1];
    if let Some(tgt_false) = parent.elem_avail(tgt_false) {
      (Some(tgt_false), Jump::CondTargetTrue(tgt_true))
    } else if let Some(tgt_true) = parent.elem_avail(tgt_true) {
      (Some(tgt_true), Jump::CondTargetFalse(tgt_false))
    } else {
      (None, Jump::CondTargetBoth(tgt_true, tgt_false))
    }
  } else {
    // JmpTbl will end up here..
    (None, Jump::Table(exits.clone()))
  };

  elem.jump = Some(jump);
  next
}

#[must_use]
fn schedule_layout_goto(elem: &mut Elem, parent: &Parent, _data: &mut ControlFlowData) -> Option<ElemId> {
  let Detail::Goto(goto) = &elem.detail else { panic!("Expected goto") };
  let next = parent.elem_avail(goto.target);
  elem.jump = Some(Jump::UncondTarget(goto.target));
  next
}

#[must_use]
fn schedule_layout_elem_block(elem: &mut Elem, parent: &Parent, data: &mut ControlFlowData) -> Option<ElemId> {
  let Detail::ElemBlock(blk) = &mut elem.detail else { panic!("Expected elem block") };

  let next = schedule_layout_body(&mut blk.body, Some(parent), data);

  elem.jump = Some(Jump::None);
  next
}

#[must_use]
fn schedule_layout_loop(elem: &mut Elem, parent: &Parent, data: &mut ControlFlowData) -> Option<ElemId> {
  let Detail::Loop(lp) = &mut elem.detail else { panic!("Expected loop") };
  let _ = schedule_layout_body(&mut lp.body, None, data);
  elem.jump = Some(Jump::None);
  for exit in elem.exits.iter().cloned() {
    if let Some(exit) = parent.elem_avail(exit) {
      return Some(exit);
    }
  }
  None
}

#[must_use]
fn schedule_layout_ifstmt(elem: &mut Elem, parent: &Parent, data: &mut ControlFlowData) -> Option<ElemId> {
  let Detail::If(ifstmt) = &mut elem.detail else { panic!("Expected ifstmt") };

  // schedule then-body
  let then_next = schedule_layout_body(&mut ifstmt.then_body, Some(parent), data);

  // figure out next for the exit/join-block
  let (next, jump) = if let Some(exit) = parent.elem_avail(ifstmt.exit) {
    (Some(exit), Jump::UncondFallthrough)
  } else {
    (None, Jump::UncondTarget(ifstmt.exit))
  };

  // By construction.. then-body and ifstmt should have made the same conclusion on "next"
  assert!(then_next == next);

  elem.jump = Some(jump);
  next
}

#[must_use]
fn schedule_layout_switch(elem: &mut Elem, parent: &Parent, data: &mut ControlFlowData) -> Option<ElemId> {
  let Detail::Switch(sw) = &mut elem.detail else { panic!("Expected switch") };
  for id in &sw.cases {
    sw.body.layout.push(*id);
    let _ = schedule_layout_elem(*id, parent, data);
  }
  elem.jump = Some(Jump::None);
  for exit in elem.exits.iter().cloned() {
    if let Some(exit) = parent.elem_avail(exit) {
      return Some(exit);
    }
  }
  None
}

#[must_use]
fn schedule_layout_elem(id: ElemId, parent: &Parent, data: &mut ControlFlowData) -> Option<ElemId> {
  let mut elem = data.checkout(id);
  //println!("==============================================");
  //debug_dump_elem(id, &elem, data);
  let next = match &elem.detail {
    Detail::BasicBlock(_) => schedule_layout_basic_block(&mut elem, &parent, data),
    Detail::Goto(_) => schedule_layout_goto(&mut elem, &parent, data),
    Detail::ElemBlock(_) => schedule_layout_elem_block(&mut elem, &parent, data),
    Detail::Loop(_) => schedule_layout_loop(&mut elem, &parent, data),
    Detail::If(_) => schedule_layout_ifstmt(&mut elem, &parent, data),
    Detail::Switch(_) => schedule_layout_switch(&mut elem, &parent, data),
    // _ => panic!("Unknown detail type: {:?}", elem.detail),
  };
  data.release(id, elem);
  next
}

#[must_use]
fn schedule_layout_body(body: &mut Body, parent: Option<&Parent>, data: &mut ControlFlowData)  -> Option<ElemId> {
  let mut remaining = body.elems.clone();
  let mut next = Some(body.lookup_from_id(body.entry).unwrap());
  while remaining.len() > 0 {
    let cur = match next {
      Some(cur) => cur,
      None => {
        // Select a block from remaining: first in blocknum ordering
        itertools::sorted(remaining.iter()).cloned().next().unwrap()
      }
    };

    if !remaining.remove(&cur) { panic!("tried to schedule an unavailable block"); }
    body.layout.push(cur);

    let parent = Parent {
      body, remain: &remaining, next: Box::new(parent),
    };
    next = schedule_layout_elem(cur, &parent, data);
  }
  next
}

fn label_blocks(cf: &mut ControlFlow) {
  if LABEL_BLOCKS_ALWAYS {
    label_blocks_always(cf);
  } else {
    label_blocks_by_demand(cf);
  }
}

fn label_blocks_always(cf: &mut ControlFlow) {
  for i in 0..cf.data.len() {
    let Some(elem) = &mut cf.data.0[i] else { continue };
    let Detail::BasicBlock(bb) = &mut elem.detail else { continue };
    bb.labeled = true;
  }
}

fn label_blocks_by_demand(cf: &mut ControlFlow) {
  // Two phase to avoid an immutable ref <=> mutable ref collision

  // Phase 1: Iterate the full controlflow, collecting all jump targets
  let mut targets =  HashSet::new();
  for elt in cf.iter() {
    match elt.elem.jump.as_ref().unwrap() {
      Jump::None => (),
      Jump::UncondFallthrough => (),
      Jump::UncondTarget(tgt) => { targets.insert(*tgt); }
      Jump::CondTargetTrue(tgt) => { targets.insert(*tgt); }
      Jump::CondTargetFalse(tgt) => { targets.insert(*tgt); }
      Jump::CondTargetBoth(tgt_true, tgt_false) => {
        targets.insert(*tgt_true);
        targets.insert(*tgt_false);
      }
      Jump::Table(tgts) => {
        for tgt in tgts {
          targets.insert(*tgt);
        }
      }
      //Jump::Continue => (),
    }
  }

  // Phase 2: Label all targetted blocks
  for tgt in targets {
    let elem = cf.data.get_mut(tgt);
    let Detail::BasicBlock(bb) = &mut elem.detail else { panic!("Expected basic block for labeling") };
    bb.labeled = true;
  }
}


pub fn format(cf: &ControlFlow) -> Result<String, std::fmt::Error> {
  let mut buf = String::new();
  let f = &mut buf;
  for elt in cf.iter() {
    debug_dump_elem_impl(f, elt.id, elt.elem, elt.cfdata, elt.depth, false)?;
  }
  Ok(buf)
}

pub fn debug_dump_elem(id: ElemId, elem: &Elem, data: &ControlFlowData) {
  let mut buf = String::new();
  debug_dump_elem_impl(&mut buf, id, elem, data, 0, true).unwrap();
  println!("{}", buf);
}

pub fn debug_dump_elem_body(f: &mut dyn Write, body: &Body, data: &ControlFlowData, indent: usize, recurse: bool) -> Result<(), std::fmt::Error> {
  for id in itertools::sorted(body.elems.iter()) {
    debug_dump_elem_impl(f, *id, data.get(*id), data, indent+1, recurse)?;
  }
  Ok(())
}

pub fn debug_dump_elem_impl(f: &mut dyn Write, id: ElemId, elem: &Elem, data: &ControlFlowData, indent: usize, recurse: bool) -> Result<(), std::fmt::Error> {
  write!(f, "{:indent$}{:?} | ", "", id, indent=2*indent)?;
  let exits: Vec<_> = elem.exits.iter().map(|x| x.0).collect();
  match &elem.detail {
    Detail::BasicBlock(b) => writeln!(f, "BasicBlock({})", b.blkref.0)?,
    Detail::Loop(lp) => {
      let backedges: Vec<_> = lp.backedges.iter().cloned().map(|x| x.0).collect();
      writeln!(f, "Loop [entry={}, exits={:?}, backedges={:?}]", elem.entry.0, exits, backedges)?;
      if recurse { debug_dump_elem_body(f, &lp.body, data, indent, recurse)?; }
    }
    Detail::If(ifstmt) => {
      writeln!(f, "If [entry={}, exits={:?}]", elem.entry.0, exits)?;
      if recurse { debug_dump_elem_body(f, &ifstmt.then_body, data, indent, recurse)?; }
    }
    Detail::Switch(sw) => {
      writeln!(f, "Switch [entry={}, exits={:?}]", elem.entry.0, exits)?;
      if recurse { debug_dump_elem_body(f, &sw.body, data, indent, recurse)?; }
    }
    Detail::Goto(goto) => {
      writeln!(f, "Goto [entry={}, exits={:?}] target={}", elem.entry.0, exits, goto.target.0)?;
    }
    Detail::ElemBlock(blk) => {
      writeln!(f, "ElemBlock [entry={}, exits={:?}]", elem.entry.0, exits)?;
      if recurse { debug_dump_elem_body(f, &blk.body, data, indent, recurse)?; }
    }
    //_ => panic!("Unknown detail type: {:?}", elem.detail),
  }
  Ok(())
}
