const datastore = require("@nucleoidjs/datastore");
const { argv } = require("yargs");
const runtime = require("./runtime");
const state = require("./state");
const config = require("./config");
const fs = require("fs");
const uuid = require("uuid").v4;

let _options = {
  declarative: false,
  details: false,
  cacheOnly: argv.cacheOnly || false,
};

function init(options) {
  if (options.test) {
    options.cacheOnly = true;
    options.terminal = false;
  }

  _options = options;

  if (options.test) {
    return _options;
  }

  let id = argv.id;

  if (!id) {
    try {
      id = fs.readFileSync(`${config.path.root}/default`, "utf8").trim();
    } catch (err) {
      id = uuid();
      fs.writeFileSync(`${config.path.root}/default`, id);
    }
  }

  datastore.init({ id, path: config.path.data });

  setImmediate(() => {
    datastore.read().forEach((details) => {
      const options = {
        declarative: !!details.c,
        cacheOnly: true,
      };

      if (!details.e) {
        runtime.process(details.s, options);
      }

      if (details.x) {
        details.x.map((exec) => state.run(null, exec));
      }
    });
  });

  return _options;
}

module.exports.init = init;
module.exports.options = () => _options;
