#pragma once
#include <stdint.h>

#include "../../src/dosbox-x/include/export/dosbox-x/hydra_machine.h"

#include "../../src/typedefs.h"
#include "../../src/addr.h"
#include "../../src/conf.h"
#include "../../src/dump.h"
#include "../../src/hooks.h"
#include "../../src/machine.h"
#include "../../src/overlay.h"
#include "../../src/functions.h"
#include "../../src/callstack.h"
#include "../../src/callstubs.h"

u8 * hydra_datasection_baseptr(void);
void hydra_callstack_dump(void);
