use pico_args;
use crate::intel_syntax;
use crate::segoff::SegOff;
use crate::decode::Decoder;
use crate::decomp::ir::{self, build, opt, sym};
use crate::decomp::config::Config;
use crate::decomp::gen;
use crate::decomp::ast;
use crate::decomp::control_flow;
use std::fs::File;
use std::io::Write;

fn print_help(appname: &str) {
  println!("usage: {} dis OPTIONS", appname);
  println!("");
  println!("REQUIRED OPTIONS:");
  println!("  --config          path to binary configuration file (required)");
  println!("  --binary          path to binary on the filesystem (required)");
  println!("");
  println!("MODE: ADDRESS RANGE");
  println!("  --start-addr      start seg:off address (maybe required)");
  println!("  --end-addr        end seg:off address (maybe required)");
  println!("");
  println!("MODE: FUNCTION NAME");
  println!("  --name            lookup address range by name in config (maybe required)");
  println!("");
  println!("EMIT MODES:");
  println!("  --emit-dis        path to emit disassembly (optional)");
  println!("  --emit-ir-initial path to emit initial unoptimized SSA IR (optional)");
  println!("  --emit-ir-sym     path to emit symbolized SSA IR (optional)");
  println!("  --emit-ir-opt     path to emit optimized SSA IR (optional)");
  println!("  --emit-ir-final   path to emit final SSA IR before control-flow analysis (optional)");
  println!("  --emit-graph      path to emit a control-flow-graph dot file (optional)");
  println!("  --emit-ctrlflow   path to emit the inferred control-flow structure (optional)");
  println!("  --emit-ast        path to emit the constructed AST (optional)");
  println!("  --emit-code       path to emit c code (optional)");
}

fn write_to_path(path: &str, data: &str) {
  if path.starts_with('-') {
    println!("{}", data);
  } else {
    let mut w = File::create(&path).unwrap();
    writeln!(&mut w, "{}", data).unwrap();
  }
}

#[derive(Debug)]
struct Args {
  binary: String,
  config: String,

  start_addr: Option<SegOff>,
  end_addr: Option<SegOff>,
  name: Option<String>,

  emit_dis: Option<String>,
  emit_ir_initial: Option<String>,
  emit_ir_sym: Option<String>,
  emit_ir_opt: Option<String>,
  emit_ir_final: Option<String>,
  emit_graph: Option<String>,
  emit_ctrlflow: Option<String>,
  emit_ast: Option<String>,
  emit_code: Option<String>,
}

fn parse_args(appname: &str) -> Result<Args, pico_args::Error> {
  let mut pargs = pico_args::Arguments::from_env();

  // Help has a higher priority and should be handled separately.
  if pargs.contains(["-h", "--help"]) {
    print_help(appname);
    std::process::exit(0);
  }

  // Parse out all args
  let args = Args {
    config:          pargs.value_from_str("--config")?,
    binary:          pargs.value_from_str("--binary")?,
    start_addr:      pargs.opt_value_from_str("--start-addr")?,
    end_addr:        pargs.opt_value_from_str("--end-addr")?,
    name:            pargs.opt_value_from_str("--name")?,
    emit_dis:        pargs.opt_value_from_str("--emit-dis")?,
    emit_ir_initial: pargs.opt_value_from_str("--emit-ir-initial")?,
    emit_ir_opt:     pargs.opt_value_from_str("--emit-ir-opt")?,
    emit_ir_sym:     pargs.opt_value_from_str("--emit-ir-sym")?,
    emit_ir_final:   pargs.opt_value_from_str("--emit-ir-final")?,
    emit_graph:      pargs.opt_value_from_str("--emit-graph")?,
    emit_ctrlflow:   pargs.opt_value_from_str("--emit-ctrlflow")?,
    emit_ast:        pargs.opt_value_from_str("--emit-ast")?,
    emit_code:       pargs.opt_value_from_str("--emit-code")?,
  };

  // It's up to the caller what to do with the remaining arguments.
  let remaining = pargs.finish();
  if remaining != &["decomp"] {
    eprintln!("Error: unused arguments left: {:?}.", remaining);
    std::process::exit(1);
  }

  Ok(args)
}

