const express = require("express");
var bodyParser = require("body-parser");
const fs = require("fs");

const app = express();
app.use(bodyParser.text({ type: "*/*" }));

const config = JSON.parse(
  fs.readFileSync("/etc/nucleoid/configuration.json", "utf8")
);

const fork = require("child_process").fork;
var processes = [];

let pid = fork("./process.js", ["--id=main", "--path=/var/lib/nucleoid/"]);
let proc = { pid, id: "main", requests: [] };
pid.on("message", m => receive(proc, m));
processes["main"] = proc;

app.get("/", (req, res) => {
  res.sendFile(`${__dirname}/terminal.html`);
});

app.post("/", (req, res) => {
  let processId = req.get("Process");

  if (!processId) {
    processId = "main";
  }

  let path = `/var/lib/nucleoid/${processId}`;

  if (fs.existsSync(path) && fs.lstatSync(path).isDirectory()) {
    req.type = "ASYNC";

    fs.readdirSync(path).forEach(file => {
      let proc = processes[`${processId}/${file}`];
      proc.requests.push(req);

      if (proc.requests.length === 1) {
        proc.pid.send(req.body);
      }
    });

    fs.appendFileSync(
      `/var/lib/nucleoid/init/${processId}`,
      JSON.stringify({ s: req.body, d: Date.now() }) + "\n"
    );
    res.status(202).end();
  } else {
    req.type = "SYNC";

    let proc = processes[processId];

    if (proc === undefined) {
      let pid = fork("./process.js", [
        `--id=${processId}`,
        "--path=/var/lib/nucleoid/"
      ]);

      proc = { pid, id: processId, requests: [req] };
      processes[processId] = proc;

      proc.pid.on("message", m => receive(proc, m));

      proc.pid.send(req.body);
    } else {
      proc.requests.push(req);

      if (proc.requests.length === 1) {
        proc.pid.send(req.body);
      }
    }
  }
});

app.listen(config.port ? config.port : 80);

function receive(proc, message) {
  let request = proc.requests.shift();

  if (request.type === "SYNC") {
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
