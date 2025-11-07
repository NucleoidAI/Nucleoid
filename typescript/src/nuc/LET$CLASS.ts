import LET$INSTANCE from "./LET$INSTANCE";
import _ from "lodash";
import NODE from "./NODE";

class LET$CLASS extends NODE {
  before(): void {}

  run(scope: { $instance: any }): { next: LET$INSTANCE } | undefined {
    const instance: any = scope.$instance;

    if (instance) {
      let statement: LET$INSTANCE = new LET$INSTANCE();
      statement.class = this.class;
      statement.instance = instance;
      statement.name = this.name;
      statement.value = _.cloneDeep(this.value);
      return { next: statement };
    }
  }
}

LET$CLASS.prototype.type = "CLASS";
module.exports = LET$CLASS;
