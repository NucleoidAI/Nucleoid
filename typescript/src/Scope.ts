import Identifier from "./lang/ast/Identifier";
import { append } from "./lang/estree/estree";

class Scope {
  prior: Scope | null;
  block: object;
  root: Scope;
  local: Record<string, unknown>;
  instance: object | null;
  graph: Record<string, unknown>;
  callback: Array<any>;
  instances: Record<string, unknown>;
  object?: { name: string };

  constructor(prior: Scope | null, block: object) {
    this.prior = prior;
    this.block = block;

    if (prior) {
      this.root = prior.root;
    } else {
      this.root = this;
    }

    this.local = {};
    this.instance = null;
    this.graph = {};
    this.callback = [];
    this.instances = {};
  }

  get $instance(): object | null {
    // eslint-disable-next-line @typescript-eslint/no-this-alias
    let index: Scope | null = this;

    while (index) {
      const instance = index.instance;

      if (instance) {
        return instance;
      }

      index = index.prior;
    }

    return null;
  }

  set $instance(instance: object | null) {
    this.instance = instance;
  }

  assign(
    variable: Identifier,
    evaluation: unknown,
    reassign: boolean = false
  ): unknown {
    let prefix: any = undefined;

    if (reassign) {
      const retrieved = this.retrieve(variable);
      if (retrieved && retrieved.object) {
        prefix = retrieved.object.node;
      }
    }

    if (!prefix) {
      prefix = {
        type: "MemberExpression",
        computed: false,
        object: {
          type: "Identifier",
          name: "scope",
        },
        property: {
          type: "Identifier",
          name: "local",
        },
      };
    }

    const local = new Identifier(append(prefix, variable.node));
    // eslint-disable-next-line no-eval
    return eval(`${local}=${evaluation}`);
  }

  retrieve(variable: Identifier, exact: boolean = false): Identifier | null {
    // eslint-disable-next-line @typescript-eslint/no-this-alias
    let index: Scope | null = this;

    let estree: any = {
      type: "Identifier",
      name: "scope",
    };

    const first = variable.first;
    if (!first) return null;

    const firstStr = first.toString();

    while (index) {
      if (
        index.graph[firstStr] !== undefined &&
        // eslint-disable-next-line no-eval
        (!exact || eval(`index.local.${variable}`) !== undefined)
      ) {
        const local = {
          type: "Identifier",
          name: "local",
        };
        estree = append(estree, local);
        return new Identifier(append(estree, variable.node));
      }

      const prior = {
        type: "Identifier",
        name: "prior",
      };

      estree = append(estree, prior);

      index = index.prior;
    }

    return null;
  }

  retrieveInstance(instance: string): unknown {
    // eslint-disable-next-line @typescript-eslint/no-this-alias
    let index: Scope | null = this;

    while (index) {
      const value = index.instances[instance];

      if (value) {
        return value;
      }

      index = index.prior;
    }
    return undefined;
  }

  retrieveObject(): string | null {
    // eslint-disable-next-line @typescript-eslint/no-this-alias
    let index: Scope | null = this;

    while (index) {
      if (index.object !== undefined) {
        return index.object.name;
      }

      index = index.prior;
    }

    return null;
  }

  retrieveGraph(instance: string): unknown {
    // eslint-disable-next-line @typescript-eslint/no-this-alias
    let index: Scope | null = this;

    while (index) {
      const value = index.graph[instance];

      if (value) {
        return value;
      }

      index = index.prior;
    }
    return undefined;
  }
}

export default Scope;
