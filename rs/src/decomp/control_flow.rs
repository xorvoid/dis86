use crate::decomp::ir;
use std::fs::File;
use std::io::Write;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug)]
pub struct Structure {
  top: Body,
}

#[derive(Debug)]
struct Body {
  adhoc: HashSet<usize>,
  structured: Vec<Element>,
}

#[derive(Debug)]
enum Element {
  Loop(Loop),
  If(If),
}

#[derive(Debug)]
struct Loop {
  loop_hdr: usize,
  back_edges: HashSet<usize>,
  exits: HashSet<usize>,
  body: Body,
}

#[derive(Debug)]
struct If {
  if_hdr: usize,
  join: usize,
  if_body: Body,
  else_body: Body,
}

impl Loop {
  fn new(loop_hdr: usize) -> Self {
    Self {
      loop_hdr,
      back_edges: HashSet::new(),
      exits: HashSet::new(),
      body: Body::new(),
    }
  }

  fn iter_blocks(&self) -> impl Iterator<Item = &usize> + '_ {
    self.body.adhoc.iter().chain(self.back_edges.iter())
  }

  fn contains_block(&self, b: usize) -> bool {
    self.body.adhoc.get(&b).is_some() || self.back_edges.get(&b).is_some()
  }
}

impl If {
  fn new(if_hdr: usize, join: usize) -> Self {
    Self {
      if_hdr,
      join,
      if_body: Body::new(),
      else_body: Body::new(),
    }
  }
}

struct AdjGraph {
  nodes: Vec<Vec<usize>>, // A list of children for each node
}

impl AdjGraph {
  fn from_ir(ir: &ir::IR) -> Self {
    let mut graph = AdjGraph { nodes: vec![] };
    for b in 0..ir.blocks.len() {
      let instr = ir.blocks[b].instrs.last().unwrap();
      let mut children = vec![];
      match instr.opcode {
        ir::Opcode::Ret => (),
        ir::Opcode::Jmp => {
          children.push(instr.operands[0].unwrap_block().0);
        }
        ir::Opcode::Jne => {
          children.push(instr.operands[1].unwrap_block().0);
          children.push(instr.operands[2].unwrap_block().0);
        }
        _ => panic!("Expected last instruction to be a branching instruction: {:?}", instr),
      }
      graph.nodes.push(children);
    }
    graph
  }

  fn search_for_loops(&self, entry: usize, allow_nodes: &HashSet<usize>) -> Vec<Loop> {
    // DFS search that terminates at back-edges and reveal the loop-headers
    let mut loops: HashMap<usize, Loop> = HashMap::new();

    // Sanity
    assert!(allow_nodes.get(&entry).is_some());

    let mut visited = vec![];
    visited.resize(self.nodes.len(), false);

    let mut path = vec![]; // Element: (node_idx, next_child_idx)

    path.push((entry, 0)); // Entry node and first child
    visited[0] = true;

    while path.len() > 0 {
      let n = path.len() - 1;
      let (node_idx, next_child_idx) = &mut path[n];

      let node = &self.nodes[*node_idx];
      let child_idx = *next_child_idx;
      *next_child_idx += 1;

      // Reached dead-end? Need to backtrace?
      if child_idx >= node.len() {
        visited[*node_idx] = false;
        path.pop();
        continue;
      }

      // Child allowed?
      let child = node[child_idx];
      if allow_nodes.get(&child).is_none() {
        continue;
      }

      // Child not visited? Then follow it!
      if !visited[child] {
        visited[child] = true;
        path.push((child, 0));
        continue;
      }

      // Child was visited: we have a back-edge! Update the loop metadata
      // and then skip the child
      let loop_hdr = child;
      let back_edge = *node_idx;

      let lp = loops.entry(loop_hdr).or_insert_with(|| Loop::new(loop_hdr));
      lp.back_edges.insert(back_edge);

      // Record all blocks on the path inside the loop body
      let mut loop_hdr_found = false;
      for (node_idx, _) in &path {
        if *node_idx == loop_hdr {
          loop_hdr_found = true;
        }
        if loop_hdr_found && lp.back_edges.get(node_idx).is_none() {
          lp.body.adhoc.insert(*node_idx);
        }
      }
    }

    // find all loopo exits
    let mut loops: Vec<Loop> = loops.into_values().collect();
    for lp in &mut loops {
      let mut exits = HashSet::new();
      for b in lp.iter_blocks() {
        for tgt in &self.nodes[*b] {
          if !lp.contains_block(*tgt) {
            exits.insert(*tgt);
          }
        }
      }
      lp.exits = exits;
    }

    loops
  }
}


impl Body {
  fn new() -> Self {
    Self {
      adhoc: HashSet::new(),
      structured: vec![],
    }
  }

