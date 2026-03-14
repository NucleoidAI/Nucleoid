const datastore = require("../datastore");
const express = require("express");
const router = express.Router();

router.get("/logs", (req, res) => res.json(datastore.tail()));

module.exports = router;
