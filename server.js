const express = require("express");
var bodyParser = require("body-parser");
var jwt = require("jsonwebtoken");
var fs = require("fs");
var dir = "./data/";

if (!fs.existsSync(dir)) {
  fs.mkdirSync(dir);
}

const app = express();
app.use(bodyParser.text({ type: "*/*" }));

const fork = require("child_process").fork;
var processes = [];

app.post("/", req => {
  let header = req.get("Authorization");
  let parts = header.split(" ");
  var payload = jwt.decode(parts[1]);

  let key = payload.username;
  let proc = processes[key];

  if (proc === undefined) {
    let pid = fork("./process.js", [`--path=${dir}${key}`]);

    proc = { pid, requests: [req] };
    processes[key] = proc;

    proc.pid.on("message", function(message) {
      let request = proc.requests.shift();
      request.res.send(message);

      if (proc.requests.length > 0) {
        let r = proc.requests[0];
        proc.pid.send(r.body);
      }
    });

    proc.pid.send(req.body);
  } else {
    proc.requests.push(req);

    if (proc.requests.length === 1) {
      proc.pid.send(req.body);
    }
  }
});

app.listen(80);
