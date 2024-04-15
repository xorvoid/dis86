use pico_args;
use crate::binary::{self, Binary};
use crate::segoff::SegOff;
use crate::asm::decode::Decoder;
use crate::asm::intel_syntax;
use crate::ir::{self, opt, sym, fuse};
use crate::config::Config;
use crate::gen;
use crate::ast;
use crate::control_flow;
use crate::spec;
use std::fs::File;
use std::io::Write;

fn print_help() {
  let appname = std::env::args().next().unwrap();
  println!("usage: {} OPTIONS", appname);
  println!("");
  println!("REQUIRED OPTIONS:");
  println!("  --config          path to binary configuration file (required)");
  println!("  --binary-exe      path to MZ format exe on the filesystem (exactly 1 --binary-* flag required)");
  println!("  --binary-raw      path to raw binary on the filesystem (exactly 1 --binary-* flag required)");
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
  println!("  --emit-ir-presym  path to emit pre-symbolized SSA IR (optional)");
  println!("  --emit-ir-sym     path to emit symbolized SSA IR (optional)");
  println!("  --emit-ir-fwd     path to emit memory forwarding SSA IR (optional)");
  println!("  --emit-ir-opt     path to emit optimized SSA IR (optional)");
  println!("  --emit-ir-final   path to emit final SSA IR before control-flow analysis (optional)");
  println!("  --emit-graph      path to emit a control-flow-graph dot file (optional)");
  println!("  --emit-ctrlflow   path to emit the inferred control-flow structure (optional)");
  println!("  --emit-ast        path to emit the constructed AST (optional)");
  println!("  --emit-code       path to emit c code (optional)");
  println!("");
  println!("IR BUILD FLAGS:");
  println!("  --build-pin-all");
  println!("");
  println!("CODEGEN FLAGS:");
  println!("  --codegen-hydra   emit code that integrates well with the hydra runtime (optional)");
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
  binary: binary::Fmt,
  config: String,

  start_addr: Option<SegOff>,
  end_addr: Option<SegOff>,
  name: Option<String>,

  emit_dis: Option<String>,
  emit_ir_initial: Option<String>,
  emit_ir_presym: Option<String>,
  emit_ir_sym: Option<String>,
  emit_ir_fwd: Option<String>,
  emit_ir_opt: Option<String>,
  emit_ir_final: Option<String>,
  emit_graph: Option<String>,
  emit_ctrlflow: Option<String>,
  emit_ast: Option<String>,
  emit_code: Option<String>,

  build_pin_all: bool,
  codegen_hydra: bool,
}

fn match_flag(args: &mut Vec<std::ffi::OsString>, flag: &str) -> bool {
  for i in 0..args.len() {
    if args[i] == flag {
      args.remove(i);
      return true;
    }
  }
  false
}

fn parse_binary_fmt(pargs: &mut pico_args::Arguments) -> Result<binary::Fmt, pico_args::Error> {
  let binary_exe = pargs.opt_value_from_str("--binary-exe")?;
  let binary_raw = pargs.opt_value_from_str("--binary-raw")?;
  if 1 != binary_exe.is_some() as i32 + binary_raw.is_some() as i32 {
    panic!("Exactly one of --binary-exe or --binary-raw must be set");
  }
  if let Some(path) = binary_exe {
    return Ok(binary::Fmt::Exe(path));
  }
  if let Some(path) = binary_raw {
    return Ok(binary::Fmt::Raw(path));
  }
  unreachable!();
}

fn parse_args() -> Result<Args, pico_args::Error> {
  let mut pargs = pico_args::Arguments::from_env();

  // Help has a higher priority and should be handled separately.
  if pargs.contains(["-h", "--help"]) {
    print_help();
    std::process::exit(0);
  }

  // Parse out all args
  let mut args = Args {
    config:          pargs.value_from_str("--config")?,
    binary:          parse_binary_fmt(&mut pargs)?,
    start_addr:      pargs.opt_value_from_str("--start-addr")?,
    end_addr:        pargs.opt_value_from_str("--end-addr")?,
    name:            pargs.opt_value_from_str("--name")?,
    emit_dis:        pargs.opt_value_from_str("--emit-dis")?,
    emit_ir_initial: pargs.opt_value_from_str("--emit-ir-initial")?,
    emit_ir_opt:     pargs.opt_value_from_str("--emit-ir-opt")?,
    emit_ir_presym:  pargs.opt_value_from_str("--emit-ir-presym")?,
    emit_ir_sym:     pargs.opt_value_from_str("--emit-ir-sym")?,
    emit_ir_fwd:     pargs.opt_value_from_str("--emit-ir-fwd")?,
    emit_ir_final:   pargs.opt_value_from_str("--emit-ir-final")?,
    emit_graph:      pargs.opt_value_from_str("--emit-graph")?,
    emit_ctrlflow:   pargs.opt_value_from_str("--emit-ctrlflow")?,
    emit_ast:        pargs.opt_value_from_str("--emit-ast")?,
    emit_code:       pargs.opt_value_from_str("--emit-code")?,
    build_pin_all:   false,
    codegen_hydra:   false,
  };

  let mut remaining = pargs.finish();
  args.build_pin_all = match_flag(&mut remaining, "--build-pin-all");
  args.codegen_hydra = match_flag(&mut remaining, "--codegen-hydra");

  // It's up to the caller what to do with the remaining arguments.
  if remaining.len() != 0 {
    eprintln!("Error: unused arguments left: {:?}.", remaining);
    std::process::exit(1);
  }

  Ok(args)
}

