import sys

class FuncData:
    def __init__(self, func, name, entry):
        self.name = name
        self.ret = "u32" if func.ret is None else func.ret  ## assume worst case
        self.args = "IGNORE" if func.args is None or func.args < 0 else str(func.args)
        self.overlay = str(int(entry.overlay))
        self.seg = f'0x{entry.seg:04x}'
        self.off = f'0x{entry.off:04x}'
        self.flags = str(func.flags)

def build_func_data(functions):
    dat = []
    for func in functions:
        ## a call location, not a function location.. skip
        if func.flags == 'INDIRECT_CALL_LOCATION': continue
        if func.is_overlay_entry:
            dat.append(FuncData(func, func.name, func.entry_stub))
            dat.append(FuncData(func, func.name + "_OVERLAY", func.start_addr))
            continue
        dat.append(FuncData(func, func.name, func.start_addr))
    return dat

def gen_hdr(data, out=None):
    f = sys.stdout if not out else out
    def emit(s): print(s, file=f)

    functions = data['functions']
    func_data = build_func_data(functions)

    datasection = data['data_section']
    structures = data['structures']

    emit('#pragma once')
    emit('#include "hydra/hydra.h"')
    emit('#if __has_include ("hydra_user_defs.h")')
    emit('  #include "hydra_user_defs.h"')
    emit('#endif')
    emit('')

    ## TODO: We no longer need the C X-MACRO style metaprogramming
    ## We should remove it and just generate everything from python directly
    ## We kept it for now to make it easy to get python code gen up and running

    emit('/**************************************************************************************************************/')
    emit('/* Callstubs */')
    emit('/**************************************************************************************************************/')
    for func in func_data:
        addr = f'ADDR_MAKE_EXT({func.overlay}, {func.seg}, {func.off})'
        emit(f'HYDRA_DEFINE_CALLSTUB( {func.name+",":30} {func.ret+",":8} {func.args+",":10} {addr}, {func.flags:15} )')
    emit('')

    emit('/**************************************************************************************************************/')
    emit('/* IS_OVERLAY_ENTRY flags */')
    emit('/**************************************************************************************************************/')
    for func in functions:
        emit('#define IS_OVERLAY_ENTRY_' + func.name + ' ' + ('1' if func.is_overlay_entry else '0'))
    emit('')

    emit('/**************************************************************************************************************/')
    emit('/* Structures */')
    emit('/**************************************************************************************************************/')
    emit('')

    for struct in structures:
        emit(f'typedef struct {struct.struct_name()} {struct.name};')
        emit(f'struct __attribute__((packed)) {struct.struct_name()}')
        emit('{')
        for mbr in struct.members:
            mbr_entry = f'{mbr.typ.fmt_ctype_str(mbr.name)};'
            emit(f'  {mbr_entry:<40}  /* 0x{mbr.off:02x} */')
        emit('};')
        emit(f'static_assert(sizeof({struct.name}) == {struct.size}, "");');
        emit('')

    emit('/**************************************************************************************************************/')
    emit('/* Data Section Globals */')
    emit('/**************************************************************************************************************/')
    emit('')

    for var in datasection:
        cast = f'({var.typ.basetype}*)'
        start = cast if var.typ.is_array else '*'+cast
        end = ''
        if var.typ.is_array:
            end = f' /* array: {var.typ} */'
        emit(f'#define {var.name:30} ({start:15} (hydra_datasection_baseptr() + 0x{var.off:04x})){end}')
    emit('')

    emit('/**************************************************************************************************************/')
    emit('/* Hook Registration */')
    emit('/**************************************************************************************************************/')
    emit('static inline void hydra_user_appdata__register_all_hooks(void) {')
    for func in functions:
        if not func.reimpl: continue
        name = func.name
        if name.startswith('F_'):
            name = name[2:]
        emit(f'  extern HYDRA_FUNC({"H_"+name});')
        emit(f'  HYDRA_REGISTER({name});')
    emit('}')
    emit('')

def gen_src(data, out=None):
    f = sys.stdout if not out else out
    def emit(s): print(s, file=f)

    functions = data['functions']
    func_data = build_func_data(functions)

    emit('#include "hydra_user_appdata.h"')
    emit('')
    emit('/**************************************************************************************************************/')
    emit('/* Generate Function Metdata */')
    emit('/**************************************************************************************************************/')
    emit('')
    emit('static hydra_function_def_t metadata[] = {')
    #emit('    HYDRA_FUNCTION_DEFINITIONS(DEFINE_FUNCDEF)')
    for func in func_data:
        #emit(f'    DEFINE_FUNCDEF( {func.name+",":30} {func.ret+",":8} {func.args+",":10} {func.seg+",":7} {func.off+",":7} {func.flags:15} )')
        name = f'"{func.name}",'
        emit(f'  {{ {name:30} {{{{ {func.overlay}, {func.seg}, {func.off} }}}} }},')
    emit('};')
    emit('')
    emit('const hydra_function_metadata_t * hydra_user_functions(void)')
    emit('{')
    emit('  static hydra_function_metadata_t md[1];')
    emit('  md->n_defs = sizeof(metadata)/sizeof(metadata[0]);')
    emit('  md->defs = metadata;')
    emit('')
    emit('  return md;')
    emit('}')
    emit('')
    emit('const hydra_callstack_metadata_t * hydra_user_callstack(void)')
    emit('{')
    emit('  static hydra_callstack_conf_t confs[] = {')
    for conf in data['callstack']:
        type_str = f'HYDRA_CALLSTACK_CONF_TYPE_{conf.typ},'
        name_str = f'"{conf.name}",'
        emit(f'    {{ {type_str:<40} {name_str:<25} {{{{ 0, 0x{conf.addr.seg:04x}, 0x{conf.addr.off:04x} }}}} }},')
    emit('  };')
    emit('')
    emit('  static hydra_callstack_metadata_t md[1];')
    emit('  md->n_confs = sizeof(confs)/sizeof(confs[0]);')
    emit('  md->confs = confs;')
    emit('')
    emit('  return md;')
    emit('}')
    emit('')
