var nucleoid = require("./nucleoid.js");

process.on("message", message => {
  try {
    var result = nucleoid.run(message);
  } catch (error) {
    if (error instanceof Error) {
      process.send(error.message);
    } else {
      process.send(error);
    }

    return;
  }

  if (result === undefined) {
    process.send("");
  } else {
    process.send(JSON.stringify(result));
  }
});
