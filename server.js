const express = require("express");
var bodyParser = require("body-parser");
const fs = require("fs");
var glob = require("glob");

const app = express();
app.use(bodyParser.text({ type: "*/*" }));

const config = JSON.parse(fs.readFileSync("/etc/nucleoid/config.json", "utf8"));

if (config.process) {
  var fn = require(`/opt/nucleoid/${config.process}`);
}

if (config.authorization) {
  var authorization = require(`/opt/nucleoid/${config.authorization}`);
}

if (config.event) {
  var event = require(`/opt/nucleoid/${config.event}`);
}

const fork = require("child_process").fork;
var processes = [];

start("main");

if (config.ide) {
  require(config.ide)(app);
} else {
  app.get("/", (req, res) => {
    res.sendFile(`${__dirname}/terminal.html`);
  });
}

app.get("/processes", (req, res) => {
  let tree = [];

  let group = req.query.group;

  if (group === undefined) {
    group = "";
  }

  fs.readdirSync(`/var/lib/nucleoid/${group}`).forEach((file) => {
    if (fs.lstatSync(`/var/lib/nucleoid/${group}/${file}`).isDirectory()) {
      if (file !== "init") tree.push({ id: file, type: "group" });
    } else {
      tree.push({ id: file, type: "process" });
    }
  });

  res.send(tree);
});

app.post("/", (req, res) => {
  let processId = req.get("Process");
  let processIds = req.get("Processes");

  if (processId && processIds) {
    res.status(422).end();
    return;
  }

  if (!processId && !processIds) {
    if (config.process) {
      processId = fn(req, res);
    } else {
      processId = "main";
    }
  }

  if (config.authorization) {
    try {
      let expression = processId ? processId : processIds;
      let valid = authorization(expression, req);

      if (valid !== true) {
        throw { status: 403, message: "Forbidden" };
      }
    } catch (e) {
      res.status(e.status).send({ message: e.message });
      return;
    }
  }

  let path = `/var/lib/nucleoid/`;

  if (processId) {
    if (
      fs.existsSync(`${path}${processId}`) &&
      fs.lstatSync(`${path}${processId}`).isDirectory()
    ) {
      req.type = "ASYNC";

      fs.readdirSync(path).forEach((file) => {
        send(`${processId}/${file}`, req);
      });

      fs.appendFileSync(
        `/var/lib/nucleoid/init/${processId}`,
        JSON.stringify({ s: req.body, d: Date.now() }) + "\n"
      );

      res.status(202).end();
    } else {
      req.type = "SYNC";
      send(processId, req);
    }
  } else {
    let files = glob.sync(processIds, { cwd: path });

    req.type = "ASYNC";
    files.forEach((file) => {
      send(file, req);
    });

    res.status(202).send({ count: files.length });
  }
});

app.listen(config.port ? config.port : 80);

function start(id) {
  let proc = processes[id];

  if (!proc) {
    proc = { id, requests: [] };
    processes[id] = proc;
  }

  proc.pid = fork("./process.js", [`--id=${id}`, "--path=/var/lib/nucleoid/"]);
  proc.pid.on("message", (m) => receive(proc, m));
  proc.pid.on("exit", () => {
    delete proc.pid;
  });

  return proc;
}

function send(id, request) {
  let proc = processes[id];

  if (!proc || !proc.pid) {
    proc = start(id);
  }

  proc.requests.push(request);

  if (proc.requests.length === 1) {
    proc.pid.send(request.body);
  }
}

function receive(proc, message) {
  let request = proc.requests.shift();

  if (request.type === "SYNC") {
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
        } catch (e) {
          e;
        }
      }
    }
  }

  if (proc.requests.length > 0) {
    let r = proc.requests[0];
    proc.pid.send(r.body);
  }
}

app.use(function (err, res) {
  res.type("txt");
  res.status(500).send(err.stack);
});
