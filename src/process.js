const datastore = require("@nucleoidjs/datastore");
const runtime = require("./runtime");
const state = require("./state");
const config = require("./config");
const openapi = require("./lib/openapi");
const context = require("./lib/context");

function init() {
  const _config = config();
  datastore.init(_config);

  setImmediate(() => {
    datastore.read().forEach((details) => {
      const options = {
        declarative: !!details.c,
        cacheOnly: true,
      };

      if (details.w && !details.e) {
        runtime.process(details.s, options);

        if (details.j) {
          details.j.map((adjust) => state.run(null, adjust));
        }
      }
    });
  });
}

module.exports.init = init;
