const express = require("express");
const router = express.Router();
const data = require("../libs/data");

router.get("/logs", (req, res) => res.json(data.tail()));

module.exports = router;
