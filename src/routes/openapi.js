const express = require("express");
const router = express.Router();
const openapi = require("../lib/openapi");
const context = require("../context");
const config = require("../config");
const Joi = require("joi");

router.get("/openapi", (req, res) => res.json(openapi.status()));
router.post("/openapi", (req, res) => {
  const {
    "x-nuc-action": action,
    "x-nuc-functions": functions = [],
    "x-nuc-port": port,
  } = Joi.attempt(
    req.body,
    Joi.object({
      "x-nuc-action": Joi.string().required(),
      "x-nuc-functions": Joi.array().optional(),
      "x-nuc-port": Joi.number().optional(),
    })
      .required()
      .options({ stripUnknown: true })
  );

  if (action === "start") {
    context.run(functions);

    openapi.init();
    openapi.load(req.body);

    openapi.start(port || config().port.openapi);
  }

  if (action === "stop") {
    openapi.stop();
  }

  res.end();
});

module.exports = router;
