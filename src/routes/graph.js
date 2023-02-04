const express = require("express");
const router = express.Router();
const graph = require("../graph");

router.get("/graph", (req, res) => {
  const result = {};

  for (const key in graph) {
    const node = graph[key];

    if (node.id && node.key) {
      result[node.key] = node;

      const tmp = {
        id: node.id,
        type: node.constructor.name,
      };

      for (const prop in node) {
        const object = node[prop];

        if (typeof object === "object" && object?.id) {
          tmp[prop] = object.id;
        } else if (typeof object === "object" && !object?.id) {
          const list = [];

          for (const innerProp in object) {
            const innerObject = object[innerProp];

            if (typeof innerObject === "object" && innerObject?.id) {
              list.push(innerObject.id);
            }
          }

          if (list.length) {
            tmp[prop] = list;
          }
        }
      }

      result[key] = tmp;
    }
  }

  res.json(result);
});

module.exports = router;