struct Func {
  name: String,
  start: SegOff,
  end: SegOff,
}

impl Func {
  fn from_config_name(cfg: &Config, name: &str) -> Func {
    let Some(func) = cfg.func_lookup_by_name(name) else {
      panic!("Failed to lookup function named: {}", name);
    };
    let Some(end) = func.end else {
      panic!("Function has no 'end' addr defined in config");
    };
    Func {
      name: name.to_string(),
      start: func.start,
      end,
    }
  }

  fn from_start_and_end(start: Option<SegOff>, end: Option<SegOff>) -> Func {
    let Some(start) = start else { panic!("No start address provided") };
    let Some(end) = end else { panic!("No end address provided") };
    let name = format!("func_{:08x}__{:04x}_{:04x}", start.abs(), start.seg, start.off);
    Func { name, start, end }
  }
}

pub fn run(appname: &str) {
  let args = match parse_args(appname) {
    Ok(v) => v,
    Err(e) => {
      eprintln!("Error: {}.", e);
      std::process::exit(1);
    }
  };

  let cfg = Config::from_path(&args.config).unwrap();

  let f = if let Some(name) = args.name.as_ref() {
    Func::from_config_name(&cfg, name)
  } else {
    Func::from_start_and_end(args.start_addr, args.end_addr)
  };
  let start_idx = f.start.abs();
  let end_idx = f.end.abs();

  let dat = match std::fs::read(&args.binary) {
    Ok(dat) => dat,
    Err(err) => panic!("Failed to read file: '{}': {:?}", args.binary, err),
  };

  let decoder = Decoder::new(&dat[start_idx..end_idx], start_idx);
  let mut instr_list = vec![];
  for (instr, raw) in decoder {
    // HAX WRONG
    if args.emit_dis.is_some() {
      println!("{}", intel_syntax::format(&instr, raw, true).unwrap());
    }
    instr_list.push(instr);
  }

  let mut ir = build::from_instrs(&instr_list, &cfg);
  if let Some(path) = args.emit_ir_initial.as_ref() {
    write_to_path(path, &format!("{}", ir));
  }

  opt::optimize(&mut ir);
  sym::symbolize(&mut ir, &cfg);

  if let Some(path) = args.emit_ir_sym.as_ref() {
    write_to_path(path, &format!("{}", ir));
  }

  opt::forward_store_to_load(&mut ir);
  opt::optimize(&mut ir);
  opt::mem_symbol_to_ref(&mut ir);
  opt::optimize(&mut ir);

  if let Some(path) = args.emit_ir_opt.as_ref() {
    let text = ir::display::display_ir_with_uses(&ir).unwrap();
    write_to_path(path, &format!("{}", &text));
  }

  ir::fin::finalize(&mut ir);
  if let Some(path) = args.emit_ir_final.as_ref() {
    let text = ir::display::display_ir_with_uses(&ir).unwrap();
    write_to_path(path, &format!("{}", &text));
  }

  if let Some(path) = args.emit_graph.as_ref() {
    let dot = ir::util::gen_graphviz_dotfile(&ir).unwrap();
    write_to_path(path, &dot);
  }

  let ctrlflow = control_flow::ControlFlow::from_ir(&ir);
  if let Some(path) = args.emit_ctrlflow.as_ref() {
    let text = control_flow::format(&ctrlflow).unwrap();
    write_to_path(path, &text);
  }

  let ast = ast::Function::from_ir(&f.name, &ir, &ctrlflow);
  if let Some(path) = args.emit_ast.as_ref() {
    let text = format!("{:#?}", ast);
    write_to_path(path, &text);
  }

  if let Some(path) = args.emit_code.as_ref() {
    let code = gen::generate(&ast).unwrap();
    write_to_path(path, &code);
  }
}
