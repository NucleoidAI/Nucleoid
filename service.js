const glob = require("glob");
const config = require("./src/config");
const { argv } = require("yargs");
const { existsSync } = require("fs");

const { fork, execSync } = require("child_process");
let processes = [];

let event, stream, storage;

try {
  event = require(`${config.path.handlers}/event.js`);
} catch (err) {} // eslint-disable-line no-empty

try {
  stream = require(`${config.path.handlers}/stream.js`);
} catch (err) {} // eslint-disable-line no-empty

try {
  storage = require(`${config.path.handlers}/storage.js`);
} catch (err) {} // eslint-disable-line no-empty

const accept = (statement, req, res) => {
  // TODO Deprecate main process
  let proc = req.get("Process") || "main";
  let files = glob.sync(proc, { cwd: config.path.data });

  if (files.length <= 1) {
    send(proc, statement, req);
  } else {
    files.forEach((file) => {
      // Async Process
      send(file, statement);
    });

    res.status(202).send({ count: files.length });
  }
};

function start(id) {
  let proc = processes[id];

  if (!proc) {
    if (storage) {
      storage(id);
    }

    proc = { id, requests: [] };
    processes[id] = proc;
  }

  const options = [`--id=${id}`];
  if (argv.cacheOnly) options.push("--cache-only");

  let module;

  const local = ".";
  const source = "src/process.js";
  const global = execSync("npm root -g").toString().trim();

  let npx;

  try {
    npx = require.resolve("nucleoidjs").slice(0, -9);
  } catch (error) {
    npx = "./node_modules/nucleoidjs";
  }

  if (existsSync(`${local}/${source}`)) {
    module = `${local}/${source}`;
  } else if (existsSync(`${npx}/${source}`)) {
    module = `${npx}/${source}`;
  } else if (existsSync(`${global}/nucleoidjs/${source}`)) {
    module = `${global}/nucleoidjs/${source}`;
  } else {
    console.error("Nucleoid is not installed correctly");
    process.exit(-1);
  }

  proc.pid = fork(module, options);
  proc.pid.on("message", (m) => receive(proc, m));
  proc.pid.on("exit", () => {
    delete proc.pid;
  });

  return proc;
}

function send(id, statement, request) {
  statement = (statement || "").trim();

  let proc = processes[id];

  if (!proc || !proc.pid) {
    proc = start(id);
  }

  proc.requests.push({ statement, request });

  if (proc.requests.length === 1) {
    proc.pid.send(statement);
  }
}

function receive(proc, message) {
  let { request } = proc.requests.shift();

  if (request) {
    let details = JSON.parse(message);

    request.res.send({
      result: details.r,
      date: details.d,
      time: details.t,
      error: details.e,
      messages: details.m,
      events: details.v,
      hash: details.h,
    });

    if (details.m) {
      let messages = details.m;
      messages.forEach((m) => {
        send(m.process, {
          body: `let m={"pid":"${proc.id}","payload":${m.payload}};new Message(m)`,
          type: "MESSAGE",
        });
      });
    }

    if (details.v && event) {
      for (let e of details.v) {
        try {
          event(proc.id, e.name, e.payload);
        } catch (error) {
          error;
        }
      }
    }

    if (details.h && stream) {
      stream(details.h);
    }
  }

  if (proc.requests.length > 0) {
    let r = proc.requests[0];
    proc.pid.send(r.statement);
  }
}

module.exports.start = start;
module.exports.accept = accept;
module.exports.send = send;
