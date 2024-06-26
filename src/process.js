const datastore = require("./datastore");
const config = require("./config");
const openapi = require("./lib/openapi");
const context = require("./context");
const terminal = require("./routes/terminal");
const stack = require("./stack");

let initialized = false;

function init() {
  if (initialized) {
    return;
  }

  const _config = config();

  datastore.init(_config);
  console.log("[✓] Data Store is initiated");

  setImmediate(() => {
    datastore.read().forEach(({ $, c, e }) => {
      if (!e && $) {
        stack.process($, null, { declarative: c });
      }
    });

    console.log("[✓] Context is loaded");

    const nucleoid = require("../");
    require(`${_config.path}/nucleoid.js`)(nucleoid);

    try {
      const event = require(`${_config.path}/extensions/event.js`);
      event.init();
    } catch (err) {} // eslint-disable-line no-empty

    const { native, port, test } = _config;

    if (native) {
      try {
        for (const route of native.routes) {
          const native = require(`${_config.path}/native/${route}`);
          terminal.use(native);
        }
      } catch (err) {
        console.error("Failed to load native routes");
        throw err;
      }
    }

    try {
      const {
        api,
        types,
        prefix,
        port,
        functions,
        events,
      } = require(`${_config.path}/openapi.json`);

      if (functions) {
        context.run(functions);
      }

      openapi.init();
      openapi.load({ api, types, prefix, events });
      openapi.start(port || _config.port.openapi);
    } catch (err) {
      if (err.code !== "MODULE_NOT_FOUND") {
        console.error("Error loading OpenAPI", err);
        process.exit(1);
      }
    }

    console.log("[✓] Process is running");

    if (!test) {
      terminal.all("*", (req, res) => res.status(404).end());
      terminal.listen(port.terminal);
      console.log("[✓] Terminal is ready");
    }
  });

  initialized = true;
}

module.exports.init = init;
