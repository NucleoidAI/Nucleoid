const datastore = require("@nucleoidjs/datastore");
const { argv } = require("yargs");
const runtime = require("./runtime");
const state = require("./state");
const config = require("./config");
const lockfile = require("lockfile");
const fs = require("fs");
const uuid = require("uuid").v4;

let id = argv.id;

if (!id) {
  try {
    id = fs.readFileSync(`${config.path.root}/default`, "utf8").trim();
  } catch (err) {
    id = uuid();
    fs.writeFileSync(`${config.path.root}/default`, id);
  }
}

try {
  lockfile.lockSync(`${config.path.root}/${id}.lock`);
} catch (e) {
  console.error("Another Nucleoid process is already running");
  process.exit(1);
}

let _options = {
  id,
  declarative: false,
  details: false,
  cacheOnly: argv.cacheOnly || false,
  port: argv.port || 8448,
};

datastore.init({ id, path: config.path.data });

setImmediate(() => {
  if (_options.test) return;

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

function options(options) {
  if (options) {
    _options = { ..._options, ...options };

    if (_options.test) {
      _options.cacheOnly = true;
      _options.terminal = false;
    }

    return _options;
  } else return _options;
}

module.exports.options = options;
