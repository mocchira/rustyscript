import * as console from 'ext:deno_console/01_console.js';

import { applyToGlobal, nonEnumerable } from 'ext:rustyscript/rustyscript.js';
applyToGlobal({
    console: nonEnumerable(
      new console.Console((_msg, _level) => {
          // noop
          // TODO(ysh)
          // This is a temporary solution to get rid of non-JSON output from log
      }),
    ),
});