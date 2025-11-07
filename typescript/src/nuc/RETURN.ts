import NODE from "./NODE.js";

class RETURN extends NODE {
  constructor(statement: string) {
    super();
    this.statement = statement;
  }

  statement: string;
}

export default RETURN;
