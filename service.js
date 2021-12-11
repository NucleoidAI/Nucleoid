const glob = require("glob");
const File = require("./src/file");

const fork = require("child_process").fork;
let processes = [];

const config = File.config;
const path = File.data;

let event;

if (config.event) {
  event = require(`/opt/nucleoid/${config.event}`);
}

const accept = (statement, req, res) => {
  let proc = req.get("Process") || "main";
  let files = glob.sync(proc, { cwd: path });

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
    proc = { id, requests: [] };
    processes[id] = proc;
  }

  proc.pid = fork("./src/process.js", [`--id=${id}`]);
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
    request.res.type("application/json");
    let details = JSON.parse(message);

    if (details.e) {
      request.res.status(400);
    }

    if (details.t !== undefined) {
      request.res.set("Server-Timing", `nucleoid;dur=${details.t}`);
    }

    if (details.r !== undefined) {
      request.res.send(details.r);
    } else {
      request.res.end();
    }

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
  }

  if (proc.requests.length > 0) {
    let r = proc.requests[0];
    proc.pid.send(r.statement);
  }
}

module.exports.start = start;
module.exports.accept = accept;
module.exports.send = send;
