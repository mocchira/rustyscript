import * as console from 'ext:deno_console/01_console.js';

import { applyToGlobal, nonEnumerable } from 'ext:rustyscript/rustyscript.js';
applyToGlobal({
    console: nonEnumerable(
      new console.Console((msg, _level) => {
          // TODO(ysh)
          // This is a temporary solution to get rid of non-JSON output from log
          rustyscript.functions['console.log'](msg);
      }),
    ),
});