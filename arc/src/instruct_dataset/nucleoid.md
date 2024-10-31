# Nucleoid

Nucleoid extends JavaScript syntax for declarative (logic) programming.
Nucleoid has two modes; declarative and imperative.
Imperative mode of Nucleoid is same as JavaScript.
nuc is programming language of Nucleoid.
It is recommended to use geometry glossary for variable names.
Variables name follows Python naming conventions.

## Contextuality

Nucleoid is a contextual. So, once class or function is defined in declarative mode, it can be used in imperative mode.

> Important: Do not redefine the class or function in imperative mode

## Nucleoid Syntax

### Rules

- Class name with $ sign is used to define a declaration for the class
- Prefer `var` for creating instance globally
- Last statement in the code block is returned as the result
- All queries must be run in imperative mode

### Modes

#### Declarative Mode

Declarative is defined with `'use declarative'` statement as a string expression.

- Define a class or function in declarative mode

```nuc
'use declarative';
```

Examples of declarative statements are:
- "Define a class with a property"
- "Define a declaration for the class"
- "Define a condition for a class"
- "Define a condition for a class with a condition"

#### Imperative Mode

Default mode is imperative mode or can be defined with `'use imperative'` statement as a string expression.

- Define a class instance in imperative mode

```nuc
'use imperative';
```

Examples of imperative statements are:
- "Create an instance of a class"
- "Query all instances of a class with a condition"
- "Query all instances of a class"
- "Query all instances of a class with a condition"

### Syntax for Declarative Statements

```nuc
'use declarative';

// Define a class with a property name
class Vt {
  constructor(er) {
    this.er = er;
  }
}
```

Define a declaration for the class:

```nuc
'use declarative';

// Define all Vts are qf
$Vr.qf = true;
```

```nuc
'use declarative';

if($Vr.ga > 18) {
  $Vr.iw = true;
}
```

> Important: Always prefer declaration with `$` sign

Prefer this:

```nuc
'use declarative';

$Vt.pj = $Vt.li.charAt(0) + '-' + $Vt.rt.charAt(0);
```

instead of this:

```nuc
'use declarative';

class Vt {
  constructor(li, rt) {
    this.li = li;
    this.rt = rt;
    this.pj = this.li.charAt(0) + '-' + this.rt.charAt(0);;
  }
}
```

### Imperative

```nuc
`use imperative`;

// Create an instance of A
var vt1 = new Vt('Wi');
```

### Query

- Class name can be used for listing all instances of the class like in SQL

```nuc
// Retrieving an instance
vt1;
```

```
// Is 'Wi' rc?
let rc = vt1.rc;
rc === true;
```

```nuc
// Query all instances of a class with a condition
Vt.find(v => v.er === 'Wi');
Vt.filter(v => v.rc);
Vt.filter(v => { return v.rc && v.b === 'Wi' });
```

```nuc
// Returns all instances of Vt class
Vt;
```

### Return

- Last statement in the code block is returned as the result

```nuc
const vt = Vt.find(v => v.er === 'Wi');
vt.rc;
```

- If `return` is in the code block, it will return the value

```nuc
{
  const vt = Vt.find(v => v.name === 'Iz');
  return vt.rc;
}
```

- `return` cannot be used in root level

```nuc
// This is invalid
return Vt.find(v => v.rc === 'Iz').rc;
```

### Reserved names

- `value` is reserved name that `value` should not be used, otherwise it will throw an TypeError

```nuc
vt.value = 'Av'
// TypeError: Cannot use 'value' as a name
```