pub fn run() -> i32 {
  let args = match parse_args() {
    Ok(v) => v,
    Err(e) => {
      eprintln!("Error: {}.", e);
      return 1;
    }
  };

  let cfg = Config::from_path(&args.config).unwrap();

  let spec = if let Some(name) = args.name.as_ref() {
    spec::Spec::from_config_name(&cfg, name)
  } else {
    spec::Spec::from_start_and_end(args.start_addr, args.end_addr)
  };

  let binary = Binary::from_fmt(&args.binary, Some(&cfg)).unwrap();

  let region = binary.region_iter(spec.start, spec.end);
  let decoder = Decoder::new(region);
  let mut instr_list = vec![];
  let mut raw_list = vec![];
  for (instr, raw) in decoder {
    instr_list.push(instr);
    raw_list.push(raw);
  }
  // FIXME: SIMPLIFY!!
  if let Some(path) = args.emit_dis.as_ref() {
    let mut buf = String::new();
    for i in 0..instr_list.len() {
      let instr = &instr_list[i];
      let raw = &raw_list[i];
      buf += &intel_syntax::format(instr.addr, Some(&instr), raw, true).unwrap();
      buf += "\n";
    }
    write_to_path(path, &buf);
    return 0;
  }

  let overlay = spec.start.is_overlay_addr();

  let mut ir = ir::IR::from_instrs(&instr_list, &cfg, &spec, &binary, overlay, args.build_pin_all);
  if let Some(path) = args.emit_ir_initial.as_ref() {
    write_to_path(path, &format!("{}", ir));
    return 0;
  }

  opt::optimize(&mut ir);
  if let Some(path) = args.emit_ir_presym.as_ref() {
    write_to_path(path, &format!("{}", ir));
    return 0;
  }

  sym::symbolize(&mut ir, &cfg);
  if let Some(path) = args.emit_ir_sym.as_ref() {
    write_to_path(path, &format!("{}", ir));
    return 0;
  }

  opt::forward_store_to_load(&mut ir);
  opt::optimize(&mut ir);
  if let Some(path) = args.emit_ir_fwd.as_ref() {
    write_to_path(path, &format!("{}", ir));
    return 0;
  }

  opt::mem_symbol_to_ref(&mut ir);
  opt::optimize(&mut ir);
  if let Some(path) = args.emit_ir_opt.as_ref() {
    let text = ir::display::display_ir_with_uses(&ir).unwrap();
    write_to_path(path, &format!("{}", &text));
    return 0;
  }

  fuse::fuse_mem(&mut ir);
  opt::optimize(&mut ir);

  ir::fin::finalize(&mut ir);
  if let Some(path) = args.emit_ir_final.as_ref() {
    let text = ir::display::display_ir_with_uses(&ir).unwrap();
    write_to_path(path, &format!("{}", &text));
    return 0;
  }

  if let Some(path) = args.emit_graph.as_ref() {
    let dot = ir::util::gen_graphviz_dotfile(&ir).unwrap();
    write_to_path(path, &dot);
    return 0;
  }

  let ctrlflow = control_flow::ControlFlow::from_ir(&ir);
  if let Some(path) = args.emit_ctrlflow.as_ref() {
    let text = control_flow::format(&ctrlflow).unwrap();
    write_to_path(path, &text);
    return 0;
  }

  let ret = spec.func.map(|f| f.ret.clone());
  let ast = ast::Function::from_ir(&cfg, &spec.name, ret, &ir, &ctrlflow);
  if let Some(path) = args.emit_ast.as_ref() {
    let text = format!("{:#?}", ast);
    write_to_path(path, &text);
    return 0;
  }

  if let Some(path) = args.emit_code.as_ref() {
    let flavor = if args.codegen_hydra { gen::Flavor::Hydra } else { gen::Flavor::Standard };
    let code = gen::generate(&ast, flavor).unwrap();
    write_to_path(path, &code);
    return 0;
  }

  return 0;
}
