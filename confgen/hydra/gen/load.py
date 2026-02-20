def annotations(path):
    import importlib.util
    import sys
    spec = importlib.util.spec_from_file_location("module.name", path)
    mod = importlib.util.module_from_spec(spec)
    sys.modules["module.name"] = mod
    spec.loader.exec_module(mod)

    return {
        'code_segments':   mod.CodeSegments,
        'functions':       mod.Functions,
        'structures':      mod.Structures,
        'data_section':    mod.DataSection,
        'text_section':    mod.TextSection,
        'callstack':       mod.Callstack,
    }