  fn is_adhoc(&self, node: usize) -> bool {
    self.adhoc.get(&node).is_some()
  }

  fn children<'a>(&'a self, node: usize, graph: &'a AdjGraph) -> Vec<usize> {
    if self.adhoc.get(&node).is_some() {
      return graph.nodes[node].clone();
    } else {
      for elt in &self.structured {
        match elt {
          Element::Loop(lp) => {
            if lp.loop_hdr == node {
              return lp.exits.iter().cloned().collect();
            }
          }
          _ => (),
        }
      }
      panic!("Node not found: {}", node);
    }
  }

  fn infer_loops(&mut self, graph: &AdjGraph, root: usize) {
    if self.adhoc.len() == 0 {
      return;
    }
    let loops = graph.search_for_loops(root, &self.adhoc);
    for lp in loops {
      if !lp.body.adhoc.is_subset(&self.adhoc) {
        println!("Detected loop is not disjoint with other structures, skipping...");
        continue;
      }
      self.adhoc = self.adhoc.difference(&lp.body.adhoc).cloned().collect();
      self.adhoc = self.adhoc.difference(&lp.back_edges).cloned().collect();
      self.structured.push(Element::Loop(lp));
    }

    // Recurse to find nested loops
    for elt in &mut self.structured {
      match elt {
        Element::Loop(lp) => lp.body.infer_loops(graph, lp.loop_hdr),
        _ => (),
      }
    }

  }
}


fn is_if(mut then_block: usize, join_block: usize, body: &Body, graph: &AdjGraph) -> bool {
  if then_block == join_block { return false; } // WEIRD!
  let mut cur = then_block;
  while cur != join_block {
    let children = body.children(cur, graph);
    if children.len() != 1 { return false; }
    cur = children[0];
  }
  true
}

fn infer_ifs(body: &Body, entry: usize, graph: &AdjGraph) -> Vec<If> {
  let mut search_queue = VecDeque::new();
  search_queue.push_back(entry);

  let mut ifs = vec![];
  while let Some(node) = search_queue.pop_front() {
    println!("node: {}", node);
    let children = body.children(node, graph);
    // attempt to match an if-stmt?
    if body.is_adhoc(node) && children.len() == 2 {
      println!("Searching if on node={}", node);
      if is_if(children[0], children[1], body, graph) {
        println!("Detected if: {} -> ({} or {})", node, children[0], children[1]);
      } else if is_if(children[1], children[0], body, graph) {
        println!("Detected if: {} -> ({} or {})", node, children[0], children[1]);
      }
    }
    for child in children {
      search_queue.push_back(child);
    }
  }
  println!("cur: {:?}", search_queue);
  ifs
}

impl Structure {
  pub fn infer_from_ir(ir: &ir::IR) -> Self {
    // build an adjacency list graph from the IR CFG
    let graph = AdjGraph::from_ir(ir);

    // build top-level block with all adhoc
    let mut body = Body::new();
    for b in 0..ir.blocks.len() {
      body.adhoc.insert(b);
    }

    body.infer_loops(&graph, 0);
    infer_ifs(&body, 0, &graph);

    Structure { top: body }
  }
}

pub fn gen_graphviz_dotfile(path: &str, ir: &ir::IR) -> std::io::Result<()> {
  let mut f = File::create(path)?;
  writeln!(f, "digraph control_flow {{")?;
  for b in 0..ir.blocks.len() {
    let blk = &ir.blocks[b];
    let src = &blk.name;
    let instr = blk.instrs.last().unwrap();
    match instr.opcode {
      ir::Opcode::Ret => {
        writeln!(f, "  {}_{} -> exit;\n", src, b)?;
      }
      ir::Opcode::Jmp => {
        let tgt = instr.operands[0].unwrap_block().0;
        let tgt_name = &ir.blocks[tgt].name;
        writeln!(f, "  {}_{} -> {}_{};\n", src, b, tgt_name, tgt)?;
      }
      ir::Opcode::Jne => {
        let true_tgt = instr.operands[1].unwrap_block().0;
        let true_tgt_name = &ir.blocks[true_tgt].name;
        let false_tgt = instr.operands[2].unwrap_block().0;
        let false_tgt_name = &ir.blocks[false_tgt].name;
        writeln!(f, "  {}_{} -> {}_{};\n", src, b, true_tgt_name, true_tgt)?;
        writeln!(f, "  {}_{} -> {}_{};\n", src, b, false_tgt_name, false_tgt)?;
      }
      _ => panic!("Expected last instruction to be a branching instruction: {:?}", instr),
    }
  }
  writeln!(f, "}}\n")?;
  Ok(())
}
