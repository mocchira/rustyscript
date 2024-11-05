
import * as net from "ext:deno_net/01_net.js";
import * as tls from "ext:deno_net/02_tls.js";

import {applyToGlobal, nonEnumerable} from 'ext:rustyscript/rustyscript.js';

applyToGlobal({
    net: nonEnumerable(net),
    tls: nonEnumerable(tls)
});