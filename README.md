## Install

```
sudo apt-add-repository ppa:nucleoid/nucleoid
sudo apt install nucleoid
```

## Docker

```
docker run -d -p 80:80 nucleoid/nucleoid
```

## Hello World

Open the terminal on your browser

```
> a = 1
> b = a * 2
> b
2
> a = 2
> b
4
```

## What is Nucleoid?

Nucleoid is an open source (Apache 2.0), a runtime environment for declarative programming in ES6/JavaScript syntax and runs as a datastore. Since statements are declarative, the runtime automatically provides logical integrity, multiprocessing, plasticity, persistency etc.

The runtime cumulatively stores each statement and builds in the state so that doesn't require external data storage like RDBMS. This design eliminates complexity of the architecture and gains high performance since no network communication required, especially, in locking situations.

### Declarative Runtime Environment

The declarative runtime environment isolates a behavior definition of a program from its technical instructions and executes declarative statements, which represent logical intention without carrying any technical details. In this paradigm, there is no segregation regarding what data is or not, instead approaches how data is related with others so that any type of data including business rules can be added without requiring any additional actions such as compiling, configuring, restarting as a result of plasticity.

### Syntax and Semantics

In declarative programming, as its name suggests, it runs based on definition of syntax, where a syntax is often followed by developers in order to achieve behaviors in specifications. For example:

```
> a = 1
> b = a * 2
```

`=` represents assignment of expression to a variable. In Nucleoid runtime, it follows formal logic as semantics in ES6 syntax. So, if the expression contains another variable, it is considered as dependency, and by the design, whenever the variable is changed, the dependent variable also has to change along with, otherwise it breaks its logical integrity.

However, in imperative programming, for the same example, when `a` is changed, it changes memory location where `a` refers, but the change doesn't alter variable `b` because in the context of IP, it is just a representation of memory location.

### Control Flow

An important difference in declarative programming oppose to imperative is who manages the control flow. In IP, a programmer has full control of instructions that runs on CPU through programming language, but in declarative programming, it is based on semantics, and in Nucleoid, it follows formal logic as semantics.

Due to its plasticity, the runtime is able to adjust control flow as receives more statements, so that externalized configuration files are optional.

### Persistency

Nucleoid runtime cumulatively stores statements in order as received so that the runtime doesn't require external database. This feature is enabled by declarative programming as a result of plasticity and lowers complexity of the system along with gaining better performance since there is no network communication required.

Learn more at https://nucleoid.org/tutorial/
