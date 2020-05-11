const argv = require("yargs").argv;
const express = require("express");
var bodyParser = require("body-parser");
var jwt = require("jsonwebtoken");
var fs = require("fs");

const app = express();
app.use(bodyParser.text({ type: "*/*" }));

const fork = require("child_process").fork;
var processes = [];

let pid = fork("./process.js", ["--id=global", "--path=/var/lib/nucleoid/"]);
let proc = { pid, id: "global", requests: [] };
pid.on("message", m => receive(proc, m));
processes["global"] = proc;

app.post("/", (req, res) => {
  let authorization = req.get("Authorization");
  let processHeader = req.get("Process");

  if (authorization !== undefined || processHeader !== undefined) {
    req.type = "REQUEST";

    let key;

    if (processHeader) {
      key = processHeader;
    } else {
      let parts = authorization.split(" ");
      let payload = jwt.decode(parts[1]);

      key = payload.username;
    }

    let proc = processes[key];

    if (proc === undefined) {
      let pid = fork("./process.js", [
        `--id=${key}`,
        "--path=/var/lib/nucleoid/"
      ]);

      proc = { pid, id: key, requests: [req] };
      processes[key] = proc;

      proc.pid.on("message", m => receive(proc, m));

      proc.pid.send(req.body);
    } else {
      proc.requests.push(req);

      if (proc.requests.length === 1) {
        proc.pid.send(req.body);
      }
    }
  } else {
    for (let p in processes) {
      if (p === "global") continue;

      let proc = processes[p];
      proc.requests.push(req);

      if (proc.requests.length === 1) {
        proc.pid.send(req.body);
      }
    }

    fs.appendFileSync(
      "/var/lib/nucleoid/process",
      JSON.stringify({ s: req.body, d: Date.now() }) + "\n"
    );
    res.status(202).end();
  }
});

app.listen(argv.port ? argv.port : 80);

function receive(proc, message) {
  let request = proc.requests.shift();

  if (request.type === "REQUEST") {
    request.res.type("application/json");
    let details = JSON.parse(message);

    if (details.e) {
      request.res.status(400);
    }

    if (details.r !== undefined) {
      request.res.send(details.r);
    } else {
      request.res.end();
    }

    if (details.m) {
      let messages = details.m;
      messages.forEach(m => {
        let p = processes[m.process];
        p.requests.push({
          body: `let m={"pid":"${proc.id}","payload":${
            m.payload
          }};new Message(m)`,
          type: "MESSAGE"
        });

        if (p.requests.length > 0) {
          let r = p.requests[0];
          p.pid.send(r.body);
        }
      });
    }
  }

  if (proc.requests.length > 0) {
    let r = proc.requests[0];
    proc.pid.send(r.body);
  }
}
