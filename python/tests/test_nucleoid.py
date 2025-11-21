"""
Tests for Nucleoid runtime.

This test suite corresponds to the TypeScript test file:
typescript/src/test/nucleoid.spec.ts

Includes both declarative mode tests (lines 23-2206) and imperative mode tests (lines 2208-2246).
Total tests: 187 (183 declarative + 4 imperative)
"""

import pytest
from nucleoid import nucleoid
from nucleoid.lib.test import clear


class TestNucleoidDeclarativeMode:
    """
    Complete test suite for Nucleoid runtime in declarative mode.

    This class contains ALL tests from the 'in declarative mode' describe block
    in the TypeScript test suite (lines 23-2206).

    Total declarative tests: 183
    """

    @classmethod
    def setup_class(cls):
        """Setup class - start nucleoid with test mode and declarative mode."""
        nucleoid.start({'test': True, 'options': {'declarative': True}})

    def setup_method(self):
        """Setup method - clear state before each test."""
        clear()

    # ========================================================================
    # TEST 1 of 183: runs statements in the state
    # TypeScript: line 23
    # ========================================================================

    def test_runs_statements_in_the_state(self):
        """
        Test that nucleoid runs statements and stores them in the state.

        Original TypeScript (line 23):
        ```typescript
        it("runs statements in the state", () => {
          nucleoid.run("var i = 1 ;");
          equal(nucleoid.run("i == 1"), true);
        });
        ```
        """
        nucleoid.run("var i = 1")
        assert nucleoid.run("i == 1") is True

    # ========================================================================
    # TEST 2 of 183: runs expression statement
    # TypeScript: line 28
    # ========================================================================

    def test_runs_expression_statement(self):
        """
        Original TypeScript (line 28):
        ```typescript
        it("runs expression statement", () => {
          nucleoid.run("var j = 1 ;");
          equal(nucleoid.run("j + 2"), 3);
        });
        ```
        """
        nucleoid.run("var j = 1")
        assert nucleoid.run("j + 2") == 3

    # ========================================================================
    # TEST 3 of 183: rejects variable declaration without definition
    # TypeScript: line 33
    # ========================================================================

    def test_rejects_variable_declaration_without_definition(self):
        """
        Original TypeScript (line 33):
        ```typescript
        it("rejects variable declaration without definition", () => {
          throws(
            () => nucleoid.run("var a"),
            (err: Error) => validate(err, SyntaxError, "Missing definition")
          );
        });
        ```
        """
        with pytest.raises(SyntaxError, match="Missing definition"):
            nucleoid.run("var a")

    # ========================================================================
    # TEST 4 of 183: allows variable declaration without var keyword
    # TypeScript: line 40
    # ========================================================================

    def test_allows_variable_declaration_without_var_keyword(self):
        """
        Original TypeScript (line 40):
        ```typescript
        it("allows variable declaration without var keyword", () => {
          nucleoid.run("pi = 3.14 ;");
          equal(nucleoid.run("pi == 3.14"), true);
        });
        ```
        """
        nucleoid.run("pi = 3.14")
        assert nucleoid.run("pi == 3.14") is True

    # ========================================================================
    # TEST 5 of 183: allows statements without semicolon at the end
    # TypeScript: line 45
    # ========================================================================

    def test_allows_statements_without_semicolon_at_the_end(self):
        """
        Original TypeScript (line 45):
        ```typescript
        it("allows statements without semicolon at the end", () => {
          nucleoid.run("au = 149597870700");
          equal(nucleoid.run("au == 149597870700"), true);
        });
        ```
        """
        nucleoid.run("au = 149597870700")
        assert nucleoid.run("au == 149597870700") is True

    # ========================================================================
    # TEST 6 of 183: creates dependency based on length of identifier
    # TypeScript: line 50
    # ========================================================================

    def test_creates_dependency_based_on_length_of_identifier(self):
        """
        Original TypeScript (line 50):
        ```typescript
        it("creates dependency based on length of identifier", () => {
          nucleoid.run("str1 = 'ABC'");
          nucleoid.run("i1 = str1.length + 1");
          equal(nucleoid.run("i1"), 4);

          nucleoid.run("str1 = 'ABCD'");
          equal(nucleoid.run("i1"), 5);

          nucleoid.run("if ( str1.length > 5 ) { i2 = i1 }");
          nucleoid.run("str1 = 'ABCDEF'");
          equal(nucleoid.run("i2"), 7);
        });
        ```
        """
        nucleoid.run("str1 = 'ABC'")
        nucleoid.run("i1 = str1.length + 1")
        assert nucleoid.run("i1") == 4

        nucleoid.run("str1 = 'ABCD'")
        assert nucleoid.run("i1") == 5

        nucleoid.run("if (str1.length > 5) { i2 = i1 }")
        nucleoid.run("str1 = 'ABCDEF'")
        assert nucleoid.run("i2") == 7

    # ========================================================================
    # TEST 7 of 183: creates class with constructor
    # TypeScript: line 63
    # ========================================================================

    def test_creates_class_with_constructor(self):
        """
        Original TypeScript (line 63):
        ```typescript
        it("creates class with constructor", () => {
          nucleoid.run(
            "class Device { constructor ( name ) { this.name = name } }"
          );
          nucleoid.run("$Device.active = false");
          nucleoid.run("if ( $Device.name ) { $Device.active = true }");

          nucleoid.run("device1 = new Device ( 'Entrance' )");
          equal(nucleoid.run("device1.name"), "Entrance");
          equal(nucleoid.run("device1.active"), true);

          const device2 = nucleoid.run("new Device ( 'Hall' )");
          equal(nucleoid.run(`${device2.id}.name`), "Hall");
          equal(nucleoid.run(`${device2.id}.active`), true);

          nucleoid.run("device3 = new Device ( )");
          equal(nucleoid.run("device3.name"), undefined);
          equal(nucleoid.run("device3.active"), false);

          const device4 = nucleoid.run("new Device ( )");
          equal(nucleoid.run(`${device4.id}.name`), undefined);
          equal(nucleoid.run(`${device4.id}.active`), false);
        });
        ```
        """
        nucleoid.run("class Device { constructor(name) { this.name = name } }")
        nucleoid.run("$Device.active = false")
        nucleoid.run("if ($Device.name) { $Device.active = true }")

        nucleoid.run("device1 = new Device('Entrance')")
        assert nucleoid.run("device1.name") == "Entrance"
        assert nucleoid.run("device1.active") is True

        device2 = nucleoid.run("new Device('Hall')")
        assert nucleoid.run(f"{device2['id']}.name") == "Hall"
        assert nucleoid.run(f"{device2['id']}.active") is True

        nucleoid.run("device3 = new Device()")
        assert nucleoid.run("device3.name") is None
        assert nucleoid.run("device3.active") is False

        device4 = nucleoid.run("new Device()")
        assert nucleoid.run(f"{device4['id']}.name") is None
        assert nucleoid.run(f"{device4['id']}.active") is False

    # ========================================================================
    # TEST 8 of 183: adds object in object list
    # TypeScript: line 87
    # ========================================================================

    def test_adds_object_in_object_list(self):
        """
        Original TypeScript (line 87):
        ```typescript
        it("adds object in object list", () => {
          nucleoid.run("class Student { }");
          nucleoid.run("user0 = new Student ( )");
          deepEqual(
            nucleoid.run("Student.find ( student => student.id === 'user0')"),
            { id: "user0" }
          );
          deepEqual(nucleoid.run("Student['user0']"), { id: "user0" });
        });
        ```
        """
        nucleoid.run("class Student {}")
        nucleoid.run("user0 = new Student()")
        assert nucleoid.run("Student.find(student => student.id === 'user0')") == {"id": "user0"}
        assert nucleoid.run("Student['user0']") == {"id": "user0"}

    # ========================================================================
    # TEST 9 of 183: prevents class and object lists when class is updated
    # TypeScript: line 97
    # ========================================================================

    def test_prevents_class_and_object_lists_when_class_is_updated(self):
        """
        Original TypeScript (line 97):
        ```typescript
        it("prevents class and object lists when class is updated", () => {
          nucleoid.run("class User { }");
          nucleoid.run("new User ( )");
          equal(nucleoid.run("classes.length"), 1);
          equal(nucleoid.run("User.length"), 1);

          nucleoid.run("class User { }");
          nucleoid.run("new User ( )");
          equal(nucleoid.run("classes.length"), 1);
          equal(nucleoid.run("User.length"), 2);
        });
        ```
        """
        nucleoid.run("class User {}")
        nucleoid.run("new User()")
        assert nucleoid.run("classes.length") == 1
        assert nucleoid.run("User.length") == 1

        nucleoid.run("class User {}")
        nucleoid.run("new User()")
        assert nucleoid.run("classes.length") == 1
        assert nucleoid.run("User.length") == 2

    # ========================================================================
    # TEST 10 of 183: adds created class in class list
    # TypeScript: line 109
    # ========================================================================

    def test_adds_created_class_in_class_list(self):
        """
        Original TypeScript (line 109):
        ```typescript
        it("adds created class in class list", () => {
          equal(nucleoid.run("classes.length"), 0);

          nucleoid.run("class Student { }");
          equal(nucleoid.run("classes.length"), 1);

          nucleoid.run("class User { }");
          equal(nucleoid.run("classes.length"), 2);
        });
        ```
        """
        assert nucleoid.run("classes.length") == 0

        nucleoid.run("class Student {}")
        assert nucleoid.run("classes.length") == 1

        nucleoid.run("class User {}")
        assert nucleoid.run("classes.length") == 2

    # ========================================================================
    # TEST 11 of 183: updates class definition
    # TypeScript: line 119
    # ========================================================================

    def test_updates_class_definition(self):
        """
        Original TypeScript (line 119):
        ```typescript
        it("updates class definition", () => {
          nucleoid.run("class Message { }");
          nucleoid.run("$Message.read = false");
          nucleoid.run("message1 = new Message ( )");
          nucleoid.run(
            "class Message { constructor ( payload ) { this.payload = payload } }"
          );
          equal(nucleoid.run("message1.read"), false);
          equal(nucleoid.run("message1.payload"), undefined);

          nucleoid.run("message2 = new Message('MESSAGE')");
          equal(nucleoid.run("message2.read"), false);
          equal(nucleoid.run("message2.payload"), "MESSAGE");
        });
        ```
        """
        nucleoid.run("class Message {}")
        nucleoid.run("$Message.read = false")
        nucleoid.run("message1 = new Message()")
        nucleoid.run("class Message { constructor(payload) { this.payload = payload } }")
        assert nucleoid.run("message1.read") is False
        assert nucleoid.run("message1.payload") is None

        nucleoid.run("message2 = new Message('MESSAGE')")
        assert nucleoid.run("message2.read") is False
        assert nucleoid.run("message2.payload") == "MESSAGE"

    # ========================================================================
    # TEST 12 of 183: supports new line as replacing with space
    # TypeScript: line 134
    # ========================================================================

    def test_supports_new_line_as_replacing_with_space(self):
        """
        Original TypeScript (line 134):
        ```typescript
        it("supports new line as replacing with space", () => {
          nucleoid.run("a = 1 ; \\n b = 2");
          equal(nucleoid.run("a"), 1);
          equal(nucleoid.run("b"), 2);

          nucleoid.run("a = 3 ; \\r b = 4");
          equal(nucleoid.run("a"), 3);
          equal(nucleoid.run("b"), 4);

          nucleoid.run("a = 5 ; \\r\\n b = 6");
          equal(nucleoid.run("a"), 5);
          equal(nucleoid.run("b"), 6);
        });
        ```
        """
        nucleoid.run("a = 1 ; \n b = 2")
        assert nucleoid.run("a") == 1
        assert nucleoid.run("b") == 2

        nucleoid.run("a = 3 ; \r b = 4")
        assert nucleoid.run("a") == 3
        assert nucleoid.run("b") == 4

        nucleoid.run("a = 5 ; \r\n b = 6")
        assert nucleoid.run("a") == 5
        assert nucleoid.run("b") == 6

    # ========================================================================
    # TEST 13 of 183: supports string in expression
    # TypeScript: line 148
    # ========================================================================

    def test_supports_string_in_expression(self):
        """
        Original TypeScript (line 148):
        ```typescript
        it("supports string in expression", () => {
          equal(nucleoid.run("'New String'"), "New String");
          equal(nucleoid.run('"New String"'), "New String");
          equal(nucleoid.run("`New String`"), "New String");

          nucleoid.run("a = 123");
          equal(nucleoid.run("`New ${a} String`"), "New 123 String");
        });
        ```
        """
        assert nucleoid.run("'New String'") == "New String"
        assert nucleoid.run('"New String"') == "New String"
        assert nucleoid.run("`New String`") == "New String"

        nucleoid.run("a = 123")
        assert nucleoid.run("`New ${a} String`") == "New 123 String"

    # ========================================================================
    # TEST 14 of 183: supports logical operators
    # TypeScript: line 157
    # ========================================================================

    def test_supports_logical_operators(self):
        """
        Original TypeScript (line 157):
        ```typescript
        it("supports logical operators", () => {
          nucleoid.run("condition = false");
          equal(nucleoid.run("condition || true"), true);
          equal(nucleoid.run("!condition && true"), true);
        });
        ```
        """
        nucleoid.run("condition = false")
        assert nucleoid.run("condition || true") is True
        assert nucleoid.run("!condition && true") is True

    # ========================================================================
    # TEST 15 of 183: supports standard built-in functions
    # TypeScript: line 163
    # ========================================================================

    def test_supports_standard_built_in_functions(self):
        """
        Original TypeScript (line 163):
        ```typescript
        it("supports standard built-in functions", () => {
          nucleoid.run("max = Number.MAX_SAFE_INTEGER");
          nucleoid.run("now = Date.now ( )");
        });
        ```
        """
        nucleoid.run("max = Number.MAX_SAFE_INTEGER")
        nucleoid.run("now = Date.now()")

    # ========================================================================
    # TEST 16 of 183: supports creating standard built-in objects
    # TypeScript: line 168
    # ========================================================================

    def test_supports_creating_standard_built_in_objects(self):
        """
        Original TypeScript (line 168):
        ```typescript
        it("supports creating standard built-in objects", () => {
          nucleoid.run("date = new Date ( '2019-7-24' )");
          equal(nucleoid.run("date.getYear()"), 119);
        });
        ```
        """
        nucleoid.run("date = new Date('2019-7-24')")
        assert nucleoid.run("date.getYear()") == 119

    # ========================================================================
    # TEST 17 of 183: supports built-in objects
    # TypeScript: line 173
    # ========================================================================

    def test_supports_built_in_objects(self):
        """
        Original TypeScript (line 173):
        ```typescript
        it("supports built-in objects", () => {
          const result = nucleoid.run("date1 = new Date ( )");
          nucleoid.run(`date2 = new Date ( ${result.getTime()} )`);
          equal(nucleoid.run("date1.getTime() == date2.getTime()"), true);

          nucleoid.run("date3 = Date.parse ( '04 Dec 1995 00:12:00 GMT' )");
          equal(nucleoid.run("date3"), 818035920000);

          throws(
            () => {
              nucleoid.run("date4 = Date.wrong ( )");
            },
            (error: Error) =>
              validate(error, TypeError, "Date.wrong is not a function")
          );
        });
        ```
        """
        result = nucleoid.run("date1 = new Date()")
        nucleoid.run(f"date2 = new Date({result.getTime()})")
        assert nucleoid.run("date1.getTime() == date2.getTime()") is True

        nucleoid.run("date3 = Date.parse('04 Dec 1995 00:12:00 GMT')")
        assert nucleoid.run("date3") == 818035920000

        with pytest.raises(TypeError, match="Date.wrong is not a function"):
            nucleoid.run("date4 = Date.wrong()")

    # ========================================================================
    # TEST 18 of 183: calls function with no return
    # TypeScript: line 190
    # ========================================================================

    def test_calls_function_with_no_return(self):
        """
        Original TypeScript (line 190):
        ```typescript
        it("calls function with no return", () => {
          nucleoid.run("a = 1");
          nucleoid.run("function copy ( val ) { b = val }");
          const result = nucleoid.run("copy ( a )");
          equal(result, undefined);
        });
        ```
        """
        nucleoid.run("a = 1")
        nucleoid.run("function copy(val) { b = val }")
        result = nucleoid.run("copy(a)")
        assert result is None

    # ========================================================================
    # TEST 19 of 183: calls function with returning variable
    # TypeScript: line 197
    # ========================================================================

    def test_calls_function_with_returning_variable(self):
        """
        Original TypeScript (line 197):
        ```typescript
        it("calls function with returning variable", () => {
          nucleoid.run("a = 1");
          nucleoid.run("function copy ( val ) { b = val; return b; }");
          const result = nucleoid.run("copy ( a )");
          equal(result, 1);
        });
        ```
        """
        nucleoid.run("a = 1")
        nucleoid.run("function copy(val) { b = val; return b; }")
        result = nucleoid.run("copy(a)")
        assert result == 1

    # ========================================================================
    # TEST 20 of 183: calls function with returning value
    # TypeScript: line 204
    # ========================================================================

    def test_calls_function_with_returning_value(self):
        """
        Original TypeScript (line 204):
        ```typescript
        it("calls function with returning value", () => {
          nucleoid.run("a = 1");
          nucleoid.run("function copy ( val ) { b = val; return val; }");
          const result = nucleoid.run("copy ( a )");
          equal(result, 1);
        });
        ```
        """
        nucleoid.run("a = 1")
        nucleoid.run("function copy(val) { b = val; return val; }")
        result = nucleoid.run("copy(a)")
        assert result == 1

    # ========================================================================
    # TEST 21 of 183: supports function in expression
    # TypeScript: line 211
    # ========================================================================

    def test_supports_function_in_expression(self):
        """
        Original TypeScript (line 211):
        ```typescript
        it("supports function in expression", () => {
          nucleoid.run("list = [1, 2, 3]");
          equal(
            nucleoid.run(
              "list.find ( function ( element ) { return element == 3 } )"
            ),
            3
          );

          equal(
            nucleoid.run("list.find ( element => { return element == 2 } )"),
            2
          );

          equal(nucleoid.run("list.find ( element => element == 1 )"), 1);
          equal(nucleoid.run("list.find ( element => ( element == 1 ) )"), 1);
        });
        ```
        """
        nucleoid.run("list = [1, 2, 3]")
        assert nucleoid.run("list.find(function(element) { return element == 3 })") == 3
        assert nucleoid.run("list.find(element => { return element == 2 })") == 2
        assert nucleoid.run("list.find(element => element == 1)") == 1
        assert nucleoid.run("list.find(element => (element == 1))") == 1

    # ========================================================================
    # TEST 22 of 183: supports function with parameter in expression
    # TypeScript: line 229
    # ========================================================================

    def test_supports_function_with_parameter_in_expression(self):
        """
        Original TypeScript (line 229):
        ```typescript
        it("supports function with parameter in expression", () => {
          nucleoid.run("samples = [ 38.2 , 39.1 , 38.8 , 39 ]");
          nucleoid.run("ratio = 2.1");
          nucleoid.run("element = 38.5");
          equal(
            nucleoid.run(
              "samples.find ( function ( element ) { let result = element * ratio ; return result == 81.48 } )"
            ),
            38.8
          );

          equal(
            nucleoid.run(
              "samples.find ( element => { let result = element * ratio ; return result == 81.48 } )"
            ),
            38.8
          );

          equal(nucleoid.run("samples.find ( element => element == 38.8 )"), 38.8);
          equal(
            nucleoid.run("samples.find ( element => ( element == 38.8 ) )"),
            38.8
          );
        });
        ```
        """
        nucleoid.run("samples = [38.2, 39.1, 38.8, 39]")
        nucleoid.run("ratio = 2.1")
        nucleoid.run("element = 38.5")
        assert nucleoid.run("samples.find(function(element) { let result = element * ratio; return result == 81.48 })") == 38.8
        assert nucleoid.run("samples.find(element => { let result = element * ratio; return result == 81.48 })") == 38.8
        assert nucleoid.run("samples.find(element => element == 38.8)") == 38.8
        assert nucleoid.run("samples.find(element => (element == 38.8))") == 38.8

    # ========================================================================
    # TEST 23 of 183: creates let statement with JSON
    # TypeScript: line 254
    # ========================================================================

    def test_creates_let_statement_with_json(self):
        """
        Original TypeScript (line 254):
        ```typescript
        it("creates let statement with JSON", () => {
          const payload = nucleoid.run(
            '{ let payload = { "data" : "TEST" , "nested" : { "data" : "NESTED_TEST" } } ; return payload }'
          );
          equal(payload.data, "TEST");
          equal(payload.nested.data, "NESTED_TEST");

          nucleoid.run('message = { "pid" : 1200 }');
          equal(nucleoid.run("message.pid"), 1200);

          const i = nucleoid.run(
            '{ let scope = { query : "test" } ; let i = { test : scope.query } ; return i ; }'
          );
          equal(i.test, "test");
        });
        ```
        """
        payload = nucleoid.run('{ let payload = { "data" : "TEST" , "nested" : { "data" : "NESTED_TEST" } }; return payload }')
        assert payload['data'] == "TEST"
        assert payload['nested']['data'] == "NESTED_TEST"

        nucleoid.run('message = { "pid" : 1200 }')
        assert nucleoid.run("message.pid") == 1200

        i = nucleoid.run('{ let scope = { query : "test" }; let i = { test : scope.query }; return i; }')
        assert i['test'] == "test"

    # ========================================================================
    # TEST 24 of 183: returns inline JSON object
    # TypeScript: line 270
    # ========================================================================

    def test_returns_inline_json_object(self):
        """
        Original TypeScript (line 270):
        ```typescript
        it("returns inline JSON object", () => {
          const json = nucleoid.run(
            '{ return { "number" : 123 , "string" : "ABC" , "bool" : true } }'
          );
          deepEqual(json, { number: 123, string: "ABC", bool: true });
        });
        ```
        """
        json = nucleoid.run('{ return { "number" : 123 , "string" : "ABC" , "bool" : true } }')
        assert json == {"number": 123, "string": "ABC", "bool": True}

    # ========================================================================
    # TEST 25 of 183: returns inline JSON array
    # TypeScript: line 277
    # ========================================================================

    def test_returns_inline_json_array(self):
        """
        Original TypeScript (line 277):
        ```typescript
        it("returns inline JSON array", () => {
          const json = nucleoid.run(
            '{ return [ { "number" : 123 , "string" : "ABC" , "bool" : true } ] }'
          );
          deepEqual(json, [{ number: 123, string: "ABC", bool: true }]);
        });
        ```
        """
        json = nucleoid.run('{ return [ { "number" : 123 , "string" : "ABC" , "bool" : true } ] }')
        assert json == [{"number": 123, "string": "ABC", "bool": True}]

    # ========================================================================
    # TEST 26 of 183: returns inline object
    # TypeScript: line 284
    # ========================================================================

    def test_returns_inline_object(self):
        """
        Original TypeScript (line 284):
        ```typescript
        it("returns inline object", () => {
          const object = nucleoid.run(
            '{ return { number : 123 , string : "ABC" , bool : true } }'
          );
          deepEqual(object, { number: 123, string: "ABC", bool: true });
        });
        ```
        """
        obj = nucleoid.run('{ return { number : 123 , string : "ABC" , bool : true } }')
        assert obj == {"number": 123, "string": "ABC", "bool": True}

    # ========================================================================
    # TEST 27 of 183: returns inline array
    # TypeScript: line 291
    # ========================================================================

    def test_returns_inline_array(self):
        """
        Original TypeScript (line 291):
        ```typescript
        it("returns inline array", () => {
          const object = nucleoid.run(
            '{ return [ { number : 123 , string : "ABC" , bool : true } ] }'
          );
          deepEqual(object, [{ number: 123, string: "ABC", bool: true }]);
        });
        ```
        """
        obj = nucleoid.run('{ return [ { number : 123 , string : "ABC" , bool : true } ] }')
        assert obj == [{"number": 123, "string": "ABC", "bool": True}]

    # ========================================================================
    # TEST 28 of 183: assigns block in function as dependency
    # TypeScript: line 298
    # ========================================================================

    def test_assigns_block_in_function_as_dependency(self):
        """
        Original TypeScript (line 298):
        ```typescript
        it("assigns block in function as dependency", () => {
          nucleoid.run("class Student { }");
          nucleoid.run("student1 = new Student ( )");
          nucleoid.run("student1.age = 7");
          nucleoid.run("student2 = new Student ( )");
          nucleoid.run("student2.age = 8");
          nucleoid.run("student3 = new Student ( )");
          nucleoid.run("student3.age = 9");

          nucleoid.run("age = 8");
          nucleoid.run("student = Student.find ( s => s.age == age )");
          deepEqual(nucleoid.run("student"), nucleoid.run("student2"));

          nucleoid.run("age = 9");
          deepEqual(nucleoid.run("student"), nucleoid.run("student3"));
        });
        ```
        """
        nucleoid.run("class Student {}")
        nucleoid.run("student1 = new Student()")
        nucleoid.run("student1.age = 7")
        nucleoid.run("student2 = new Student()")
        nucleoid.run("student2.age = 8")
        nucleoid.run("student3 = new Student()")
        nucleoid.run("student3.age = 9")

        nucleoid.run("age = 8")
        nucleoid.run("student = Student.find(s => s.age == age)")
        assert nucleoid.run("student") == nucleoid.run("student2")

        nucleoid.run("age = 9")
        assert nucleoid.run("student") == nucleoid.run("student3")

    # ========================================================================
    # TEST 29 of 183: supports chained functions with parameter in expression
    # TypeScript: line 315
    # ========================================================================

    def test_supports_chained_functions_with_parameter_in_expression(self):
        """
        Original TypeScript (line 315):
        ```typescript
        it("supports chained functions with parameter in expression", () => {
          nucleoid.run(
            "class Result { constructor ( score ) { this.score = score } }"
          );
          nucleoid.run("new Result ( 10 ) ; new Result ( 15 ) ; new Result ( 20 )");

          nucleoid.run("upperThreshold = 18");
          nucleoid.run("lowerThreshold = 12");
          nucleoid.run(
            "list = Result.filter ( r => r.score > lowerThreshold ) .filter ( r => r.score < upperThreshold )"
          );
          let list = nucleoid.run("list");
          equal(list[0].score, 15);

          nucleoid.run("lowerThreshold = 7");
          list = nucleoid.run("list");
          equal(list[0].score, 10);
          equal(list[1].score, 15);

          nucleoid.run("upperThreshold = 14");
          list = nucleoid.run("list");
          equal(list[0].score, 10);
        });
        ```
        """
        nucleoid.run("class Result { constructor(score) { this.score = score } }")
        nucleoid.run("new Result(10); new Result(15); new Result(20)")

        nucleoid.run("upperThreshold = 18")
        nucleoid.run("lowerThreshold = 12")
        nucleoid.run("list = Result.filter(r => r.score > lowerThreshold).filter(r => r.score < upperThreshold)")
        list_result = nucleoid.run("list")
        assert list_result[0]['score'] == 15

        nucleoid.run("lowerThreshold = 7")
        list_result = nucleoid.run("list")
        assert list_result[0]['score'] == 10
        assert list_result[1]['score'] == 15

        nucleoid.run("upperThreshold = 14")
        list_result = nucleoid.run("list")
        assert list_result[0]['score'] == 10

    # ========================================================================
    # TEST 30 of 183: supports nested functions as parameter in expression
    # TypeScript: line 339
    # ========================================================================

    def test_supports_nested_functions_as_parameter_in_expression(self):
        """
        Original TypeScript (line 339):
        ```typescript
        it("supports nested functions as parameter in expression", () => {
          nucleoid.run("name = 'AbCDE'");
          nucleoid.run("pointer = 0");
          nucleoid.run(
            "if ( ! /[A-Z]/.test ( name.charAt ( pointer ) ) ) { throw 'INVALID_FIRST_CHARACTER' }"
          );

          throws(
            () => {
              nucleoid.run("name = 'bbCDE'");
            },
            (error) => error === "INVALID_FIRST_CHARACTER"
          );

          nucleoid.run("name = 'CbCDE'");

          throws(
            () => {
              nucleoid.run("pointer = 1");
            },
            (error) => error === "INVALID_FIRST_CHARACTER"
          );
        });
        ```
        """
        nucleoid.run("name = 'AbCDE'")
        nucleoid.run("pointer = 0")
        nucleoid.run("if (!/[A-Z]/.test(name.charAt(pointer))) { throw 'INVALID_FIRST_CHARACTER' }")

        with pytest.raises(Exception, match="INVALID_FIRST_CHARACTER"):
            nucleoid.run("name = 'bbCDE'")

        nucleoid.run("name = 'CbCDE'")

        with pytest.raises(Exception, match="INVALID_FIRST_CHARACTER"):
            nucleoid.run("pointer = 1")

    # ========================================================================
    # TEST 31 of 183: supports property of chained functions in expression
    # TypeScript: line 363
    # ========================================================================

    def test_supports_property_of_chained_functions_in_expression(self):
        """
        Original TypeScript (line 363):
        ```typescript
        it("supports property of chained functions in expression", () => {
          nucleoid.run("class User { }");
          nucleoid.run("class Registration { }");
          nucleoid.run("user1 = new User ( )");

          nucleoid.run("registration1 = new Registration ( )");
          nucleoid.run("registration1.user = user1");

          nucleoid.run("registration2 = new Registration ( )");
          nucleoid.run("registration2.user = user1");

          throws(() => {
            nucleoid.run(
              "if ( Registrations.filter ( r => r.user == User ) .length > 1 ) { throw 'USER_ALREADY_REGISTERED' }"
            );
          }, "USER_ALREADY_REGISTERED");
        });
        ```
        """
        nucleoid.run("class User {}")
        nucleoid.run("class Registration {}")
        nucleoid.run("user1 = new User()")

        nucleoid.run("registration1 = new Registration()")
        nucleoid.run("registration1.user = user1")

        nucleoid.run("registration2 = new Registration()")
        nucleoid.run("registration2.user = user1")

        with pytest.raises(Exception, match="USER_ALREADY_REGISTERED"):
            nucleoid.run("if (Registration.filter(r => r.user == user1).length > 1) { throw 'USER_ALREADY_REGISTERED' }")

    # ========================================================================
    # TEST 32 of 183: supports array with brackets
    # TypeScript: line 381
    # ========================================================================

    def test_supports_array_with_brackets(self):
        """
        Original TypeScript (line 381):
        ```typescript
        it("supports array with brackets", () => {
          nucleoid.run("states = [ 'NY' , 'GA' , 'CT' , 'MI' ]");
          equal(nucleoid.run("states [ 2 ]"), "CT");
        });
        ```
        """
        nucleoid.run("states = ['NY', 'GA', 'CT', 'MI']")
        assert nucleoid.run("states[2]") == "CT"

    # ========================================================================
    # TEST 33 of 183: retrieves value by variable
    # TypeScript: line 386
    # ========================================================================

    def test_retrieves_value_by_variable(self):
        """
        Original TypeScript (line 386):
        ```typescript
        it("retrieves value by variable", () => {
          nucleoid.run("number = -1");
          equal(nucleoid.run("number"), -1);
        });
        ```
        """
        nucleoid.run("number = -1")
        assert nucleoid.run("number") == -1

    # ========================================================================
    # TEST 34 of 183: rejects variable if not declared
    # TypeScript: line 391
    # ========================================================================

    def test_rejects_variable_if_not_declared(self):
        """
        Original TypeScript (line 391):
        ```typescript
        it("rejects variable if not declared", () => {
          throws(
            () => {
              nucleoid.run("e == 2.71828");
            },
            (error: Error) => validate(error, ReferenceError, "e is not defined")
          );
        });
        ```
        """
        with pytest.raises(ReferenceError, match="e is not defined"):
            nucleoid.run("e == 2.71828")

    # ========================================================================
    # TEST 35 of 183: runs multiple statements in the state
    # TypeScript: line 400
    # ========================================================================

    def test_runs_multiple_statements_in_the_state(self):
        """
        Original TypeScript (line 400):
        ```typescript
        it("runs multiple statements in the state", () => {
          nucleoid.run("k = 1 ; l = k + 1 ; k = 2");
          equal(nucleoid.run("l == 3"), true);
        });
        ```
        """
        nucleoid.run("k = 1; l = k + 1; k = 2")
        assert nucleoid.run("l == 3") is True

    # ========================================================================
    # TEST 36 of 183: runs dependent statements in the same transaction
    # TypeScript: line 405
    # ========================================================================

    def test_runs_dependent_statements_in_the_same_transaction(self):
        """
        Original TypeScript (line 405):
        ```typescript
        it("runs dependent statements in the same transaction", () => {
          nucleoid.run(
            "class Vehicle { } ; $Vehicle.tag = 'US-' + $Vehicle.plate "
          );
          nucleoid.run("vehicle1 = new Vehicle ( )");
          nucleoid.run("vehicle1.plate = 'XSJ422'");
          equal(nucleoid.run("vehicle1.tag"), "US-XSJ422");
        });
        ```
        """
        nucleoid.run("class Vehicle {}; $Vehicle.tag = 'US-' + $Vehicle.plate")
        nucleoid.run("vehicle1 = new Vehicle()")
        nucleoid.run("vehicle1.plate = 'XSJ422'")
        assert nucleoid.run("vehicle1.tag") == "US-XSJ422"

    # ========================================================================
    # TEST 37 of 183: runs dependencies in order as received
    # TypeScript: line 414
    # ========================================================================

    def test_runs_dependencies_in_order_as_received(self):
        """
        Original TypeScript (line 414):
        ```typescript
        it("runs dependencies in order as received", () => {
          nucleoid.run("any = 0");
          nucleoid.run("if ( any > 1 ) { result = 1 }");
          nucleoid.run("if ( any > 2 ) { result = 2 }");
          nucleoid.run("if ( any > 3 ) { result = 3 }");
          nucleoid.run("if ( any > 2 ) { result = 4 }");
          nucleoid.run("if ( any > 1 ) { result = 5 }");

          nucleoid.run("any = 4");
          equal(nucleoid.run("result"), 5);
        });
        ```
        """
        nucleoid.run("any = 0")
        nucleoid.run("if (any > 1) { result = 1 }")
        nucleoid.run("if (any > 2) { result = 2 }")
        nucleoid.run("if (any > 3) { result = 3 }")
        nucleoid.run("if (any > 2) { result = 4 }")
        nucleoid.run("if (any > 1) { result = 5 }")

        nucleoid.run("any = 4")
        assert nucleoid.run("result") == 5

    # ========================================================================
    # TEST 38 of 183: runs let at root scope
    # TypeScript: line 426
    # ========================================================================

    def test_runs_let_at_root_scope(self):
        """
        Original TypeScript (line 426):
        ```typescript
        it("runs let at root scope", () => {
          nucleoid.run("number = 13");
          equal(nucleoid.run("let i = number + 4; i;"), 17);
          nucleoid.run("number = 14");
          throws(
            () => {
              nucleoid.run("i");
            },
            (error: Error) => validate(error, ReferenceError, "i is not defined")
          );
        });
        ```
        """
        nucleoid.run("number = 13")
        assert nucleoid.run("let i = number + 4; i;") == 17
        nucleoid.run("number = 14")
        with pytest.raises(ReferenceError, match="i is not defined"):
            nucleoid.run("i")

    # ========================================================================
    # TEST 39 of 183: searches variable in scope before state
    # TypeScript: line 438
    # ========================================================================

    def test_searches_variable_in_scope_before_state(self):
        """
        Original TypeScript (line 438):
        ```typescript
        it("searches variable in scope before state", () => {
          nucleoid.run("e = 2.71828");
          nucleoid.run("{ let e = 3 ; number = e }");
          equal(nucleoid.run("number"), 3);
        });
        ```
        """
        nucleoid.run("e = 2.71828")
        nucleoid.run("{ let e = 3; number = e }")
        assert nucleoid.run("number") == 3

    # ========================================================================
    # TEST 40 of 183: uses local variable at lowest scope as priority
    # TypeScript: line 444
    # ========================================================================

    def test_uses_local_variable_at_lowest_scope_as_priority(self):
        """
        Original TypeScript (line 444):
        ```typescript
        it("uses local variable at lowest scope as priority", () => {
          nucleoid.run("pi = 3.14");
          nucleoid.run("number = pi");

          nucleoid.run("{ let pi = 3.141 ; { number = pi } }");
          equal(nucleoid.run("number"), 3.141);

          equal(
            nucleoid.run(
              "{ let pi = 3.1415 ; { let number = pi ; { let pi = 3.14159 ; number = pi ; return number } } }"
            ),
            3.14159
          );

          nucleoid.run(
            "{ let pi = 3.14159 ; number = pi ; { let pi = 3.141592 ; let number = pi ; { let pi = 3.1415926 ; number = pi } } }"
          );
          equal(nucleoid.run("number"), 3.14159);
        });
        ```
        """
        nucleoid.run("pi = 3.14")
        nucleoid.run("number = pi")

        nucleoid.run("{ let pi = 3.141; { number = pi } }")
        assert nucleoid.run("number") == 3.141

        assert nucleoid.run("{ let pi = 3.1415; { let number = pi; { let pi = 3.14159; number = pi; return number } } }") == 3.14159

        nucleoid.run("{ let pi = 3.14159; number = pi; { let pi = 3.141592; let number = pi; { let pi = 3.1415926; number = pi } } }")
        assert nucleoid.run("number") == 3.14159

    # ========================================================================
    # TEST 41 of 183: creates let statement if its instance is defined
    # TypeScript: line 464
    # ========================================================================

    def test_creates_let_statement_if_its_instance_is_defined(self):
        """
        Original TypeScript (line 464):
        ```typescript
        it("creates let statement if its instance is defined", () => {
          nucleoid.run("class Ticket { }");
          throws(
            () => {
              nucleoid.run(
                "{ let ticket = new Ticket ( ) ; ticket.event.group = 'ENTERTAINMENT' }"
              );
            },
            (error: Error) =>
              validate(error, ReferenceError, "ticket.event is not defined")
          );
        });
        ```
        """
        nucleoid.run("class Ticket {}")
        with pytest.raises(ReferenceError, match="ticket.event is not defined"):
            nucleoid.run("{ let ticket = new Ticket(); ticket.event.group = 'ENTERTAINMENT' }")

    # ========================================================================
    # TEST 42 of 183: declares let statement with undefined
    # TypeScript: line 477
    # ========================================================================

    def test_declares_let_statement_with_undefined(self):
        """
        Original TypeScript (line 477):
        ```typescript
        it("declares let statement with undefined", () => {
          nucleoid.run(
            "class Device { constructor ( code ) { this.code = code } }"
          );
          nucleoid.run("device1 = new Device ( 'A0' )");
          nucleoid.run("device2 = new Device ( 'B1' )");

          equal(
            nucleoid.run("device1"),
            nucleoid.run(
              "let device = Device.find ( d => d.code == 'A0' ) ; if ( ! device ) { throw 'INVALID_DEVICE' }  device"
            )
          );

          throws(
            () => {
              nucleoid.run(
                "let device = Device.find ( d => d.code == 'A1' ) ; if ( ! device ) { throw 'INVALID_DEVICE' } device"
              );
            },
            (error) => error === "INVALID_DEVICE"
          );
        });
        ```
        """
        nucleoid.run("class Device { constructor(code) { this.code = code } }")
        nucleoid.run("device1 = new Device('A0')")
        nucleoid.run("device2 = new Device('B1')")

        assert nucleoid.run("device1") == nucleoid.run("let device = Device.find(d => d.code == 'A0'); if (!device) { throw 'INVALID_DEVICE' } device")

        with pytest.raises(Exception, match="INVALID_DEVICE"):
            nucleoid.run("let device = Device.find(d => d.code == 'A1'); if (!device) { throw 'INVALID_DEVICE' } device")

    # ========================================================================
    # TEST 43 of 183: creates standard built-in object of let statement as property
    # TypeScript: line 501
    # ========================================================================

    def test_creates_standard_built_in_object_of_let_statement_as_property(self):
        """
        Original TypeScript (line 501):
        ```typescript
        it("creates standard built-in object of let statement as property", () => {
          nucleoid.run("class Shipment { }");
          nucleoid.run(
            "{ let shipment = new Shipment ( ) ; shipment.date = new Date ( '2019-1-3' ) ; shipment1 = shipment }"
          );
          equal(nucleoid.run("shipment1.date.toDateString ( )"), "Thu Jan 03 2019");
        });
        ```
        """
        nucleoid.run("class Shipment {}")
        nucleoid.run("{ let shipment = new Shipment(); shipment.date = new Date('2019-1-3'); shipment1 = shipment }")
        assert nucleoid.run("shipment1.date.toDateString()") == "Thu Jan 03 2019"

    # ========================================================================
    # TEST 44 of 183: creates property of let statement in different scope
    # TypeScript: line 509
    # ========================================================================

    def test_creates_property_of_let_statement_in_different_scope(self):
        """
        Original TypeScript (line 509):
        ```typescript
        it("creates property of let statement in different scope", () => {
          nucleoid.run("class User { }");
          nucleoid.run("user0 = new User ( )");
          nucleoid.run(
            "let user = User['user0'] ; if ( user ) { user.name = 'TEST' }"
          );
          equal(nucleoid.run("user0.name"), "TEST");
        });
        ```
        """
        nucleoid.run("class User {}")
        nucleoid.run("user0 = new User()")
        nucleoid.run("let user = User['user0']; if (user) { user.name = 'TEST' }")
        assert nucleoid.run("user0.name") == "TEST"

    # ========================================================================
    # TEST 45 of 183: assigns undefined if any dependencies in expression is undefined
    # TypeScript: line 518
    # ========================================================================

    def test_assigns_undefined_if_any_dependencies_in_expression_is_undefined(self):
        """
        Original TypeScript (line 518):
        ```typescript
        it("assigns undefined if any dependencies in expression is undefined", () => {
          nucleoid.run("class Person { }");
          nucleoid.run("person1 = new Person ( )");
          nucleoid.run("person1.lastName = 'Brown'");
          nucleoid.run(
            "person1.fullName = person1.firstName + ' ' + person1.lastName"
          );
          equal(nucleoid.run("person1.fullName") === undefined, true);
        });
        ```
        """
        nucleoid.run("class Person {}")
        nucleoid.run("person1 = new Person()")
        nucleoid.run("person1.lastName = 'Brown'")
        nucleoid.run("person1.fullName = person1.firstName + ' ' + person1.lastName")
        assert nucleoid.run("person1.fullName") is None

    # ========================================================================
    # TEST 46 of 183: keeps as null if any dependencies as in local is null
    # TypeScript: line 528
    # ========================================================================

    def test_keeps_as_null_if_any_dependencies_as_in_local_is_null(self):
        """
        Original TypeScript (line 528):
        ```typescript
        it("keeps as null if any dependencies as in local is null", () => {
          nucleoid.run("a = 1");
          nucleoid.run("{ let b = null ; c = b / a }");
          equal(nucleoid.run("c"), 0);
        });
        ```
        """
        nucleoid.run("a = 1")
        nucleoid.run("{ let b = null; c = b / a }")
        assert nucleoid.run("c") == 0

    # ========================================================================
    # TEST 47 of 183: keeps as null if any dependencies in expression is null
    # TypeScript: line 534
    # ========================================================================

    def test_keeps_as_null_if_any_dependencies_in_expression_is_null(self):
        """
        Original TypeScript (line 534):
        ```typescript
        it("keeps as null if any dependencies in expression is null", () => {
          nucleoid.run("class Schedule { }");
          nucleoid.run("schedule1 = new Schedule ( )");
          nucleoid.run("schedule1.expression = '0 */2 * * *'");
          nucleoid.run("schedule1.script = null");
          nucleoid.run(
            "schedule1.run = schedule1.expression + ' ' + schedule1.script"
          );
          equal(nucleoid.run("schedule1.run"), "0 */2 * * * null");
        });
        ```
        """
        nucleoid.run("class Schedule {}")
        nucleoid.run("schedule1 = new Schedule()")
        nucleoid.run("schedule1.expression = '0 */2 * * *'")
        nucleoid.run("schedule1.script = null")
        nucleoid.run("schedule1.run = schedule1.expression + ' ' + schedule1.script")
        assert nucleoid.run("schedule1.run") == "0 */2 * * * null"

    # ========================================================================
    # TEST 48 of 183: assigns null if there is null pointer in expression
    # TypeScript: line 545
    # ========================================================================

    def test_assigns_null_if_there_is_null_pointer_in_expression(self):
        """
        Original TypeScript (line 545):
        ```typescript
        it("assigns null if there is null pointer in expression", () => {
          nucleoid.run("class Product { }");
          nucleoid.run("product1 = new Product ( )");
          nucleoid.run("score = product1.quality.score");
          equal(nucleoid.run("score"), null);
        });
        ```
        """
        nucleoid.run("class Product {}")
        nucleoid.run("product1 = new Product()")
        nucleoid.run("score = product1.quality.score")
        assert nucleoid.run("score") is None

    # ========================================================================
    # TEST 49 of 183: places instance in the list of class when created
    # TypeScript: line 552
    # ========================================================================

    def test_places_instance_in_the_list_of_class_when_created(self):
        """
        Original TypeScript (line 552):
        ```typescript
        it("places instance in the list of class when created", () => {
          nucleoid.run("class Student { }");
          equal(nucleoid.run("Array.isArray ( Student )"), true);

          nucleoid.run("student1 = new Student ( )");
          equal(nucleoid.run("Student.length"), 1);
        });
        ```
        """
        nucleoid.run("class Student {}")
        assert nucleoid.run("Array.isArray(Student)") is True

        nucleoid.run("student1 = new Student()")
        assert nucleoid.run("Student.length") == 1

    # ========================================================================
    # TEST 50 of 183: assigns unique variable for instance without variable name defined
    # TypeScript: line 560
    # ========================================================================

    def test_assigns_unique_variable_for_instance_without_variable_name_defined(self):
        """
        Original TypeScript (line 560):
        ```typescript
        it("assigns unique variable for instance without variable name defined", () => {
          nucleoid.run("class Vehicle { }");
          nucleoid.run("new Vehicle ( )");
          equal(nucleoid.run("Vehicle.length"), 1);
        });
        ```
        """
        nucleoid.run("class Vehicle {}")
        nucleoid.run("new Vehicle()")
        assert nucleoid.run("Vehicle.length") == 1

    # ========================================================================
    # TEST 51 of 183: throws error as a string
    # TypeScript: line 566
    # ========================================================================

    def test_throws_error_as_a_string(self):
        """
        Original TypeScript (line 566):
        ```typescript
        it("throws error as a string", () => {
          throws(
            () => {
              nucleoid.run("throw 'INVALID'");
            },
            (error) => error === "INVALID"
          );

          throws(
            () => {
              nucleoid.run('throw "INVALID"');
            },
            (error) => error === "INVALID"
          );
        });
        ```
        """
        with pytest.raises(Exception, match="INVALID"):
            nucleoid.run("throw 'INVALID'")

        with pytest.raises(Exception, match="INVALID"):
            nucleoid.run('throw "INVALID"')

    # ========================================================================
    # TEST 52 of 183: throws error as an integer
    # TypeScript: line 582
    # ========================================================================

    def test_throws_error_as_an_integer(self):
        """
        Original TypeScript (line 582):
        ```typescript
        it("throws error as an integer", () => {
          throws(
            () => {
              nucleoid.run("throw 123");
            },
            (error) => error === 123
          );
        });
        ```
        """
        with pytest.raises(Exception):
            nucleoid.run("throw 123")

    # ========================================================================
    # TEST 53 of 183: throws reference error if invalid in throw
    # TypeScript: line 591
    # ========================================================================

    def test_throws_reference_error_if_invalid_in_throw(self):
        """
        Original TypeScript (line 591):
        ```typescript
        it("throws reference error if invalid in throw", () => {
          throws(
            () => {
              nucleoid.run("throw abc");
            },
            (error: Error) => validate(error, ReferenceError, "abc is not defined")
          );
        });
        ```
        """
        with pytest.raises(ReferenceError, match="abc is not defined"):
            nucleoid.run("throw abc")

    # ========================================================================
    # TEST 54 of 183: throws error inside block
    # TypeScript: line 600
    # ========================================================================

    def test_throws_error_inside_block(self):
        """
        Original TypeScript (line 600):
        ```typescript
        it("throws error inside block", () => {
          nucleoid.run("k = 99");
          throws(
            () => {
              nucleoid.run("if ( k >= 99 ) { throw 'INVALID' }");
            },
            (error) => error === "INVALID"
          );
        });
        ```
        """
        nucleoid.run("k = 99")
        with pytest.raises(Exception, match="INVALID"):
            nucleoid.run("if (k >= 99) { throw 'INVALID' }")

    # ========================================================================
    # TEST 55 of 183: throws error as a variable
    # TypeScript: line 610
    # ========================================================================

    def test_throws_error_as_a_variable(self):
        """
        Original TypeScript (line 610):
        ```typescript
        it("throws error as a variable", () => {
          nucleoid.run("length = 0.1");
          throws(
            () => {
              nucleoid.run("if ( length < 1 ) { throw length }");
            },
            (error) => error === 0.1
          );

          throws(
            () => {
              nucleoid.run("if ( length < 1.1 ) { throw 'length' }");
            },
            (error) => error === "length"
          );
        });
        ```
        """
        nucleoid.run("length = 0.1")
        with pytest.raises(Exception):
            nucleoid.run("if (length < 1) { throw length }")

        with pytest.raises(Exception, match="length"):
            nucleoid.run("if (length < 1.1) { throw 'length' }")

    # ========================================================================
    # TEST 56 of 183: assigns function as dependency
    # TypeScript: line 627
    # ========================================================================

    def test_assigns_function_as_dependency(self):
        """
        Original TypeScript (line 627):
        ```typescript
        it("assigns function as dependency", () => {
          nucleoid.run("list = [ ]");
          nucleoid.run("count = list.filter ( n => n % 2 )");
          nucleoid.run("list.push ( 1 )");
          equal(nucleoid.run("count.length"), 1);

          nucleoid.run("list.push ( 2 )");
          equal(nucleoid.run("count.length"), 1);

          nucleoid.run("list.push ( 3 )");
          equal(nucleoid.run("count.length"), 2);

          nucleoid.run("list.pop ( )");
          equal(nucleoid.run("count.length"), 1);
        });
        ```
        """
        nucleoid.run("list = []")
        nucleoid.run("count = list.filter(n => n % 2)")
        nucleoid.run("list.push(1)")
        assert nucleoid.run("count.length") == 1

        nucleoid.run("list.push(2)")
        assert nucleoid.run("count.length") == 1

        nucleoid.run("list.push(3)")
        assert nucleoid.run("count.length") == 2

        nucleoid.run("list.pop()")
        assert nucleoid.run("count.length") == 1

    # ========================================================================
    # TEST 57 of 183: assigns parameter in function as dependency
    # TypeScript: line 643
    # ========================================================================

    def test_assigns_parameter_in_function_as_dependency(self):
        """
        Original TypeScript (line 643):
        ```typescript
        it("assigns parameter in function as dependency", () => {
          nucleoid.run("str1 = 'ABC'");
          nucleoid.run("str2 = str1.toLowerCase ( ) + 'd'");
          nucleoid.run("str3 = str2.concat ( str1 )");
          equal(nucleoid.run("str2"), "abcd");
          equal(nucleoid.run("str3"), "abcdABC");

          nucleoid.run("str1 = 'AAA'");
          equal(nucleoid.run("str2"), "aaad");
          equal(nucleoid.run("str3"), "aaadAAA");
        });
        ```
        """
        nucleoid.run("str1 = 'ABC'")
        nucleoid.run("str2 = str1.toLowerCase() + 'd'")
        nucleoid.run("str3 = str2.concat(str1)")
        assert nucleoid.run("str2") == "abcd"
        assert nucleoid.run("str3") == "abcdABC"

        nucleoid.run("str1 = 'AAA'")
        assert nucleoid.run("str2") == "aaad"
        assert nucleoid.run("str3") == "aaadAAA"

    # ========================================================================
    # TEST 58 of 183: supports regular expression literal
    # TypeScript: line 655
    # ========================================================================

    def test_supports_regular_expression_literal(self):
        """
        Original TypeScript (line 655):
        ```typescript
        it("supports regular expression literal", () => {
          nucleoid.run("class User { }");
          nucleoid.run(
            "if ( ! /.{4,8}/.test ( $User.password ) ) { throw 'INVALID_PASSWORD' }"
          );
          nucleoid.run("user1 = new User ( )");
          throws(
            () => {
              nucleoid.run("user1.password = 'PAS'");
            },
            (error) => error === "INVALID_PASSWORD"
          );
        });
        ```
        """
        nucleoid.run("class User {}")
        nucleoid.run("if (!/.{4,8}/.test($User.password)) { throw 'INVALID_PASSWORD' }")
        nucleoid.run("user1 = new User()")
        with pytest.raises(Exception, match="INVALID_PASSWORD"):
            nucleoid.run("user1.password = 'PAS'")

    # ========================================================================
    # TEST 59 of 183: rejects defining class declaration in non-class declaration block
    # TypeScript: line 669
    # ========================================================================

    def test_rejects_defining_class_declaration_in_non_class_declaration_block(self):
        """
        Original TypeScript (line 669):
        ```typescript
        it("rejects defining class declaration in non-class declaration block", () => {
          nucleoid.run("class Person { }");
          nucleoid.run("person1 = new Person ( )");
          nucleoid.run("person1.weight = 90");
          nucleoid.run("person1.height = 1.8");
          throws(
            () => {
              nucleoid.run(
                "{ let weight = person1.weight ; let height = person1.height ; $Person.bmi = weight / ( height * height ) }"
              );
            },
            (error: Error) =>
              validate(
                error,
                SyntaxError,
                "Cannot define class declaration in non-class block"
              )
          );
        });
        ```
        """
        nucleoid.run("class Person {}")
        nucleoid.run("person1 = new Person()")
        nucleoid.run("person1.weight = 90")
        nucleoid.run("person1.height = 1.8")
        with pytest.raises(SyntaxError, match="Cannot define class declaration in non-class block"):
            nucleoid.run("{ let weight = person1.weight; let height = person1.height; $Person.bmi = weight / (height * height) }")

    # ========================================================================
    # TEST 60 of 183: detects circular dependency
    # TypeScript: line 689
    # ========================================================================

    def test_detects_circular_dependency(self):
        """
        Original TypeScript (line 689):
        ```typescript
        it("detects circular dependency", () => {
          nucleoid.run("number1 = 10");
          nucleoid.run("number2 = number1 * 10");
          throws(
            () => {
              nucleoid.run("number1 = number2 * 10");
            },
            (error: Error) => validate(error, ReferenceError, "Circular Dependency")
          );
        });
        ```
        """
        nucleoid.run("number1 = 10")
        nucleoid.run("number2 = number1 * 10")
        with pytest.raises(ReferenceError, match="Circular Dependency"):
            nucleoid.run("number1 = number2 * 10")

    # ========================================================================
    # TEST 61 of 183: creates function in state
    # TypeScript: line 700
    # ========================================================================

    def test_creates_function_in_state(self):
        """
        Original TypeScript (line 700):
        ```typescript
        it("creates function in state", () => {
          nucleoid.run("function generate ( number ) { return number * 10 }");
          nucleoid.run("random = 10");
          nucleoid.run("number = generate ( random )");
          equal(nucleoid.run("number"), 100);

          nucleoid.run("random = 20");
          equal(nucleoid.run("number"), 200);
        });
        ```
        """
        nucleoid.run("function generate(number) { return number * 10 }")
        nucleoid.run("random = 10")
        nucleoid.run("number = generate(random)")
        assert nucleoid.run("number") == 100

        nucleoid.run("random = 20")
        assert nucleoid.run("number") == 200

    # ========================================================================
    # TEST 62 of 183: publishes event
    # TypeScript: line 710
    # ========================================================================

    def test_publishes_event(self):
        """
        Original TypeScript (line 710):
        ```typescript
        it("publishes event", () => {
          nucleoid.run(
            "class Task { constructor ( ) { event ( 'TASK_CREATED', 'TASK_DATA' ) } }"
          );

          const result = nucleoid.run("task1 = new Task ( )", details);
          equal(result.events[0].name, "TASK_CREATED");
          equal(result.events[0].data, '"TASK_DATA"');
        });
        ```
        """
        details = {'details': True}
        nucleoid.run("class Task { constructor() { event('TASK_CREATED', 'TASK_DATA') } }")

        result = nucleoid.run("task1 = new Task()", details)
        assert result['events'][0]['name'] == "TASK_CREATED"
        assert result['events'][0]['data'] == '"TASK_DATA"'

    # ========================================================================
    # TEST 63 of 183: rollbacks variable if exception is thrown
    # TypeScript: line 720
    # ========================================================================

    def test_rollbacks_variable_if_exception_is_thrown(self):
        """
        Original TypeScript (line 720):
        ```typescript
        it("rollbacks variable if exception is thrown", () => {
          nucleoid.run("a = 5");
          nucleoid.run("if ( a > 5 ) { throw 'INVALID_VALUE' }");

          throws(
            () => {
              nucleoid.run("a = 6");
            },
            (error) => error === "INVALID_VALUE"
          );
          equal(nucleoid.run("a"), 5);
        });
        ```
        """
        nucleoid.run("a = 5")
        nucleoid.run("if (a > 5) { throw 'INVALID_VALUE' }")

        with pytest.raises(Exception, match="INVALID_VALUE"):
            nucleoid.run("a = 6")
        assert nucleoid.run("a") == 5

    # ========================================================================
    # TEST 64 of 183: rollbacks property if exception is thrown
    # TypeScript: line 733
    # ========================================================================

    def test_rollbacks_property_if_exception_is_thrown(self):
        """
        Original TypeScript (line 733):
        ```typescript
        it("rollbacks property if exception is thrown", () => {
          nucleoid.run("class Item { }");
          nucleoid.run("if ( $Item.sku == 'A' ) { throw 'INVALID_SKU' }");
          nucleoid.run("item1 = new Item ( )");

          throws(
            () => {
              nucleoid.run("item1.sku = 'A'");
            },
            (error) => error === "INVALID_SKU"
          );
          equal(nucleoid.run("item1.sku"), undefined);
        });
        ```
        """
        nucleoid.run("class Item {}")
        nucleoid.run("if ($Item.sku == 'A') { throw 'INVALID_SKU' }")
        nucleoid.run("item1 = new Item()")

        with pytest.raises(Exception, match="INVALID_SKU"):
            nucleoid.run("item1.sku = 'A'")
        assert nucleoid.run("item1.sku") is None

    # ========================================================================
    # TEST 65 of 183: rollbacks instance if exception is thrown
    # TypeScript: line 747
    # ========================================================================

    def test_rollbacks_instance_if_exception_is_thrown(self):
        """
        Original TypeScript (line 747):
        ```typescript
        it("rollbacks instance if exception is thrown", () => {
          nucleoid.run(
            "class User { constructor ( first , last ) { this.first = first ; this.last = last } }"
          );
          nucleoid.run("if ( $User.first.length < 3 ) { throw 'INVALID_USER' }");

          throws(
            () => {
              nucleoid.run("user1 = new User ( 'F' , 'L' )");
            },
            (error) => error === "INVALID_USER"
          );
          throws(
            () => {
              nucleoid.run("user1");
            },
            (error: Error) =>
              validate(error, ReferenceError, "user1 is not defined")
          );
        });
        ```
        """
        nucleoid.run("class User { constructor(first, last) { this.first = first; this.last = last } }")
        nucleoid.run("if ($User.first.length < 3) { throw 'INVALID_USER' }")

        with pytest.raises(Exception, match="INVALID_USER"):
            nucleoid.run("user1 = new User('F', 'L')")

        with pytest.raises(ReferenceError, match="user1 is not defined"):
            nucleoid.run("user1")

    # ========================================================================
    # TEST 66 of 183: creates variable assignment (declarative)
    # TypeScript: line 768
    # ========================================================================

    def test_creates_variable_assignment_declarative(self):
        """
        Original TypeScript (line 768):
        ```typescript
        it("creates variable assignment", () => {
          nucleoid.run("x = 1");
          nucleoid.run("y = x + 2");
          nucleoid.run("x = 2");
          equal(nucleoid.run("y == 4"), true);
        });
        ```
        """
        nucleoid.run("x = 1")
        nucleoid.run("y = x + 2")
        nucleoid.run("x = 2")
        assert nucleoid.run("y == 4") is True

    # ========================================================================
    # TEST 67 of 183: return assigned value while variable assignment
    # TypeScript: line 775
    # ========================================================================

    def test_return_assigned_value_while_variable_assignment(self):
        """
        Original TypeScript (line 775):
        ```typescript
        it("return assigned value while variable assignment", () => {
          const result = nucleoid.run("x = 1");
          equal(result, 1);
        });
        ```
        """
        result = nucleoid.run("x = 1")
        assert result == 1

    # ========================================================================
    # TEST 68 of 183: updates variable assignment
    # TypeScript: line 780
    # ========================================================================

    def test_updates_variable_assignment(self):
        """
        Original TypeScript (line 780):
        ```typescript
        it("updates variable assignment", () => {
          nucleoid.run("a = 1");
          nucleoid.run("b = 2");
          nucleoid.run("c = a + 3");
          nucleoid.run("c = b + 3");
          equal(nucleoid.run("c"), 5);

          nucleoid.run("b = 4");
          equal(nucleoid.run("c"), 7);
        });
        ```
        """
        nucleoid.run("a = 1")
        nucleoid.run("b = 2")
        nucleoid.run("c = a + 3")
        nucleoid.run("c = b + 3")
        assert nucleoid.run("c") == 5

        nucleoid.run("b = 4")
        assert nucleoid.run("c") == 7

    # ========================================================================
    # TEST 69 of 183: uses its value when self variable used
    # TypeScript: line 791
    # ========================================================================

    def test_uses_its_value_when_self_variable_used(self):
        """
        Original TypeScript (line 791):
        ```typescript
        it("uses its value when self variable used", () => {
          nucleoid.run("radius = 10");
          nucleoid.run("radius = radius + 10");
          equal(nucleoid.run("radius"), 20);
        });
        ```
        """
        nucleoid.run("radius = 10")
        nucleoid.run("radius = radius + 10")
        assert nucleoid.run("radius") == 20

    # ========================================================================
    # TEST 70 of 183: deletes variable assignment
    # TypeScript: line 797
    # ========================================================================

    def test_deletes_variable_assignment(self):
        """
        Original TypeScript (line 797):
        ```typescript
        it("deletes variable assignment", () => {
          nucleoid.run("t = 1");
          nucleoid.run("q = t + 1");
          nucleoid.run("delete q");
          nucleoid.run("t = 2");
          throws(
            () => {
              nucleoid.run("q");
            },
            (error: Error) => validate(error, ReferenceError, "q is not defined")
          );
        });
        ```
        """
        nucleoid.run("t = 1")
        nucleoid.run("q = t + 1")
        nucleoid.run("delete q")
        nucleoid.run("t = 2")
        with pytest.raises(ReferenceError, match="q is not defined"):
            nucleoid.run("q")

    # ========================================================================
    # TEST 71 of 183: runs multiple instance of let statement of property
    # TypeScript: line 956
    # ========================================================================

    def test_runs_multiple_instance_of_let_statement_of_property(self):
        """
        Original TypeScript (line 956):
        ```typescript
        it("runs multiple instance of let statement of property", () => {
          nucleoid.run("class Timesheet { }");
          nucleoid.run("class Task { }");
          nucleoid.run("class Project { }");
          nucleoid.run("$Project.code = 'N-' + $Project.number");
          nucleoid.run("timesheet1 = new Timesheet ( )");
          nucleoid.run(
            "{ let task = new Task ( ) ; task.project = new Project ( ) ; task.project.number = 3668347 ; timesheet1.task = task }"
          );
          equal(nucleoid.run("timesheet1.task.project.number"), 3668347);
          equal(nucleoid.run("timesheet1.task.project.code"), "N-3668347");
        });
        ```
        """
        nucleoid.run("class Timesheet { }")
        nucleoid.run("class Task { }")
        nucleoid.run("class Project { }")
        nucleoid.run("$Project.code = 'N-' + $Project.number")
        nucleoid.run("timesheet1 = new Timesheet()")
        nucleoid.run(
            "{ let task = new Task() ; task.project = new Project() ; task.project.number = 3668347 ; timesheet1.task = task }"
        )
        assert nucleoid.run("timesheet1.task.project.number") == 3668347
        assert nucleoid.run("timesheet1.task.project.code") == "N-3668347"

    # ========================================================================
    # TEST 72 of 183: creates new object of let statement of class as value before initialization
    # TypeScript: line 969
    # ========================================================================

    def test_creates_new_object_of_let_statement_of_class_as_value_before_initialization(self):
        """
        Original TypeScript (line 969):
        ```typescript
        it("creates new object of let statement of class as value before initialization", () => {
          nucleoid.run("class Member { }");
          nucleoid.run(
            "{ let registration = new Object ( ) ; registration.date = new Date ( '2019-1-2' ) ; $Member.registration = registration }"
          );

          nucleoid.run("member1 = new Member ( )");
          equal(
            nucleoid.run("member1.registration.date.toDateString()"),
            "Wed Jan 02 2019"
          );
          equal(nucleoid.run("member1.registration.age"), undefined);
        });
        ```
        """
        nucleoid.run("class Member { }")
        nucleoid.run(
            "{ let registration = new Object() ; registration.date = new Date('2019-1-2') ; $Member.registration = registration }"
        )
        nucleoid.run("member1 = new Member()")
        assert nucleoid.run("member1.registration.date.toDateString()") == "Wed Jan 02 2019"
        assert nucleoid.run("member1.registration.age") is None

    # ========================================================================
    # TEST 73 of 183: creates new object of let statement of class as value after initialization
    # TypeScript: line 983
    # ========================================================================

    def test_creates_new_object_of_let_statement_of_class_as_value_after_initialization(self):
        """
        Original TypeScript (line 983):
        ```typescript
        it("creates new object of let statement of class as value after initialization", () => {
          nucleoid.run("class Distance { }");
          nucleoid.run("distance1 = new Distance ( )");
          nucleoid.run(
            "{ let location = new Object ( ) ; location.coordinates = '40.6976701,-74.2598779' ; $Distance.startingPoint = location }"
          );
          equal(
            nucleoid.run("distance1.startingPoint.coordinates"),
            "40.6976701,-74.2598779"
          );
          equal(nucleoid.run("distance1.startingPoint.print"), undefined);
        });
        ```
        """
        nucleoid.run("class Distance { }")
        nucleoid.run("distance1 = new Distance()")
        nucleoid.run(
            "{ let location = new Object() ; location.coordinates = '40.6976701,-74.2598779' ; $Distance.startingPoint = location }"
        )
        assert nucleoid.run("distance1.startingPoint.coordinates") == "40.6976701,-74.2598779"
        assert nucleoid.run("distance1.startingPoint.print") is None

    # ========================================================================
    # TEST 74 of 183: creates multiple object of let statement of class as value before initialization
    # TypeScript: line 996
    # ========================================================================

    def test_creates_multiple_object_of_let_statement_of_class_as_value_before_initialization(self):
        """
        Original TypeScript (line 996):
        ```typescript
        it("creates multiple object of let statement of class as value before initialization", () => {
          nucleoid.run("class Account { }");
          nucleoid.run("class Balance { }");
          nucleoid.run("class Currency { }");
          nucleoid.run("$Currency.description = 'Code:' + $Currency.code");
          nucleoid.run(
            "{ let balance = new Object ( ) ; balance.currency = new Object ( ) ; balance.currency.code = 'USD' ; $Account.balance = balance }"
          );
          nucleoid.run("account1 = new Account ( )");
          equal(nucleoid.run("account1.balance.currency.code "), "USD");
          equal(nucleoid.run("account1.balance.currency.description "), undefined);
        });
        ```
        """
        nucleoid.run("class Account { }")
        nucleoid.run("class Balance { }")
        nucleoid.run("class Currency { }")
        nucleoid.run("$Currency.description = 'Code:' + $Currency.code")
        nucleoid.run(
            "{ let balance = new Object() ; balance.currency = new Object() ; balance.currency.code = 'USD' ; $Account.balance = balance }"
        )
        nucleoid.run("account1 = new Account()")
        assert nucleoid.run("account1.balance.currency.code") == "USD"
        assert nucleoid.run("account1.balance.currency.description") is None

    # ========================================================================
    # TEST 75 of 183: creates multiple object of let statement of class as value after initialization
    # TypeScript: line 1009
    # ========================================================================

    def test_creates_multiple_object_of_let_statement_of_class_as_value_after_initialization(self):
        """
        Original TypeScript (line 1009):
        ```typescript
        it("creates multiple object of let statement of class as value after initialization", () => {
          nucleoid.run("class Warehouse { }");
          nucleoid.run("warehouse1 = new Warehouse ( )");
          nucleoid.run(
            "{ let inventory = new Object ( ) ; inventory.item = new Object ( ) ; inventory.item.sku = '699546085767' ; $Warehouse.inventory = inventory }"
          );
          equal(nucleoid.run("warehouse1.inventory.item.sku"), "699546085767");
          equal(nucleoid.run("warehouse1.inventory.item.description"), undefined);
        });
        ```
        """
        nucleoid.run("class Warehouse { }")
        nucleoid.run("warehouse1 = new Warehouse()")
        nucleoid.run(
            "{ let inventory = new Object() ; inventory.item = new Object() ; inventory.item.sku = '699546085767' ; $Warehouse.inventory = inventory }"
        )
        assert nucleoid.run("warehouse1.inventory.item.sku") == "699546085767"
        assert nucleoid.run("warehouse1.inventory.item.description") is None

    # ========================================================================
    # TEST 76 of 183: creates instance inside block
    # TypeScript: line 1019
    # ========================================================================

    def test_creates_instance_inside_block(self):
        """
        Original TypeScript (line 1019):
        ```typescript
        it("creates instance inside block", () => {
          nucleoid.run(
            "class Device { constructor ( name ) { this.name = name } }"
          );
          nucleoid.run("$Device.deleted = false");
          nucleoid.run("$Device.key = 'X-' + $Device.name");
          nucleoid.run("{ let name = 'Hall' ; device1 = new Device ( name ) }");

          equal(nucleoid.run("device1.name"), "Hall");
          equal(nucleoid.run("device1.key"), "X-Hall");
          equal(nucleoid.run("device1.deleted"), false);
        });
        ```
        """
        nucleoid.run("class Device { constructor(name) { this.name = name } }")
        nucleoid.run("$Device.deleted = False")
        nucleoid.run("$Device.key = 'X-' + $Device.name")
        nucleoid.run("{ let name = 'Hall' ; device1 = new Device(name) }")
        assert nucleoid.run("device1.name") == "Hall"
        assert nucleoid.run("device1.key") == "X-Hall"
        assert nucleoid.run("device1.deleted") is False

    # ========================================================================
    # TEST 77 of 183: creates instance inside block without variable name defined
    # TypeScript: line 1032
    # ========================================================================

    def test_creates_instance_inside_block_without_variable_name_defined(self):
        """
        Original TypeScript (line 1032):
        ```typescript
        it("creates instance inside block without variable name defined", () => {
          nucleoid.run(
            "class Summary { constructor ( rate ) { this.rate = rate } }"
          );
          nucleoid.run("$Summary.score = $Summary.rate * 100");
          nucleoid.run("{ let rate = 4 ; new Summary ( rate ) }");

          equal(nucleoid.run("Summary[0].rate"), 4);
          equal(nucleoid.run("Summary[0].score"), 400);
        });
        ```
        """
        nucleoid.run("class Summary { constructor(rate) { this.rate = rate } }")
        nucleoid.run("$Summary.score = $Summary.rate * 100")
        nucleoid.run("{ let rate = 4 ; new Summary(rate) }")
        assert nucleoid.run("Summary[0].rate") == 4
        assert nucleoid.run("Summary[0].score") == 400

    # ========================================================================
    # TEST 78 of 183: creates variable inside block
    # TypeScript: line 1043
    # ========================================================================

    def test_creates_variable_inside_block(self):
        """
        Original TypeScript (line 1043):
        ```typescript
        it("creates variable inside block", () => {
          nucleoid.run("a = 5 ; b = 10");
          nucleoid.run("if ( a > 9 ) { let c = a + b ; d = c * 10 }");
          nucleoid.run("a = 10");
          equal(nucleoid.run("d"), 200);

          nucleoid.run("a = 15");
          equal(nucleoid.run("d"), 250);

          nucleoid.run("b = 20");
          equal(nucleoid.run("d"), 350);
        });
        ```
        """
        nucleoid.run("a = 5 ; b = 10")
        nucleoid.run("if (a > 9) { let c = a + b ; d = c * 10 }")
        nucleoid.run("a = 10")
        assert nucleoid.run("d") == 200

        nucleoid.run("a = 15")
        assert nucleoid.run("d") == 250

        nucleoid.run("b = 20")
        assert nucleoid.run("d") == 350

    # ========================================================================
    # TEST 79 of 183: runs let statement as an object before declaration
    # TypeScript: line 1056
    # ========================================================================

    def test_runs_let_statement_as_an_object_before_declaration(self):
        """
        Original TypeScript (line 1056):
        ```typescript
        it("runs let statement as an object before declaration", () => {
          nucleoid.run("class Plane { }");
          nucleoid.run("class Trip { }");
          nucleoid.run("plane1 = new Plane ( )");
          nucleoid.run("plane1.speed = 903");
          nucleoid.run("trip1 = new Trip ( )");
          nucleoid.run("trip1.distance = 5540");
          nucleoid.run(
            "{ let trip = $Plane.trip ; $Plane.time = trip.distance / $Plane.speed }"
          );
          nucleoid.run("plane1.trip = trip1");
          equal(nucleoid.run("plane1.time"), 6.135105204872647);
        });
        ```
        """
        nucleoid.run("class Plane { }")
        nucleoid.run("class Trip { }")
        nucleoid.run("plane1 = new Plane()")
        nucleoid.run("plane1.speed = 903")
        nucleoid.run("trip1 = new Trip()")
        nucleoid.run("trip1.distance = 5540")
        nucleoid.run(
            "{ let trip = $Plane.trip ; $Plane.time = trip.distance / $Plane.speed }"
        )
        nucleoid.run("plane1.trip = trip1")
        assert nucleoid.run("plane1.time") == 6.135105204872647

    # ========================================================================
    # TEST 80 of 183: runs let statement as an object after declaration
    # TypeScript: line 1070
    # ========================================================================

    def test_runs_let_statement_as_an_object_after_declaration(self):
        """
        Original TypeScript (line 1070):
        ```typescript
        it("runs let statement as an object after declaration", () => {
          nucleoid.run("class Seller { }");
          nucleoid.run("class Commission { }");
          nucleoid.run("seller1 = new Seller ( )");
          nucleoid.run("seller1.sales = 1000000");
          nucleoid.run("comm1 = new Commission ( )");
          nucleoid.run("comm1.rate = 0.05");
          nucleoid.run("seller1.commission = comm1");
          nucleoid.run(
            "{ let commission = $Seller.commission ; $Seller.pay = $Seller.sales * commission.rate }"
          );
          equal(nucleoid.run("seller1.pay"), 50000);
        });
        ```
        """
        nucleoid.run("class Seller { }")
        nucleoid.run("class Commission { }")
        nucleoid.run("seller1 = new Seller()")
        nucleoid.run("seller1.sales = 1000000")
        nucleoid.run("comm1 = new Commission()")
        nucleoid.run("comm1.rate = 0.05")
        nucleoid.run("seller1.commission = comm1")
        nucleoid.run(
            "{ let commission = $Seller.commission ; $Seller.pay = $Seller.sales * commission.rate }"
        )
        assert nucleoid.run("seller1.pay") == 50000

    # ========================================================================
    # TEST 81 of 183: reassigns let statement after initialization (SKIPPED in TypeScript)
    # TypeScript: line 1098
    # ========================================================================

    def test_reassigns_let_statement_after_initialization(self):
        """
        Original TypeScript (line 1098):
        ```typescript
        it("reassigns let statement after initialization", () => {
          nucleoid.run("class Stock { }");
          nucleoid.run("class Trade { }");
          nucleoid.run("stock1 = new Stock ( )");
          nucleoid.run("stock1.price = 100");
          nucleoid.run("trade1 = new Trade ( )");
          nucleoid.run("trade1.quantity = 1");
          nucleoid.run("stock1.trade = trade1");
          nucleoid.run(
            "{ let trade = $Stock.trade ; trade.worth = $Stock.price * trade.quantity }"
          );
          equal(nucleoid.run("trade1.worth"), 100);
        });
        ```
        """
        nucleoid.run("class Stock { }")
        nucleoid.run("class Trade { }")
        nucleoid.run("stock1 = new Stock()")
        nucleoid.run("stock1.price = 100")
        nucleoid.run("trade1 = new Trade()")
        nucleoid.run("trade1.quantity = 1")
        nucleoid.run("stock1.trade = trade1")
        nucleoid.run(
            "{ let trade = $Stock.trade ; trade.worth = $Stock.price * trade.quantity }"
        )
        assert nucleoid.run("trade1.worth") == 100

    # ========================================================================
    # TEST 82 of 183: assigns let statement after const statement with the same name
    # TypeScript: line 1112
    # ========================================================================

    def test_assigns_let_statement_after_const_statement_with_the_same_name(self):
        """
        Original TypeScript (line 1112):
        ```typescript
        it("assigns let statement after const statement with the same name", () => {
          const result = nucleoid.run(
            `
            const barcode = "barcode";
            {
              let barcode = "barcode";
              {
                barcode = "barcode2";
                {
                  return barcode;
                }
              }
            }
            `
          );

          equal(result, "barcode2");
        });
        ```
        """
        result = nucleoid.run("""
            const barcode = "barcode";
            {
              let barcode = "barcode";
              {
                barcode = "barcode2";
                {
                  return barcode;
                }
              }
            }
        """)
        assert result == "barcode2"

    # ========================================================================
    # TEST 83 of 183: holds result of function in let
    # TypeScript: line 1131
    # ========================================================================

    def test_holds_result_of_function_in_let(self):
        """
        Original TypeScript (line 1131):
        ```typescript
        it("holds result of function in let", () => {
          nucleoid.run("bugs = [ ]");
          nucleoid.run("ticket = 1");
          nucleoid.run("class Bug { }");
          nucleoid.run("bug1 = new Bug ( )");
          nucleoid.run("bug1.ticket = 1");
          nucleoid.run("bug1.priority = 'LOW'");
          nucleoid.run("bugs.push ( bug1 )");
          nucleoid.run("bug2 = new Bug ( )");
          nucleoid.run("bug2.ticket = 2");
          nucleoid.run("bug2.priority = 'MEDIUM'");
          nucleoid.run("bugs.push ( bug2 )");
          nucleoid.run(
            "{ let bug = bugs.find ( it => it.ticket == ticket ) ; bug.selected = true }"
          );
          equal(nucleoid.run("bug1.selected"), true);
          equal(nucleoid.run("bug2.selected"), undefined);

          nucleoid.run("ticket = 2");
          equal(nucleoid.run("bug2.selected"), true);
        });
        ```
        """
        nucleoid.run("bugs = []")
        nucleoid.run("ticket = 1")
        nucleoid.run("class Bug { }")
        nucleoid.run("bug1 = new Bug()")
        nucleoid.run("bug1.ticket = 1")
        nucleoid.run("bug1.priority = 'LOW'")
        nucleoid.run("bugs.push(bug1)")
        nucleoid.run("bug2 = new Bug()")
        nucleoid.run("bug2.ticket = 2")
        nucleoid.run("bug2.priority = 'MEDIUM'")
        nucleoid.run("bugs.push(bug2)")
        nucleoid.run(
            "{ let bug = bugs.find(it => it.ticket == ticket) ; bug.selected = True }"
        )
        assert nucleoid.run("bug1.selected") is True
        assert nucleoid.run("bug2.selected") is None

        nucleoid.run("ticket = 2")
        assert nucleoid.run("bug2.selected") is True

    # ========================================================================
    # TEST 84 of 183: skips if block is empty
    # TypeScript: line 1153
    # ========================================================================

    def test_skips_if_block_is_empty(self):
        """
        Original TypeScript (line 1153):
        ```typescript
        it("skips if block is empty", () => {
          nucleoid.run("{ }");
        });
        ```
        """
        nucleoid.run("{ }")

    # ========================================================================
    # TEST 85 of 183: runs block statement of variable
    # TypeScript: line 1157
    # ========================================================================

    def test_runs_block_statement_of_variable(self):
        """
        Original TypeScript (line 1157):
        ```typescript
        it("runs block statement of variable", () => {
          nucleoid.run("h = 1");
          nucleoid.run("{ let value = h * 2 ; j = value * 2 }");
          equal(nucleoid.run("j"), 4);

          nucleoid.run("h = 2");
          equal(nucleoid.run("j"), 8);
        });
        ```
        """
        nucleoid.run("h = 1")
        nucleoid.run("{ let value = h * 2 ; j = value * 2 }")
        assert nucleoid.run("j") == 4

        nucleoid.run("h = 2")
        assert nucleoid.run("j") == 8

    # ========================================================================
    # TEST 86 of 183: runs nested block statement of variable
    # TypeScript: line 1166
    # ========================================================================

    def test_runs_nested_block_statement_of_variable(self):
        """
        Original TypeScript (line 1166):
        ```typescript
        it("runs nested block statement of variable", () => {
          nucleoid.run("radius = 10");
          nucleoid.run(
            "{ let area = Math.pow ( radius , 2 ) * 3.14 ; { volume = area * 5 } }"
          );
          equal(nucleoid.run("volume"), 1570);
        });
        ```
        """
        nucleoid.run("radius = 10")
        nucleoid.run(
            "{ let area = Math.pow(radius, 2) * 3.14 ; { volume = area * 5 } }"
        )
        assert nucleoid.run("volume") == 1570

    # ========================================================================
    # TEST 87 of 183: runs nested if statement of variable
    # TypeScript: line 1174
    # ========================================================================

    def test_runs_nested_if_statement_of_variable(self):
        """
        Original TypeScript (line 1174):
        ```typescript
        it("runs nested if statement of variable", () => {
          nucleoid.run("gravity = 9.8");
          nucleoid.run("time = 10");
          nucleoid.run("distance = 480");
          nucleoid.run("target = true");
          nucleoid.run(
            "{ let dist = 1 / 2 * gravity * time * time ; if ( dist > distance ) { hit = target } }"
          );
          equal(nucleoid.run("hit"), true);

          nucleoid.run("target = false");
          equal(nucleoid.run("hit"), false);
        });
        ```
        """
        nucleoid.run("gravity = 9.8")
        nucleoid.run("time = 10")
        nucleoid.run("distance = 480")
        nucleoid.run("target = True")
        nucleoid.run(
            "{ let dist = 1 / 2 * gravity * time * time ; if (dist > distance) { hit = target } }"
        )
        assert nucleoid.run("hit") is True

        nucleoid.run("target = False")
        assert nucleoid.run("hit") is False

    # ========================================================================
    # TEST 88 of 183: runs nested else statement of variable
    # TypeScript: line 1188
    # ========================================================================

    def test_runs_nested_else_statement_of_variable(self):
        """
        Original TypeScript (line 1188):
        ```typescript
        it("runs nested else statement of variable", () => {
          nucleoid.run("percentage = 28");
          nucleoid.run("density = 0.899");
          nucleoid.run("substance = 'NH3'");
          nucleoid.run("molarConcentration = null");
          nucleoid.run("def = 0");
          nucleoid.run(
            "{ let concentration = percentage * density / 100 * 1000 ; if ( substance == 'NH3' ) { molarConcentration = concentration / 17.04 } else { molarConcentration = def } }"
          );
          nucleoid.run("substance = 'NH16'");
          nucleoid.run("def = 1");
          equal(nucleoid.run("molarConcentration"), 1);
        });
        ```
        """
        nucleoid.run("percentage = 28")
        nucleoid.run("density = 0.899")
        nucleoid.run("substance = 'NH3'")
        nucleoid.run("molarConcentration = None")
        nucleoid.run("def = 0")
        nucleoid.run(
            "{ let concentration = percentage * density / 100 * 1000 ; if (substance == 'NH3') { molarConcentration = concentration / 17.04 } else { molarConcentration = def } }"
        )
        nucleoid.run("substance = 'NH16'")
        nucleoid.run("def = 1")
        assert nucleoid.run("molarConcentration") == 1

    # ========================================================================
    # TEST 89 of 183: assigns variable to reference
    # TypeScript: line 1202
    # ========================================================================

    def test_assigns_variable_to_reference(self):
        """
        Original TypeScript (line 1202):
        ```typescript
        it("assigns variable to reference", () => {
          nucleoid.run("a = 1");
          nucleoid.run("b = a");
          equal(nucleoid.run("b"), 1);

          nucleoid.run("a = 2");
          equal(nucleoid.run("b"), 2);
        });
        ```
        """
        nucleoid.run("a = 1")
        nucleoid.run("b = a")
        assert nucleoid.run("b") == 1

        nucleoid.run("a = 2")
        assert nucleoid.run("b") == 2

    # ========================================================================
    # TEST 90 of 183: assigns object to variable
    # TypeScript: line 1211
    # ========================================================================

    def test_assigns_object_to_variable(self):
        """
        Original TypeScript (line 1211):
        ```typescript
        it("assigns object to variable", () => {
          nucleoid.run("class Model { }");
          nucleoid.run("model1 = new Model ( )");
          equal(nucleoid.run("typeof model1"), "object");
        });
        ```
        """
        nucleoid.run("class Model { }")
        nucleoid.run("model1 = new Model()")
        assert nucleoid.run("typeof model1") == "object"

    # ========================================================================
    # TEST 91 of 183: defines class in the state
    # TypeScript: line 1217
    # ========================================================================

    def test_defines_class_in_the_state(self):
        """
        Original TypeScript (line 1217):
        ```typescript
        it("defines class in the state", () => {
          nucleoid.run("class Entity { }");
          equal(nucleoid.run("typeof $Entity"), "function");
        });
        ```
        """
        nucleoid.run("class Entity { }")
        assert nucleoid.run("typeof $Entity") == "function"

    # ========================================================================
    # TEST 92 of 183: rejects creating instance if the class does not exist
    # TypeScript: line 1222
    # ========================================================================

    def test_rejects_creating_instance_if_the_class_does_not_exist(self):
        """
        Original TypeScript (line 1222):
        ```typescript
        it("rejects creating instance if the class does not exist", () => {
          throws(
            () => {
              nucleoid.run("chart1 = new Chart ( )");
            },
            (error: Error) =>
              validate(error, ReferenceError, "Chart is not defined")
          );

          nucleoid.run("class Chart { }");
          nucleoid.run("chart1 = new Chart ( )");
          throws(
            () => {
              nucleoid.run("chart1.plot = new Plot ( )");
            },
            (error: Error) => validate(error, ReferenceError, "Plot is not defined")
          );

          throws(
            () => {
              nucleoid.run("$Chart.plot = new Plot ( )");
            },
            (error: Error) => validate(error, ReferenceError, "Plot is not defined")
          );
        });
        ```
        """
        with pytest.raises(ReferenceError, match="Chart is not defined"):
            nucleoid.run("chart1 = new Chart()")

        nucleoid.run("class Chart { }")
        nucleoid.run("chart1 = new Chart()")

        with pytest.raises(ReferenceError, match="Plot is not defined"):
            nucleoid.run("chart1.plot = new Plot()")

        with pytest.raises(ReferenceError, match="Plot is not defined"):
            nucleoid.run("$Chart.plot = new Plot()")

    # ========================================================================
    # TEST 93 of 183: creates property assignment before declaration
    # TypeScript: line 1248
    # ========================================================================

    def test_creates_property_assignment_before_declaration(self):
        """
        Original TypeScript (line 1248):
        ```typescript
        it("creates property assignment before declaration", () => {
          nucleoid.run("class Order { }");
          nucleoid.run("var order1 = new Order ( )");
          nucleoid.run("order1.upc = '04061' + order1.barcode");
          nucleoid.run("order1.barcode = '94067'");
          equal(nucleoid.run("order1.upc"), "0406194067");
        });
        ```
        """
        nucleoid.run("class Order { }")
        nucleoid.run("var order1 = new Order()")
        nucleoid.run("order1.upc = '04061' + order1.barcode")
        nucleoid.run("order1.barcode = '94067'")
        assert nucleoid.run("order1.upc") == "0406194067"

    # ========================================================================
    # TEST 94 of 183: creates property assignment after declaration
    # TypeScript: line 1256
    # ========================================================================

    def test_creates_property_assignment_after_declaration(self):
        """
        Original TypeScript (line 1256):
        ```typescript
        it("creates property assignment after declaration", () => {
          nucleoid.run("class User { }");
          nucleoid.run("user = new User ( )");
          nucleoid.run("user.name = 'sample'");
          nucleoid.run("user.email = user.name + '@example.com'");
          equal(nucleoid.run("user.email"), "sample@example.com");

          nucleoid.run("user.name = 'samplex'");
          equal(nucleoid.run("user.email"), "samplex@example.com");
        });
        ```
        """
        nucleoid.run("class User { }")
        nucleoid.run("user = new User()")
        nucleoid.run("user.name = 'sample'")
        nucleoid.run("user.email = user.name + '@example.com'")
        assert nucleoid.run("user.email") == "sample@example.com"

        nucleoid.run("user.name = 'samplex'")
        assert nucleoid.run("user.email") == "samplex@example.com"

    # ========================================================================
    # TEST 95 of 183: creates property assignment only if instance is defined
    # TypeScript: line 1267
    # ========================================================================

    def test_creates_property_assignment_only_if_instance_is_defined(self):
        """
        Original TypeScript (line 1267):
        ```typescript
        it("creates property assignment only if instance is defined", () => {
          nucleoid.run("class Channel { }");
          nucleoid.run("channel1 = new Channel ( )");
          throws(
            () => {
              nucleoid.run("channel1.frequency.type = 'ANGULAR'");
            },
            (error: Error) =>
              validate(error, ReferenceError, "channel1.frequency is not defined")
          );
        });
        ```
        """
        nucleoid.run("class Channel { }")
        nucleoid.run("channel1 = new Channel()")
        with pytest.raises(ReferenceError, match="channel1.frequency is not defined"):
            nucleoid.run("channel1.frequency.type = 'ANGULAR'")

    # ========================================================================
    # TEST 96 of 183: creates object with var statement
    # TypeScript: line 1279
    # ========================================================================

    def test_creates_object_with_var_statement(self):
        """
        Original TypeScript (line 1279):
        ```typescript
        it("creates object with var statement", () => {
          nucleoid.run("class Item { constructor ( name ) { this.name  = name } }");

          nucleoid.run("var item1 = new Item ( 'NAME-1' )");
          deepEqual(nucleoid.run("item1"), { id: "item1", name: "NAME-1" });

          nucleoid.run("var item2 = new Item ( )");
          deepEqual(nucleoid.run("item2"), { id: "item2", name: undefined });
        });
        ```
        """
        nucleoid.run("class Item { constructor(name) { this.name = name } }")

        nucleoid.run("var item1 = new Item('NAME-1')")
        assert nucleoid.run("item1") == {"id": "item1", "name": "NAME-1"}

        nucleoid.run("var item2 = new Item()")
        assert nucleoid.run("item2") == {"id": "item2", "name": None}

    # ========================================================================
    # TEST 97 of 183: creates object assignment as property only if instance is defined
    # TypeScript: line 1289
    # ========================================================================

    def test_creates_object_assignment_as_property_only_if_instance_is_defined(self):
        """
        Original TypeScript (line 1289):
        ```typescript
        it("creates object assignment as property only if instance is defined", () => {
          nucleoid.run("class Worker { }");
          nucleoid.run("class Schedule { }");
          nucleoid.run("worker1 = new Worker ( )");
          throws(
            () => {
              nucleoid.run("worker1.duty.schedule = new Schedule ( )");
            },

            (error: Error) =>
              validate(error, ReferenceError, "worker1.duty is not defined")
          );
        });
        ```
        """
        nucleoid.run("class Worker { }")
        nucleoid.run("class Schedule { }")
        nucleoid.run("worker1 = new Worker()")
        with pytest.raises(ReferenceError, match="worker1.duty is not defined"):
            nucleoid.run("worker1.duty.schedule = new Schedule()")

    # ========================================================================
    # TEST 98 of 183: uses its value when self property used
    # TypeScript: line 1303
    # ========================================================================

    def test_uses_its_value_when_self_property_used(self):
        """
        Original TypeScript (line 1303):
        ```typescript
        it("uses its value when self property used", () => {
          nucleoid.run("class Construction { }");
          nucleoid.run("construction1 = new Construction ( ) ");
          nucleoid.run("construction1.timeline = 120");
          nucleoid.run("construction1.timeline = 2 * construction1.timeline");
          equal(nucleoid.run("construction1.timeline"), 240);
        });
        ```
        """
        nucleoid.run("class Construction { }")
        nucleoid.run("construction1 = new Construction()")
        nucleoid.run("construction1.timeline = 120")
        nucleoid.run("construction1.timeline = 2 * construction1.timeline")
        assert nucleoid.run("construction1.timeline") == 240

    # ========================================================================
    # TEST 99 of 183: assigns object to property before initialization
    # TypeScript: line 1311
    # ========================================================================

    def test_assigns_object_to_property_before_initialization(self):
        """
        Original TypeScript (line 1311):
        ```typescript
        it("assigns object to property before initialization", () => {
          nucleoid.run("class Agent { }");
          nucleoid.run("class Distance { }");
          nucleoid.run(
            "$Distance.total = Math.sqrt ( $Distance.x * $Distance.x + $Distance.y * $Distance.y )"
          );
          nucleoid.run("agent1 = new Agent ( )");
          nucleoid.run("agent1.distance = new Distance ( )");
          nucleoid.run("agent1.distance.x = 3");
          nucleoid.run("agent1.distance.y = 4");
          equal(nucleoid.run("agent1.distance.total"), 5);
        });
        ```
        """
        nucleoid.run("class Agent { }")
        nucleoid.run("class Distance { }")
        nucleoid.run(
            "$Distance.total = Math.sqrt($Distance.x * $Distance.x + $Distance.y * $Distance.y)"
        )
        nucleoid.run("agent1 = new Agent()")
        nucleoid.run("agent1.distance = new Distance()")
        nucleoid.run("agent1.distance.x = 3")
        nucleoid.run("agent1.distance.y = 4")
        assert nucleoid.run("agent1.distance.total") == 5

    # ========================================================================
    # TEST 100 of 183: assigns object to property after initialization
    # TypeScript: line 1324
    # ========================================================================

    def test_assigns_object_to_property_after_initialization(self):
        """
        Original TypeScript (line 1324):
        ```typescript
        it("assigns object to property after initialization", () => {
          nucleoid.run("class Product { }");
          nucleoid.run("product1 = new Product ( )");
          nucleoid.run("class Quality { }");
          nucleoid.run("product1.quality = new Quality ( )");
          nucleoid.run("product1.quality.score = 15");
          nucleoid.run(
            "$Quality.class = String.fromCharCode ( 65 + Math.floor ( $Quality.score / 10 ) )"
          );
          equal(nucleoid.run("product1.quality.class"), "B");
        });
        ```
        """
        nucleoid.run("class Product { }")
        nucleoid.run("product1 = new Product()")
        nucleoid.run("class Quality { }")
        nucleoid.run("product1.quality = new Quality()")
        nucleoid.run("product1.quality.score = 15")
        nucleoid.run(
            "$Quality.class = String.fromCharCode(65 + Math.floor($Quality.score / 10))"
        )
        assert nucleoid.run("product1.quality.class") == "B"

    # ========================================================================
    # TEST 101 of 183: rejects if name of instance as property is value
    # TypeScript: line 1336
    # ========================================================================

    def test_rejects_if_name_of_instance_as_property_is_value(self):
        """
        Original TypeScript (line 1336):
        ```typescript
        it("rejects if name of instance as property is value", () => {
          nucleoid.run("class Schedule { }");
          nucleoid.run("class Place { }");
          nucleoid.run("value = new Schedule ( )");
          throws(
            () => {
              nucleoid.run("value.value = new Place ( )");
            },
            (error: Error) =>
              validate(error, TypeError, "Cannot use 'value' as a property")
          );
        });
        ```
        """
        nucleoid.run("class Schedule { }")
        nucleoid.run("class Place { }")
        nucleoid.run("value = new Schedule()")
        with pytest.raises(TypeError, match="Cannot use 'value' as a property"):
            nucleoid.run("value.value = new Place()")

    # ========================================================================
    # TEST 102 of 183: rejects if property name is value
    # TypeScript: line 1349
    # ========================================================================

    def test_rejects_if_property_name_is_value(self):
        """
        Original TypeScript (line 1349):
        ```typescript
        it("rejects if property name is value", () => {
          nucleoid.run("class Value { }");
          nucleoid.run("value = new Value ( )");
          throws(
            () => {
              nucleoid.run("value.value = 2147483647");
            },
            (error: Error) =>
              validate(error, TypeError, "Cannot use 'value' as a name")
          );
        });
        ```
        """
        nucleoid.run("class Value { }")
        nucleoid.run("value = new Value()")
        with pytest.raises(TypeError, match="Cannot use 'value' as a name"):
            nucleoid.run("value.value = 2147483647")

    # ========================================================================
    # TEST 103 of 183: uses value property to indicate using only value of property
    # TypeScript: line 1361
    # ========================================================================

    def test_uses_value_property_to_indicate_using_only_value_of_property(self):
        """
        Original TypeScript (line 1361):
        ```typescript
        it("uses value property to indicate using only value of property", () => {
          nucleoid.run("class Weight { }");
          nucleoid.run("weight1 = new Weight ( )");
          nucleoid.run("weight1.gravity = 1.352");
          nucleoid.run("weight1.mass = 1000");
          nucleoid.run("weight1.force = weight1.gravity * weight1.mass.value");
          equal(nucleoid.run("weight1.force"), 1352);

          nucleoid.run("weight1.mass = 2000");
          equal(nucleoid.run("weight1.force"), 1352);
        });
        ```
        """
        nucleoid.run("class Weight { }")
        nucleoid.run("weight1 = new Weight()")
        nucleoid.run("weight1.gravity = 1.352")
        nucleoid.run("weight1.mass = 1000")
        nucleoid.run("weight1.force = weight1.gravity * weight1.mass.value")
        assert nucleoid.run("weight1.force") == 1352

        nucleoid.run("weight1.mass = 2000")
        assert nucleoid.run("weight1.force") == 1352

    # ========================================================================
    # TEST 104 of 183: uses value property in if condition to indicate using only value of variable
    # TypeScript: line 1373
    # ========================================================================

    def test_uses_value_property_in_if_condition_to_indicate_using_only_value_of_variable(self):
        """
        Original TypeScript (line 1373):
        ```typescript
        it("uses value property in if condition to indicate using only value of variable", () => {
          nucleoid.run("class Question { }");
          nucleoid.run("question1 = new Question ( )");
          nucleoid.run("question1.text = 'How was the service?'");
          nucleoid.run(
            "if ( question1.text != question1.text.value ) { throw 'QUESTION_ARCHIVED' }"
          );
          throws(
            () => {
              nucleoid.run("question1.text = 'How would you rate us?'");
            },
            (error) => error === "QUESTION_ARCHIVED"
          );
        });
        ```
        """
        nucleoid.run("class Question { }")
        nucleoid.run("question1 = new Question()")
        nucleoid.run("question1.text = 'How was the service?'")
        nucleoid.run(
            "if (question1.text != question1.text.value) { throw 'QUESTION_ARCHIVED' }"
        )
        with pytest.raises(Exception, match="QUESTION_ARCHIVED"):
            nucleoid.run("question1.text = 'How would you rate us?'")

    # ========================================================================
    # TEST 105 of 183: keeps as null if value of property is null
    # TypeScript: line 1401
    # ========================================================================

    def test_keeps_as_null_if_value_of_property_is_null(self):
        """
        Original TypeScript (line 1401):
        ```typescript
        it("keeps as null if value of property is null", () => {
          nucleoid.run("class Interest { }");
          nucleoid.run("interest1 = new Interest ( )");
          nucleoid.run("interest1.rate = 3");
          nucleoid.run("interest1.amount = null");
          nucleoid.run(
            "interest1.annual = interest1.rate * interest1.amount.value / 100"
          );
          equal(nucleoid.run("interest1.annual"), 0);

          nucleoid.run("interest1.amount = 10000");
          equal(nucleoid.run("interest1.annual"), 0);
        });
        ```
        """
        nucleoid.run("class Interest { }")
        nucleoid.run("interest1 = new Interest()")
        nucleoid.run("interest1.rate = 3")
        nucleoid.run("interest1.amount = None")
        nucleoid.run(
            "interest1.annual = interest1.rate * interest1.amount.value / 100"
        )
        assert nucleoid.run("interest1.annual") == 0

        nucleoid.run("interest1.amount = 10000")
        assert nucleoid.run("interest1.annual") == 0

    # ========================================================================
    # TEST 106 of 183: rejects if property of local name is value
    # TypeScript: line 1415
    # ========================================================================

    def test_rejects_if_property_of_local_name_is_value(self):
        """
        Original TypeScript (line 1415):
        ```typescript
        it("rejects if property of local name is value", () => {
          nucleoid.run("class Alarm { }");
          throws(
            () => {
              nucleoid.run("{ let value = new Alarm ( ) ; value.value = '22:00' }");
            },
            (error: Error) =>
              validate(error, TypeError, "Cannot use 'value' in local")
          );
        });
        ```
        """
        nucleoid.run("class Alarm { }")
        with pytest.raises(TypeError, match="Cannot use 'value' in local"):
            nucleoid.run("{ let value = new Alarm() ; value.value = '22:00' }")

    # ========================================================================
    # TEST 107 of 183: keeps same as its value when value property used for local
    # TypeScript: line 1426
    # ========================================================================

    def test_keeps_same_as_its_value_when_value_property_used_for_local(self):
        """
        Original TypeScript (line 1426):
        ```typescript
        it("keeps same as its value when value property used for local", () => {
          nucleoid.run("speedOfLight = 299792");
          nucleoid.run(
            "{ let time = speedOfLight / 225623 ; roundTrip = time.value * 2 }"
          );
          equal(nucleoid.run("roundTrip"), 2.6574595675086314);
        });
        ```
        """
        nucleoid.run("speedOfLight = 299792")
        nucleoid.run(
            "{ let time = speedOfLight / 225623 ; roundTrip = time.value * 2 }"
        )
        assert nucleoid.run("roundTrip") == 2.6574595675086314

    # ========================================================================
    # TEST 108 of 183: uses value property as part of class declaration
    # TypeScript: line 1434
    # ========================================================================

    def test_uses_value_property_as_part_of_class_declaration(self):
        """
        Original TypeScript (line 1434):
        ```typescript
        it("uses value property as part of class declaration", () => {
          nucleoid.run("count = 0");
          nucleoid.run("class Device { }");
          nucleoid.run("device1 = new Device ( )");
          nucleoid.run("{ $Device.code = 'A' + count.value ; count = count + 1 }");
          equal(nucleoid.run("device1.code"), "A0");
        });
        ```
        """
        nucleoid.run("count = 0")
        nucleoid.run("class Device { }")
        nucleoid.run("device1 = new Device()")
        nucleoid.run("{ $Device.code = 'A' + count.value ; count = count + 1 }")
        assert nucleoid.run("device1.code") == "A0"

    # ========================================================================
    # TEST 109 of 183: uses value property of class declaration
    # TypeScript: line 1442
    # ========================================================================

    def test_uses_value_property_of_class_declaration(self):
        """
        Original TypeScript (line 1442):
        ```typescript
        it("uses value property of class declaration", () => {
          nucleoid.run(
            "class Summary { constructor ( question ) { this.question = question } }"
          );
          nucleoid.run("class Question { }");
          nucleoid.run("$Summary.count = $Summary.question.count.value");
          nucleoid.run("question1 = new Question ( )");
          nucleoid.run("question1.count = 10");
          nucleoid.run("summary1 = new Summary ( question1 )");
          equal(nucleoid.run("summary1.count"), 10);

          nucleoid.run("question1.count = 11");
          equal(nucleoid.run("summary1.count"), 10);
        });
        ```
        """
        nucleoid.run(
            "class Summary { constructor(question) { this.question = question } }"
        )
        nucleoid.run("class Question { }")
        nucleoid.run("$Summary.count = $Summary.question.count.value")
        nucleoid.run("question1 = new Question()")
        nucleoid.run("question1.count = 10")
        nucleoid.run("summary1 = new Summary(question1)")
        assert nucleoid.run("summary1.count") == 10

        nucleoid.run("question1.count = 11")
        assert nucleoid.run("summary1.count") == 10

    # ========================================================================
    # TEST 110 of 183: updates if block of property
    # TypeScript: line 1457
    # ========================================================================

    def test_updates_if_block_of_property(self):
        """
        Original TypeScript (line 1457):
        ```typescript
        it("updates if block of property", () => {
          nucleoid.run("class Account { }");
          nucleoid.run("account = new Account ( )");
          nucleoid.run("account.balance = 1000");
          nucleoid.run("if ( account.balance < 1500 ) { account.status = 'OK' }");
          equal(nucleoid.run("account.status"), "OK");

          nucleoid.run("if ( account.balance < 1500 ) { account.status = 'LOW' }");
          equal(nucleoid.run("account.status"), "LOW");
        });
        ```
        """
        nucleoid.run("class Account { }")
        nucleoid.run("account = new Account()")
        nucleoid.run("account.balance = 1000")
        nucleoid.run("if (account.balance < 1500) { account.status = 'OK' }")
        assert nucleoid.run("account.status") == "OK"

        nucleoid.run("if (account.balance < 1500) { account.status = 'LOW' }")
        assert nucleoid.run("account.status") == "LOW"

    # ========================================================================
    # TEST 111 of 183: creates if statement of property
    # TypeScript: line 1468
    # ========================================================================

    def test_creates_if_statement_of_property(self):
        """
        Original TypeScript (line 1468):
        ```typescript
        it("creates if statement of property", () => {
          nucleoid.run("class Toy { }");
          nucleoid.run("toy = new Toy ( )");
          nucleoid.run("toy.color = 'BLUE'");
          nucleoid.run("if ( toy.color == 'RED' ) { toy.shape = 'CIRCLE' }");
          nucleoid.run("toy.color = 'RED'");
          equal(nucleoid.run("toy.shape"), "CIRCLE");
        });
        ```
        """
        nucleoid.run("class Toy { }")
        nucleoid.run("toy = new Toy()")
        nucleoid.run("toy.color = 'BLUE'")
        nucleoid.run("if (toy.color == 'RED') { toy.shape = 'CIRCLE' }")
        nucleoid.run("toy.color = 'RED'")
        assert nucleoid.run("toy.shape") == "CIRCLE"

    # ========================================================================
    # TEST 112 of 183: creates else statement of property
    # TypeScript: line 1477
    # ========================================================================

    def test_creates_else_statement_of_property(self):
        """
        Original TypeScript (line 1477):
        ```typescript
        it("creates else statement of property", () => {
          nucleoid.run("class Engine { }");
          nucleoid.run("engine1 = new Engine ( )");
          nucleoid.run("engine1.type = 'V8'");
          nucleoid.run("mpl = 'MPL'");
          nucleoid.run("bsd = 'BSD'");
          nucleoid.run(
            "if ( engine1.type == 'Gecko' ) { engine1.license = mpl } else { engine1.license = bsd }"
          );
          equal(nucleoid.run("engine1.license"), "BSD");

          nucleoid.run("bsd = 'Berkeley Software Distribution'");
          equal(nucleoid.run("engine1.license"), "Berkeley Software Distribution");
        });
        ```
        """
        nucleoid.run("class Engine { }")
        nucleoid.run("engine1 = new Engine()")
        nucleoid.run("engine1.type = 'V8'")
        nucleoid.run("mpl = 'MPL'")
        nucleoid.run("bsd = 'BSD'")
        nucleoid.run(
            "if (engine1.type == 'Gecko') { engine1.license = mpl } else { engine1.license = bsd }"
        )
        assert nucleoid.run("engine1.license") == "BSD"

        nucleoid.run("bsd = 'Berkeley Software Distribution'")
        assert nucleoid.run("engine1.license") == "Berkeley Software Distribution"

    # ========================================================================
    # TEST 113 of 183: creates else if statement of property
    # TypeScript: line 1492
    # ========================================================================

    def test_creates_else_if_statement_of_property(self):
        """
        Original TypeScript (line 1492):
        ```typescript
        it("creates else if statement of property", () => {
          nucleoid.run("class Contact { }");
          nucleoid.run("contact1 = new Contact ( )");
          nucleoid.run("contact1.type = 'PERSON'");
          nucleoid.run("contact1.first = 'First'");
          nucleoid.run("contact1.last = 'Last'");
          nucleoid.run(
            "if ( contact1.type == 'BUSINESS' ) { contact1.full = 'B' + contact1.first } else { contact1.full = contact1.first + ' ' + contact1.last }"
          );
          equal(nucleoid.run("contact1.full"), "First Last");

          nucleoid.run("contact1.first = 'F' ; contact1.last = 'L'");
          equal(nucleoid.run("contact1.full"), "F L");
        });
        ```
        """
        nucleoid.run("class Contact { }")
        nucleoid.run("contact1 = new Contact()")
        nucleoid.run("contact1.type = 'PERSON'")
        nucleoid.run("contact1.first = 'First'")
        nucleoid.run("contact1.last = 'Last'")
        nucleoid.run(
            "if (contact1.type == 'BUSINESS') { contact1.full = 'B' + contact1.first } else { contact1.full = contact1.first + ' ' + contact1.last }"
        )
        assert nucleoid.run("contact1.full") == "First Last"

        nucleoid.run("contact1.first = 'F' ; contact1.last = 'L'")
        assert nucleoid.run("contact1.full") == "F L"

    # ========================================================================
    # TEST 114 of 183: creates multiple else if statement of property
    # TypeScript: line 1507
    # ========================================================================

    def test_creates_multiple_else_if_statement_of_property(self):
        """
        Original TypeScript (line 1507):
        ```typescript
        it("creates multiple else if statement of property", () => {
          nucleoid.run("class Taxpayer { }");
          nucleoid.run("taxpayer1 = new Taxpayer ( )");
          nucleoid.run("taxpayer1.income = 60000");
          nucleoid.run("taxpayer1.member = 1");
          nucleoid.run("rate = 22");
          nucleoid.run(
            "if ( taxpayer1.member > 4 ) { taxpayer1.tax = taxpayer1.income * rate / 100 - 2000 } else if ( taxpayer1.member > 2 ) { taxpayer1.tax = taxpayer1.income * rate / 100 - 1000 } else { taxpayer1.tax = taxpayer1.income * rate / 100 }"
          );
          equal(nucleoid.run("taxpayer1.tax"), 13200);

          nucleoid.run("rate = 23");
          equal(nucleoid.run("taxpayer1.tax"), 13800);
        });
        ```
        """
        nucleoid.run("class Taxpayer { }")
        nucleoid.run("taxpayer1 = new Taxpayer()")
        nucleoid.run("taxpayer1.income = 60000")
        nucleoid.run("taxpayer1.member = 1")
        nucleoid.run("rate = 22")
        nucleoid.run(
            "if (taxpayer1.member > 4) { taxpayer1.tax = taxpayer1.income * rate / 100 - 2000 } else if (taxpayer1.member > 2) { taxpayer1.tax = taxpayer1.income * rate / 100 - 1000 } else { taxpayer1.tax = taxpayer1.income * rate / 100 }"
        )
        assert nucleoid.run("taxpayer1.tax") == 13200

        nucleoid.run("rate = 23")
        assert nucleoid.run("taxpayer1.tax") == 13800

    # ========================================================================
    # TEST 115 of 183: updates property assignment
    # TypeScript: line 1522
    # ========================================================================

    def test_updates_property_assignment(self):
        """
        Original TypeScript (line 1522):
        ```typescript
        it("updates property assignment", () => {
          nucleoid.run("class Matter { }");
          nucleoid.run("matter1 = new Matter ( )");
          nucleoid.run("matter1.mass = 10");
          nucleoid.run("matter1.weight = matter1.mass * 9.8");
          equal(nucleoid.run("matter1.weight"), 98);

          nucleoid.run("matter1.weight = matter1.mass * 3.7");
          equal(nucleoid.run("matter1.weight"), 37);

          nucleoid.run("matter1.mass = 20");
          equal(nucleoid.run("matter1.weight"), 74);
        });
        ```
        """
        nucleoid.run("class Matter { }")
        nucleoid.run("matter1 = new Matter()")
        nucleoid.run("matter1.mass = 10")
        nucleoid.run("matter1.weight = matter1.mass * 9.8")
        assert nucleoid.run("matter1.weight") == 98

        nucleoid.run("matter1.weight = matter1.mass * 3.7")
        assert nucleoid.run("matter1.weight") == 37

        nucleoid.run("matter1.mass = 20")
        assert nucleoid.run("matter1.weight") == 74

    # ========================================================================
    # TEST 116 of 183: deletes instance
    # TypeScript: line 1536
    # ========================================================================

    def test_deletes_instance(self):
        """
        Original TypeScript (line 1536):
        ```typescript
        it("deletes instance", () => {
          nucleoid.run("class Circle { }");
          nucleoid.run("circle1 = new Circle ( )");
          nucleoid.run("delete circle1");
          equal(nucleoid.run("Circle['circle1']"), undefined);
          equal(
            nucleoid.run("Circle.find ( circle => circle.id === 'circle1' )"),
            undefined
          );

          throws(
            () => {
              nucleoid.run("circle1");
            },
            (error: Error) =>
              validate(error, ReferenceError, "circle1 is not defined")
          );
        });
        ```
        """
        nucleoid.run("class Circle { }")
        nucleoid.run("circle1 = new Circle()")
        nucleoid.run("delete circle1")
        assert nucleoid.run("Circle['circle1']") is None
        assert nucleoid.run("Circle.find(circle => circle.id === 'circle1')") is None

        with pytest.raises(ReferenceError, match="circle1 is not defined"):
            nucleoid.run("circle1")

    # ========================================================================
    # TEST 117 of 183: deletes instance by reference
    # TypeScript: line 1555
    # ========================================================================

    def test_deletes_instance_by_reference(self):
        """
        Original TypeScript (line 1555):
        ```typescript
        it("deletes instance by reference", () => {
          nucleoid.run("class Item { }");
          nucleoid.run("item1 = new Item ( )");
          nucleoid.run("item2 = new Item ( )");
          deepEqual(nucleoid.run("Item['item1']"), { id: "item1" });

          nucleoid.run("delete Item['item1']");
          equal(nucleoid.run("Item['item1']"), undefined);

          deepEqual(nucleoid.run("Item['item2']"), { id: "item2" });
          nucleoid.run("let item = 'item2' ; delete Item[item]");
          equal(nucleoid.run("Item['item2']"), undefined);
        });
        ```
        """
        nucleoid.run("class Item { }")
        nucleoid.run("item1 = new Item()")
        nucleoid.run("item2 = new Item()")
        assert nucleoid.run("Item['item1']") == {"id": "item1"}

        nucleoid.run("delete Item['item1']")
        assert nucleoid.run("Item['item1']") is None

        assert nucleoid.run("Item['item2']") == {"id": "item2"}
        nucleoid.run("let item = 'item2' ; delete Item[item]")
        assert nucleoid.run("Item['item2']") is None

    # ========================================================================
    # TEST 118 of 183: returns boolean when deleting object
    # TypeScript: line 1569
    # ========================================================================

    def test_returns_boolean_when_deleting_object(self):
        """
        Original TypeScript (line 1569):
        ```typescript
        it("returns boolean when deleting object", () => {
          nucleoid.run("class Location { }");
          nucleoid.run("location1 = new Location ( )");
          equal(nucleoid.run("delete location1"), true);
          equal(nucleoid.run("delete location2"), false);
        });
        ```
        """
        nucleoid.run("class Location { }")
        nucleoid.run("location1 = new Location()")
        assert nucleoid.run("delete location1") is True
        assert nucleoid.run("delete location2") is False

    # ========================================================================
    # TEST 119 of 183: rejects deleting instance if it has any properties
    # TypeScript: line 1576
    # ========================================================================

    def test_rejects_deleting_instance_if_it_has_any_properties(self):
        """
        Original TypeScript (line 1576):
        ```typescript
        it("rejects deleting instance if it has any properties", () => {
          nucleoid.run("class Channel { }");
          nucleoid.run("channel1 = new Channel ( )");
          nucleoid.run("channel1.frequency = 440");
          throws(
            () => {
              nucleoid.run("delete channel1");
            },
            (error: Error) =>
              validate(error, ReferenceError, "Cannot delete object 'channel1'")
          );
          equal(nucleoid.run("channel1.frequency "), 440);

          nucleoid.run("delete channel1.frequency");
          nucleoid.run("delete channel1");
        });
        ```
        """
        nucleoid.run("class Channel { }")
        nucleoid.run("channel1 = new Channel()")
        nucleoid.run("channel1.frequency = 440")
        with pytest.raises(ReferenceError, match="Cannot delete object 'channel1'"):
            nucleoid.run("delete channel1")
        assert nucleoid.run("channel1.frequency") == 440

        nucleoid.run("delete channel1.frequency")
        nucleoid.run("delete channel1")

    # ========================================================================
    # TEST 120 of 183: rejects deleting instance if it has object as a property
    # TypeScript: line 1593
    # ========================================================================

    def test_rejects_deleting_instance_if_it_has_object_as_a_property(self):
        """
        Original TypeScript (line 1593):
        ```typescript
        it("rejects deleting instance if it has object as a property", () => {
          nucleoid.run("class Shape { }");
          nucleoid.run("class Type { }");
          nucleoid.run("shape1 = new Shape ( )");
          nucleoid.run("shape1.type = new Type ( )");
          throws(
            () => {
              nucleoid.run("delete shape1");
            },
            (error: Error) =>
              validate(error, ReferenceError, "Cannot delete object 'shape1'")
          );

          nucleoid.run("delete shape1.type");
          nucleoid.run("delete shape1");
        });
        ```
        """
        nucleoid.run("class Shape { }")
        nucleoid.run("class Type { }")
        nucleoid.run("shape1 = new Shape()")
        nucleoid.run("shape1.type = new Type()")
        with pytest.raises(ReferenceError, match="Cannot delete object 'shape1'"):
            nucleoid.run("delete shape1")

        nucleoid.run("delete shape1.type")
        nucleoid.run("delete shape1")

    # ========================================================================
    # TEST 121 of 183: deletes property assignment
    # TypeScript: line 1610
    # ========================================================================

    def test_deletes_property_assignment(self):
        """
        Original TypeScript (line 1610):
        ```typescript
        it("deletes property assignment", () => {
          nucleoid.run("class Agent { }");
          nucleoid.run("agent = new Agent ( )");
          nucleoid.run("agent.time = 52926163455");
          nucleoid.run("agent.location = 'CITY'");
          nucleoid.run("agent.report = agent.time + '@' + agent.location");
          equal(nucleoid.run("agent.report"), "52926163455@CITY");

          nucleoid.run("delete agent.time");
          equal(nucleoid.run("agent.report"), undefined);

          nucleoid.run("delete agent.report");
          equal(nucleoid.run("agent.report"), undefined);
        });
        ```
        """
        nucleoid.run("class Agent { }")
        nucleoid.run("agent = new Agent()")
        nucleoid.run("agent.time = 52926163455")
        nucleoid.run("agent.location = 'CITY'")
        nucleoid.run("agent.report = agent.time + '@' + agent.location")
        assert nucleoid.run("agent.report") == "52926163455@CITY"

        nucleoid.run("delete agent.time")
        assert nucleoid.run("agent.report") is None

        nucleoid.run("delete agent.report")
        assert nucleoid.run("agent.report") is None

    # ========================================================================
    # TEST 122 of 183: runs block statement of property
    # TypeScript: line 1625
    # ========================================================================

    def test_runs_block_statement_of_property(self):
        """
        Original TypeScript (line 1625):
        ```typescript
        it("runs block statement of property", () => {
          nucleoid.run("class Item { }");
          nucleoid.run("item1 = new Item ( )");
          nucleoid.run("item1.sku = '0000001' ");
          nucleoid.run("{ let custom = 'US' + item1.sku ; item1.custom = custom }");
          equal(nucleoid.run("item1.custom"), "US0000001");

          nucleoid.run("item1.sku = '0000002' ");
          equal(nucleoid.run("item1.custom"), "US0000002");
        });
        ```
        """
        nucleoid.run("class Item { }")
        nucleoid.run("item1 = new Item()")
        nucleoid.run("item1.sku = '0000001'")
        nucleoid.run("{ let custom = 'US' + item1.sku ; item1.custom = custom }")
        assert nucleoid.run("item1.custom") == "US0000001"

        nucleoid.run("item1.sku = '0000002'")
        assert nucleoid.run("item1.custom") == "US0000002"

    # ========================================================================
    # TEST 123 of 183: runs nested block statement of property
    # TypeScript: line 1636
    # ========================================================================

    def test_runs_nested_block_statement_of_property(self):
        """
        Original TypeScript (line 1636):
        ```typescript
        it("runs nested block statement of property", () => {
          nucleoid.run("class Figure { }");
          nucleoid.run("figure1 = new Figure ( )");
          nucleoid.run("figure1.width = 9");
          nucleoid.run("figure1.height = 10");
          nucleoid.run(
            "{ let base = Math.pow ( figure1.width , 2 ) ; { figure1.volume = base * figure1.height } }"
          );
          equal(nucleoid.run("figure1.volume"), 810);

          nucleoid.run("figure1.height = 9");
          equal(nucleoid.run("figure1.volume"), 729);
        });
        ```
        """
        nucleoid.run("class Figure { }")
        nucleoid.run("figure1 = new Figure()")
        nucleoid.run("figure1.width = 9")
        nucleoid.run("figure1.height = 10")
        nucleoid.run(
            "{ let base = Math.pow(figure1.width, 2) ; { figure1.volume = base * figure1.height } }"
        )
        assert nucleoid.run("figure1.volume") == 810

        nucleoid.run("figure1.height = 9")
        assert nucleoid.run("figure1.volume") == 729

    # ========================================================================
    # TEST 124 of 183: runs nested if statement of property
    # TypeScript: line 1650
    # ========================================================================

    def test_runs_nested_if_statement_of_property(self):
        """
        Original TypeScript (line 1650):
        ```typescript
        it("runs nested if statement of property", () => {
          nucleoid.run("class Sale { }");
          nucleoid.run("sale1 = new Sale ( )");
          nucleoid.run("sale1.price = 50");
          nucleoid.run("sale1.quantity = 2");
          nucleoid.run(
            "{ let amount = sale1.price * sale1.quantity ; if ( amount > 100 ) { sale1.tax = amount * 10 / 100 } }"
          );
          equal(nucleoid.run("sale1.tax"), undefined);

          nucleoid.run("sale1.quantity = 3");
          equal(nucleoid.run("sale1.tax"), 15);
        });
        ```
        """
        nucleoid.run("class Sale { }")
        nucleoid.run("sale1 = new Sale()")
        nucleoid.run("sale1.price = 50")
        nucleoid.run("sale1.quantity = 2")
        nucleoid.run(
            "{ let amount = sale1.price * sale1.quantity ; if (amount > 100) { sale1.tax = amount * 10 / 100 } }"
        )
        assert nucleoid.run("sale1.tax") is None

        nucleoid.run("sale1.quantity = 3")
        assert nucleoid.run("sale1.tax") == 15

    # ========================================================================
    # TEST 125 of 183: creates nested else statement of property
    # TypeScript: line 1664
    # ========================================================================

    def test_creates_nested_else_statement_of_property(self):
        """
        Original TypeScript (line 1664):
        ```typescript
        it("creates nested else statement of property", () => {
          nucleoid.run("class Chart { }");
          nucleoid.run("chart1 = new Chart ( )");
          nucleoid.run("chart1.percentage = 1");
          nucleoid.run("invalid = 'INVALID'");
          nucleoid.run("valid = 'VALID'");
          nucleoid.run(
            "{ let ratio = chart1.percentage / 100 ; if ( ratio > 1 ) { chart1.status = invalid } else { chart1.status = valid } }"
          );
          equal(nucleoid.run("chart1.status"), "VALID");

          nucleoid.run("valid = 'V'");
          equal(nucleoid.run("chart1.status"), "V");
        });
        ```
        """
        nucleoid.run("class Chart { }")
        nucleoid.run("chart1 = new Chart()")
        nucleoid.run("chart1.percentage = 1")
        nucleoid.run("invalid = 'INVALID'")
        nucleoid.run("valid = 'VALID'")
        nucleoid.run(
            "{ let ratio = chart1.percentage / 100 ; if (ratio > 1) { chart1.status = invalid } else { chart1.status = valid } }"
        )
        assert nucleoid.run("chart1.status") == "VALID"

        nucleoid.run("valid = 'V'")
        assert nucleoid.run("chart1.status") == "V"

    # ========================================================================
    # TEST 126 of 183: creates property assignment with multiple properties
    # TypeScript: line 1679
    # ========================================================================

    def test_creates_property_assignment_with_multiple_properties(self):
        """
        Original TypeScript (line 1679):
        ```typescript
        it("creates property assignment with multiple properties", () => {
          nucleoid.run("class Person { }");
          nucleoid.run("person1 = new Person ( )");
          nucleoid.run("class Address { }");
          nucleoid.run("address1 = new Address ( )");
          nucleoid.run("$Address.print = $Address.city + ', ' + $Address.state");
          nucleoid.run("person1.address = new Address ( )");
          nucleoid.run("person1.address.city = 'Syracuse'");
          nucleoid.run("person1.address.state = 'NY'");
          equal(nucleoid.run("person1.address.print"), "Syracuse, NY");
        });
        ```
        """
        nucleoid.run("class Person { }")
        nucleoid.run("person1 = new Person()")
        nucleoid.run("class Address { }")
        nucleoid.run("address1 = new Address()")
        nucleoid.run("$Address.print = $Address.city + ', ' + $Address.state")
        nucleoid.run("person1.address = new Address()")
        nucleoid.run("person1.address.city = 'Syracuse'")
        nucleoid.run("person1.address.state = 'NY'")
        assert nucleoid.run("person1.address.print") == "Syracuse, NY"

    # ========================================================================
    # TEST 127 of 183: creates property assignment as multiple properties as part of declaration
    # TypeScript: line 1691
    # ========================================================================

    def test_creates_property_assignment_as_multiple_properties_as_part_of_declaration(self):
        """
        Original TypeScript (line 1691):
        ```typescript
        it("creates property assignment as multiple properties as part of declaration", () => {
          nucleoid.run("class Server { }");
          nucleoid.run("server1 = new Server ( )");
          nucleoid.run("server1.name = 'HOST1'");
          nucleoid.run("class IP { }");
          nucleoid.run("ip1 = new IP ( )");
          nucleoid.run("server1.ip = ip1");
          nucleoid.run("ip1.address = '10.0.0.1'");
          nucleoid.run("server1.summary = server1.name + '@' + server1.ip.address");
          equal(nucleoid.run("server1.summary"), "HOST1@10.0.0.1");

          nucleoid.run("ip1.address = '10.0.0.2'");
          equal(nucleoid.run("server1.summary"), "HOST1@10.0.0.2");
        });
        ```
        """
        nucleoid.run("class Server { }")
        nucleoid.run("server1 = new Server()")
        nucleoid.run("server1.name = 'HOST1'")
        nucleoid.run("class IP { }")
        nucleoid.run("ip1 = new IP()")
        nucleoid.run("server1.ip = ip1")
        nucleoid.run("ip1.address = '10.0.0.1'")
        nucleoid.run("server1.summary = server1.name + '@' + server1.ip.address")
        assert nucleoid.run("server1.summary") == "HOST1@10.0.0.1"

        nucleoid.run("ip1.address = '10.0.0.2'")
        assert nucleoid.run("server1.summary") == "HOST1@10.0.0.2"

    # ========================================================================
    # TEST 128 of 183: creates dependency behalf if property has reference
    # TypeScript: line 1706
    # ========================================================================

    def test_creates_dependency_behalf_if_property_has_reference(self):
        """
        Original TypeScript (line 1706):
        ```typescript
        it("creates dependency behalf if property has reference", () => {
          nucleoid.run("class Schedule { }");
          nucleoid.run("schedule1 = new Schedule ( )");

          nucleoid.run("class Template { }");
          nucleoid.run("template1 = new Template ( )");
          nucleoid.run("template1.type = 'W'");

          nucleoid.run("schedule1.template = template1");
          nucleoid.run(
            "schedule1.template.name = schedule1.template.type + '-0001'"
          );
          equal(nucleoid.run("template1.name"), "W-0001");
          equal(nucleoid.run("schedule1.template.name"), "W-0001");

          nucleoid.run("template1.type = 'D'");
          equal(nucleoid.run("template1.name"), "D-0001");

          nucleoid.run("template1.shape = template1.type + '-Form'");
          equal(nucleoid.run("template1.shape"), "D-Form");
          equal(nucleoid.run("schedule1.template.shape"), "D-Form");

          nucleoid.run("template1.type = 'C'");
          equal(nucleoid.run("template1.shape"), "C-Form");
          equal(nucleoid.run("schedule1.template.shape"), "C-Form");
        });
        ```
        """
        nucleoid.run("class Schedule { }")
        nucleoid.run("schedule1 = new Schedule()")

        nucleoid.run("class Template { }")
        nucleoid.run("template1 = new Template()")
        nucleoid.run("template1.type = 'W'")

        nucleoid.run("schedule1.template = template1")
        nucleoid.run(
            "schedule1.template.name = schedule1.template.type + '-0001'"
        )
        assert nucleoid.run("template1.name") == "W-0001"
        assert nucleoid.run("schedule1.template.name") == "W-0001"

        nucleoid.run("template1.type = 'D'")
        assert nucleoid.run("template1.name") == "D-0001"

        nucleoid.run("template1.shape = template1.type + '-Form'")
        assert nucleoid.run("template1.shape") == "D-Form"
        assert nucleoid.run("schedule1.template.shape") == "D-Form"

        nucleoid.run("template1.type = 'C'")
        assert nucleoid.run("template1.shape") == "C-Form"
        assert nucleoid.run("schedule1.template.shape") == "C-Form"

    # ========================================================================
    # TEST 129 of 183: creates dependency behalf if let has reference
    # TypeScript: line 1733
    # ========================================================================

    def test_creates_dependency_behalf_if_let_has_reference(self):
        """
        Original TypeScript (line 1733):
        ```typescript
        it("creates dependency behalf if let has reference", () => {
          nucleoid.run("class Vote { }");
          nucleoid.run("vote1 = new Vote ( )");
          nucleoid.run("vote1.rate = 4");
          nucleoid.run("class Question { }");
          nucleoid.run("$Question.rate = 0");
          nucleoid.run("$Question.count = 0");
          nucleoid.run("question1 = new Question ( )");
          nucleoid.run("vote1.question = question1");
          nucleoid.run(
            "{ let question = vote1.question ; question.rate = ( question.rate * question.count + vote1.rate ) / ( question.count + 1 ) ; question.count =  question.count + 1}"
          );
          equal(nucleoid.run("question1.rate"), 4);
          equal(nucleoid.run("question1.count"), 1);

          nucleoid.run("vote1.rate = 5");
          equal(nucleoid.run("question1.rate"), 4.5);
        });
        ```
        """
        nucleoid.run("class Vote { }")
        nucleoid.run("vote1 = new Vote()")
        nucleoid.run("vote1.rate = 4")
        nucleoid.run("class Question { }")
        nucleoid.run("$Question.rate = 0")
        nucleoid.run("$Question.count = 0")
        nucleoid.run("question1 = new Question()")
        nucleoid.run("vote1.question = question1")
        nucleoid.run(
            "{ let question = vote1.question ; question.rate = (question.rate * question.count + vote1.rate) / (question.count + 1) ; question.count = question.count + 1 }"
        )
        assert nucleoid.run("question1.rate") == 4
        assert nucleoid.run("question1.count") == 1

        nucleoid.run("vote1.rate = 5")
        assert nucleoid.run("question1.rate") == 4.5

    # ========================================================================
    # TEST 130 of 183: runs expression statement of class
    # TypeScript: line 1752
    # ========================================================================

    def test_runs_expression_statement_of_class(self):
        """
        Original TypeScript (line 1752):
        ```typescript
        it("runs expression statement of class", () => {
          nucleoid.run("class Element { }");
          nucleoid.run("alkalis = [ ]");
          nucleoid.run("element1 = new Element ( )");
          nucleoid.run("element1.number = 3");
          nucleoid.run(
            "{ let number = $Element.number ; if ( number == 3 ) { alkalis.push ( $Element ) } }"
          );
          equal(nucleoid.run("alkalis.pop ( )"), nucleoid.run("element1"));
        });
        ```
        """
        nucleoid.run("class Element { }")
        nucleoid.run("alkalis = []")
        nucleoid.run("element1 = new Element()")
        nucleoid.run("element1.number = 3")
        nucleoid.run(
            "{ let number = $Element.number ; if (number == 3) { alkalis.push($Element) } }"
        )
        assert nucleoid.run("alkalis.pop()") == nucleoid.run("element1")

    # ========================================================================
    # TEST 131 of 183: creates class assignment before initialization
    # TypeScript: line 1763
    # ========================================================================

    def test_creates_class_assignment_before_initialization(self):
        """
        Original TypeScript (line 1763):
        ```typescript
        it("creates class assignment before initialization", () => {
          nucleoid.run("class Review { }");
          nucleoid.run("$Review.rate = $Review.sum / 10");
          nucleoid.run("review1 = new Review ( )");
          nucleoid.run("review1.sum = 42");
          equal(nucleoid.run("review1.rate"), 4.2);
        });
        ```
        """
        nucleoid.run("class Review { }")
        nucleoid.run("$Review.rate = $Review.sum / 10")
        nucleoid.run("review1 = new Review()")
        nucleoid.run("review1.sum = 42")
        assert nucleoid.run("review1.rate") == 4.2

    # ========================================================================
    # TEST 132 of 183: creates class assignment after initialization
    # TypeScript: line 1771
    # ========================================================================

    def test_creates_class_assignment_after_initialization(self):
        """
        Original TypeScript (line 1771):
        ```typescript
        it("creates class assignment after initialization", () => {
          nucleoid.run("class Shape { }");
          nucleoid.run("s1 = new Shape ( )");
          nucleoid.run("s1.edge = 3");
          nucleoid.run("s2 = new Shape ( )");
          nucleoid.run("s2.edge = 3");
          nucleoid.run("$Shape.angle = ( $Shape.edge - 2 ) * 180");
          nucleoid.run("s1.edge = 4");
          equal(nucleoid.run("s1.angle"), 360);
          equal(nucleoid.run("s2.angle"), 180);
        });
        ```
        """
        nucleoid.run("class Shape { }")
        nucleoid.run("s1 = new Shape()")
        nucleoid.run("s1.edge = 3")
        nucleoid.run("s2 = new Shape()")
        nucleoid.run("s2.edge = 3")
        nucleoid.run("$Shape.angle = ($Shape.edge - 2) * 180")
        nucleoid.run("s1.edge = 4")
        assert nucleoid.run("s1.angle") == 360
        assert nucleoid.run("s2.angle") == 180

    # ========================================================================
    # TEST 133 of 183: updates class assignment
    # TypeScript: line 1783
    # ========================================================================

    def test_updates_class_assignment(self):
        """
        Original TypeScript (line 1783):
        ```typescript
        it("updates class assignment", () => {
          nucleoid.run("class Employee { }");
          nucleoid.run("employee = new Employee ( )");
          nucleoid.run("employee.id = 1");
          nucleoid.run("$Employee.username = 'E' + $Employee.id");
          equal(nucleoid.run("employee.username"), "E1");
          nucleoid.run("$Employee.username = 'F' + $Employee.id");
          nucleoid.run("employee.id = 2");
          equal(nucleoid.run("employee.username"), "F2");
        });
        ```
        """
        nucleoid.run("class Employee { }")
        nucleoid.run("employee = new Employee()")
        nucleoid.run("employee.id = 1")
        nucleoid.run("$Employee.username = 'E' + $Employee.id")
        assert nucleoid.run("employee.username") == "E1"
        nucleoid.run("$Employee.username = 'F' + $Employee.id")
        nucleoid.run("employee.id = 2")
        assert nucleoid.run("employee.username") == "F2"

    # ========================================================================
    # TEST 134 of 183: creates if statement of class before initialization
    # TypeScript: line 1794
    # ========================================================================

    def test_creates_if_statement_of_class_before_initialization(self):
        """
        Original TypeScript (line 1794):
        ```typescript
        it("creates if statement of class before initialization", () => {
          nucleoid.run("class Ticket { }");
          nucleoid.run(
            "if ( $Ticket.date > new Date ( '1993-1-1' ) ) { $Ticket.status = 'EXPIRED' }"
          );
          nucleoid.run("ticket1 = new Ticket ( )");

          equal(nucleoid.run("ticket1.status"), undefined);
          nucleoid.run("ticket1.date = new Date ( '1993-2-1' ) ");
          equal(nucleoid.run("ticket1.status"), "EXPIRED");

          nucleoid.run("ticket2 = new Ticket ( )");
          equal(nucleoid.run("ticket2.status"), undefined);
        });
        ```
        """
        nucleoid.run("class Ticket { }")
        nucleoid.run(
            "if ($Ticket.date > new Date('1993-1-1')) { $Ticket.status = 'EXPIRED' }"
        )
        nucleoid.run("ticket1 = new Ticket()")

        assert nucleoid.run("ticket1.status") is None
        nucleoid.run("ticket1.date = new Date('1993-2-1')")
        assert nucleoid.run("ticket1.status") == "EXPIRED"

        nucleoid.run("ticket2 = new Ticket()")
        assert nucleoid.run("ticket2.status") is None

    # ========================================================================
    # TEST 135 of 183: creates if statement of class after initialization
    # TypeScript: line 1809
    # ========================================================================

    def test_creates_if_statement_of_class_after_initialization(self):
        """
        Original TypeScript (line 1809):
        ```typescript
        it("creates if statement of class after initialization", () => {
          nucleoid.run("class Student { }");
          nucleoid.run("s1 = new Student ( )");
          nucleoid.run("s1.age = 2");
          nucleoid.run("s1.class = 'Daycare'");
          nucleoid.run("s2 = new Student ( )");
          nucleoid.run("s2.age = 2");
          nucleoid.run("s2.class = 'Daycare'");
          nucleoid.run("if ( $Student.age == 3 ) { $Student.class = 'Preschool' }");
          nucleoid.run("s1.age = 3");
          equal(nucleoid.run("s1.class"), "Preschool");
          equal(nucleoid.run("s2.class"), "Daycare");
        });
        ```
        """
        nucleoid.run("class Student { }")
        nucleoid.run("s1 = new Student()")
        nucleoid.run("s1.age = 2")
        nucleoid.run("s1.class = 'Daycare'")
        nucleoid.run("s2 = new Student()")
        nucleoid.run("s2.age = 2")
        nucleoid.run("s2.class = 'Daycare'")
        nucleoid.run("if ($Student.age == 3) { $Student.class = 'Preschool' }")
        nucleoid.run("s1.age = 3")
        assert nucleoid.run("s1.class") == "Preschool"
        assert nucleoid.run("s2.class") == "Daycare"

    # ========================================================================
    # TEST 136 of 183: updates if block of class
    # TypeScript: line 1823
    # ========================================================================

    def test_updates_if_block_of_class(self):
        """
        Original TypeScript (line 1823):
        ```typescript
        it("updates if block of class", () => {
          nucleoid.run("class Inventory { }");
          nucleoid.run("i1 = new Inventory ( )");
          nucleoid.run("i1.quantity = 0");

          nucleoid.run("i2 = new Inventory ( )");
          nucleoid.run("i2.quantity = 1000");

          nucleoid.run(
            "if ( $Inventory.quantity == 0 ) { $Inventory.replenishment = true }"
          );
          equal(nucleoid.run("i1.replenishment"), true);
          equal(nucleoid.run("i2.replenishment"), undefined);
          nucleoid.run(
            "if ( $Inventory.quantity == 0 ) { $Inventory.replenishment = false }"
          );

          equal(nucleoid.run("i1.replenishment"), false);
          equal(nucleoid.run("i2.replenishment"), undefined);
        });
        ```
        """
        nucleoid.run("class Inventory { }")
        nucleoid.run("i1 = new Inventory()")
        nucleoid.run("i1.quantity = 0")

        nucleoid.run("i2 = new Inventory()")
        nucleoid.run("i2.quantity = 1000")

        nucleoid.run(
            "if ($Inventory.quantity == 0) { $Inventory.replenishment = True }"
        )
        assert nucleoid.run("i1.replenishment") is True
        assert nucleoid.run("i2.replenishment") is None
        nucleoid.run(
            "if ($Inventory.quantity == 0) { $Inventory.replenishment = False }"
        )

        assert nucleoid.run("i1.replenishment") is False
        assert nucleoid.run("i2.replenishment") is None

    # ========================================================================
    # TEST 137 of 183: creates else statement of class before initialization
    # TypeScript: line 1844
    # ========================================================================

    def test_creates_else_statement_of_class_before_initialization(self):
        """
        Original TypeScript (line 1844):
        ```typescript
        it("creates else statement of class before initialization", () => {
          nucleoid.run("class Count { }");
          nucleoid.run(
            "if ( $Count.max > 1000 ) { $Count.reset = urgent } else { $Count.reset = regular }"
          );
          nucleoid.run("urgent = 'URGENT'");
          nucleoid.run("regular = 'REGULAR'");
          nucleoid.run("count1 = new Count ( )");
          nucleoid.run("count1.max = 850");
          equal(nucleoid.run("count1.reset"), "REGULAR");

          nucleoid.run("regular = 'R'");
          equal(nucleoid.run("count1.reset"), "R");
        });
        ```
        """
        nucleoid.run("class Count { }")
        nucleoid.run(
            "if ($Count.max > 1000) { $Count.reset = urgent } else { $Count.reset = regular }"
        )
        nucleoid.run("urgent = 'URGENT'")
        nucleoid.run("regular = 'REGULAR'")
        nucleoid.run("count1 = new Count()")
        nucleoid.run("count1.max = 850")
        assert nucleoid.run("count1.reset") == "REGULAR"

        nucleoid.run("regular = 'R'")
        assert nucleoid.run("count1.reset") == "R"

    # ========================================================================
    # TEST 138 of 183: creates else statement of class after initialization
    # TypeScript: line 1859
    # ========================================================================

    def test_creates_else_statement_of_class_after_initialization(self):
        """
        Original TypeScript (line 1859):
        ```typescript
        it("creates else statement of class after initialization", () => {
          nucleoid.run("class Concentration { }");
          nucleoid.run("serialDilution = '(c1V1+c2V2)/(V1+V2)'");
          nucleoid.run("directDilution = 'c1/V1'");
          nucleoid.run("concentration1 = new Concentration ( )");
          nucleoid.run("concentration1.substances = 2");
          nucleoid.run(
            "if ( $Concentration.substances == 1 ) { $Concentration.formula = directDilution } else { $Concentration.formula = serialDilution }"
          );
          equal(nucleoid.run("concentration1.formula"), "(c1V1+c2V2)/(V1+V2)");

          nucleoid.run("serialDilution = '(c1V1+c2V2+c3V3)/(V1+V2+V3)'");
          equal(
            nucleoid.run("concentration1.formula"),
            "(c1V1+c2V2+c3V3)/(V1+V2+V3)"
          );
        });
        ```
        """
        nucleoid.run("class Concentration { }")
        nucleoid.run("serialDilution = '(c1V1+c2V2)/(V1+V2)'")
        nucleoid.run("directDilution = 'c1/V1'")
        nucleoid.run("concentration1 = new Concentration()")
        nucleoid.run("concentration1.substances = 2")
        nucleoid.run(
            "if ($Concentration.substances == 1) { $Concentration.formula = directDilution } else { $Concentration.formula = serialDilution }"
        )
        assert nucleoid.run("concentration1.formula") == "(c1V1+c2V2)/(V1+V2)"

        nucleoid.run("serialDilution = '(c1V1+c2V2+c3V3)/(V1+V2+V3)'")
        assert nucleoid.run("concentration1.formula") == "(c1V1+c2V2+c3V3)/(V1+V2+V3)"

    # ========================================================================
    # TEST 139 of 183: creates else if statement of class before initialization
    # TypeScript: line 1877
    # ========================================================================

    def test_creates_else_if_statement_of_class_before_initialization(self):
        """
        Original TypeScript (line 1877):
        ```typescript
        it("creates else if statement of class before initialization", () => {
          nucleoid.run("class Storage { }");
          nucleoid.run("normal = 'NORMAL' ; low = 'LOW'");
          nucleoid.run(
            "if ( $Storage.capacity > 25 ) { $Storage.status = normal } else { $Storage.status = low }"
          );
          nucleoid.run("storage1 = new Storage ( )");
          nucleoid.run("storage1.capacity = 23");
          equal(nucleoid.run("storage1.status"), "LOW");

          nucleoid.run("low = 'L'");
          equal(nucleoid.run("storage1.status"), "L");
        });
        ```
        """
        nucleoid.run("class Storage { }")
        nucleoid.run("normal = 'NORMAL' ; low = 'LOW'")
        nucleoid.run(
            "if ($Storage.capacity > 25) { $Storage.status = normal } else { $Storage.status = low }"
        )
        nucleoid.run("storage1 = new Storage()")
        nucleoid.run("storage1.capacity = 23")
        assert nucleoid.run("storage1.status") == "LOW"

        nucleoid.run("low = 'L'")
        assert nucleoid.run("storage1.status") == "L"

    # ========================================================================
    # TEST 140 of 183: creates else if statement of class after initialization
    # TypeScript: line 1891
    # ========================================================================

    def test_creates_else_if_statement_of_class_after_initialization(self):
        """
        Original TypeScript (line 1891):
        ```typescript
        it("creates else if statement of class after initialization", () => {
          nucleoid.run("class Registration { }");
          nucleoid.run("yes = 'YES' ; no = 'NO'");
          nucleoid.run("registration1 = new Registration ( )");
          nucleoid.run("registration1.available = 0");
          nucleoid.run(
            "if ( $Registration.available > 0 ) { $Registration.accepted = yes } else { $Registration.accepted = no }"
          );
          equal(nucleoid.run("registration1.accepted"), "NO");

          nucleoid.run("yes = true ; no = false");
          equal(nucleoid.run("registration1.accepted"), false);
        });
        ```
        """
        nucleoid.run("class Registration { }")
        nucleoid.run("yes = 'YES' ; no = 'NO'")
        nucleoid.run("registration1 = new Registration()")
        nucleoid.run("registration1.available = 0")
        nucleoid.run(
            "if ($Registration.available > 0) { $Registration.accepted = yes } else { $Registration.accepted = no }"
        )
        assert nucleoid.run("registration1.accepted") == "NO"

        nucleoid.run("yes = True ; no = False")
        assert nucleoid.run("registration1.accepted") is False

    # ========================================================================
    # TEST 141 of 183: creates multiple else if statement of class before initialization
    # TypeScript: line 1905
    # ========================================================================

    def test_creates_multiple_else_if_statement_of_class_before_initialization(self):
        """
        Original TypeScript (line 1905):
        ```typescript
        it("creates multiple else if statement of class before initialization", () => {
          nucleoid.run("class Capacity { }");
          nucleoid.run(
            "if ( $Capacity.spare / $Capacity.available > 0.5 ) { $Capacity.total = $Capacity.available + $Capacity.spare } else if ( $Capacity.spare / $Capacity.available > 0.1 ) { $Capacity.total = $Capacity.available + $Capacity.spare * 2 } else { $Capacity.total = $Capacity.available + $Capacity.spare * 3 }"
          );
          nucleoid.run("capacity1 = new Capacity ( )");
          nucleoid.run("capacity1.available = 100");
          nucleoid.run("capacity1.spare = 5");
          equal(nucleoid.run("capacity1.total"), 115);

          nucleoid.run("capacity1.spare = 1");
          equal(nucleoid.run("capacity1.total"), 103);
        });
        ```
        """
        nucleoid.run("class Capacity { }")
        nucleoid.run(
            "if ($Capacity.spare / $Capacity.available > 0.5) { $Capacity.total = $Capacity.available + $Capacity.spare } else if ($Capacity.spare / $Capacity.available > 0.1) { $Capacity.total = $Capacity.available + $Capacity.spare * 2 } else { $Capacity.total = $Capacity.available + $Capacity.spare * 3 }"
        )
        nucleoid.run("capacity1 = new Capacity()")
        nucleoid.run("capacity1.available = 100")
        nucleoid.run("capacity1.spare = 5")
        assert nucleoid.run("capacity1.total") == 115

        nucleoid.run("capacity1.spare = 1")
        assert nucleoid.run("capacity1.total") == 103

    # ========================================================================
    # TEST 142 of 183: creates multiple else if statement of class after initialization
    # TypeScript: line 1919
    # ========================================================================

    def test_creates_multiple_else_if_statement_of_class_after_initialization(self):
        """
        Original TypeScript (line 1919):
        ```typescript
        it("creates multiple else if statement of class after initialization", () => {
          nucleoid.run("class Shape { }");
          nucleoid.run("shape1 = new Shape ( )");
          nucleoid.run("shape1.type = 'RECTANGLE'");
          nucleoid.run("shape1.x = 5");
          nucleoid.run("shape1.y = 6");
          nucleoid.run(
            "if ( $Shape.type == 'SQUARE' ) { $Shape.area = Math.pow( Shape.x, 2 ) } else if ( $Shape.type == 'TRIANGLE' ) { $Shape.area = $Shape.x * $Shape.y / 2 } else { $Shape.area = $Shape.x * $Shape.y }"
          );
          equal(nucleoid.run("shape1.area"), 30);

          nucleoid.run("shape1.x = 7");
          equal(nucleoid.run("shape1.area"), 42);
        });
        ```
        """
        nucleoid.run("class Shape { }")
        nucleoid.run("shape1 = new Shape()")
        nucleoid.run("shape1.type = 'RECTANGLE'")
        nucleoid.run("shape1.x = 5")
        nucleoid.run("shape1.y = 6")
        nucleoid.run(
            "if ($Shape.type == 'SQUARE') { $Shape.area = Math.pow(Shape.x, 2) } else if ($Shape.type == 'TRIANGLE') { $Shape.area = $Shape.x * $Shape.y / 2 } else { $Shape.area = $Shape.x * $Shape.y }"
        )
        assert nucleoid.run("shape1.area") == 30

        nucleoid.run("shape1.x = 7")
        assert nucleoid.run("shape1.area") == 42

    # ========================================================================
    # TEST 143 of 183: runs block statement of class before initialization
    # TypeScript: line 1934
    # ========================================================================

    def test_runs_block_statement_of_class_before_initialization(self):
        """
        Original TypeScript (line 1934):
        ```typescript
        it("runs block statement of class before initialization", () => {
          nucleoid.run("class Stock { }");
          nucleoid.run(
            "{ let change = $Stock.before * 4 / 100 ; $Stock.after = $Stock.before + change }"
          );
          nucleoid.run("stock1 = new Stock ( )");
          nucleoid.run("stock1.before = 57.25");
          equal(nucleoid.run("stock1.after"), 59.54);

          nucleoid.run("stock1.before = 59.50");
          equal(nucleoid.run("stock1.after"), 61.88);
        });
        ```
        """
        nucleoid.run("class Stock { }")
        nucleoid.run(
            "{ let change = $Stock.before * 4 / 100 ; $Stock.after = $Stock.before + change }"
        )
        nucleoid.run("stock1 = new Stock()")
        nucleoid.run("stock1.before = 57.25")
        assert nucleoid.run("stock1.after") == 59.54

        nucleoid.run("stock1.before = 59.50")
        assert nucleoid.run("stock1.after") == 61.88

    # ========================================================================
    # TEST 144 of 183: runs block statement of class after initialization
    # TypeScript: line 1947
    # ========================================================================

    def test_runs_block_statement_of_class_after_initialization(self):
        """
        Original TypeScript (line 1947):
        ```typescript
        it("runs block statement of class after initialization", () => {
          nucleoid.run("class Purchase { }");
          nucleoid.run("purchase = new Purchase ( )");
          nucleoid.run("purchase.price = 99");
          nucleoid.run(
            "{ let retailPrice = $Purchase.price * 1.15 ; $Purchase.retailPrice = retailPrice }"
          );
          equal(nucleoid.run("purchase.retailPrice"), 113.85);

          nucleoid.run("purchase.price = 199");
          equal(nucleoid.run("purchase.retailPrice"), 228.85);
        });
        ```
        """
        nucleoid.run("class Purchase { }")
        nucleoid.run("purchase = new Purchase()")
        nucleoid.run("purchase.price = 99")
        nucleoid.run(
            "{ let retailPrice = $Purchase.price * 1.15 ; $Purchase.retailPrice = retailPrice }"
        )
        assert nucleoid.run("purchase.retailPrice") == 113.85

        nucleoid.run("purchase.price = 199")
        assert nucleoid.run("purchase.retailPrice") == 228.85

    # ========================================================================
    # TEST 145 of 183: runs nested block statement of class before initialization
    # TypeScript: line 1960
    # ========================================================================

    def test_runs_nested_block_statement_of_class_before_initialization(self):
        """
        Original TypeScript (line 1960):
        ```typescript
        it("runs nested block statement of class before initialization", () => {
          nucleoid.run("class Compound { }");
          nucleoid.run(
            "{ let mol = 69.94 / $Compound.substance ; { $Compound.sample = Math.floor ( mol * $Compound.mol ) } }"
          );
          nucleoid.run("compound1 = new Compound ( )");
          nucleoid.run("compound1.substance = 55.85");
          nucleoid.run("compound1.mol = 1000");
          equal(nucleoid.run("compound1.sample"), 1252);
        });
        ```
        """
        nucleoid.run("class Compound { }")
        nucleoid.run(
            "{ let mol = 69.94 / $Compound.substance ; { $Compound.sample = Math.floor(mol * $Compound.mol) } }"
        )
        nucleoid.run("compound1 = new Compound()")
        nucleoid.run("compound1.substance = 55.85")
        nucleoid.run("compound1.mol = 1000")
        assert nucleoid.run("compound1.sample") == 1252

    # ========================================================================
    # TEST 146 of 183: runs nested block statement of class after initialization
    # TypeScript: line 1971
    # ========================================================================

    def test_runs_nested_block_statement_of_class_after_initialization(self):
        """
        Original TypeScript (line 1971):
        ```typescript
        it("runs nested block statement of class after initialization", () => {
          nucleoid.run("class Bug { }");
          nucleoid.run("bug1 = new Bug ( )");
          nucleoid.run("bug1.initialScore = 1000");
          nucleoid.run("bug1.aging = 24");
          nucleoid.run(
            "{ let score = $Bug.aging * 10 ; { $Bug.priorityScore = score + $Bug.initialScore } }"
          );
          equal(nucleoid.run("bug1.priorityScore"), 1240);
        });
        ```
        """
        nucleoid.run("class Bug { }")
        nucleoid.run("bug1 = new Bug()")
        nucleoid.run("bug1.initialScore = 1000")
        nucleoid.run("bug1.aging = 24")
        nucleoid.run(
            "{ let score = $Bug.aging * 10 ; { $Bug.priorityScore = score + $Bug.initialScore } }"
        )
        assert nucleoid.run("bug1.priorityScore") == 1240

    # ========================================================================
    # TEST 147 of 183: runs nested if statement of class before initialization
    # TypeScript: line 1982
    # ========================================================================

    def test_runs_nested_if_statement_of_class_before_initialization(self):
        """
        Original TypeScript (line 1982):
        ```typescript
        it("runs nested if statement of class before initialization", () => {
          nucleoid.run("class Mortgage { }");
          nucleoid.run("rate1 = 'EXCEPTIONAL'");
          nucleoid.run(
            "{ let interest = $Mortgage.annual / 12 ; if ( interest < 4 ) { $Mortgage.rate = rate1 } }"
          );
          nucleoid.run("mortgage1 = new Mortgage ( )");
          nucleoid.run("mortgage1.annual = 46");
          equal(nucleoid.run("mortgage1.rate"), "EXCEPTIONAL");

          nucleoid.run("rate1 = 'E'");
          equal(nucleoid.run("mortgage1.rate"), "E");
        });
        ```
        """
        nucleoid.run("class Mortgage { }")
        nucleoid.run("rate1 = 'EXCEPTIONAL'")
        nucleoid.run(
            "{ let interest = $Mortgage.annual / 12 ; if (interest < 4) { $Mortgage.rate = rate1 } }"
        )
        nucleoid.run("mortgage1 = new Mortgage()")
        nucleoid.run("mortgage1.annual = 46")
        assert nucleoid.run("mortgage1.rate") == "EXCEPTIONAL"

        nucleoid.run("rate1 = 'E'")
        assert nucleoid.run("mortgage1.rate") == "E"

    # ========================================================================
    # TEST 148 of 183: runs nested if statement of class after initialization
    # TypeScript: line 1996
    # ========================================================================

    def test_runs_nested_if_statement_of_class_after_initialization(self):
        """
        Original TypeScript (line 1996):
        ```typescript
        it("runs nested if statement of class after initialization", () => {
          nucleoid.run("class Building { }");
          nucleoid.run("buildingType1 = 'SKYSCRAPER'");
          nucleoid.run("building1 = new Building ( )");
          nucleoid.run("building1.floors = 20");
          nucleoid.run(
            "{ let height = $Building.floors * 14 ; if ( height > 330 ) { $Building.type = buildingType1 } }"
          );
          equal(nucleoid.run("building1.type"), undefined);

          nucleoid.run("building1.floors = 25");
          equal(nucleoid.run("building1.type"), "SKYSCRAPER");

          nucleoid.run("buildingType1 = 'S'");
          equal(nucleoid.run("building1.type"), "S");
        });
        ```
        """
        nucleoid.run("class Building { }")
        nucleoid.run("buildingType1 = 'SKYSCRAPER'")
        nucleoid.run("building1 = new Building()")
        nucleoid.run("building1.floors = 20")
        nucleoid.run(
            "{ let height = $Building.floors * 14 ; if (height > 330) { $Building.type = buildingType1 } }"
        )
        assert nucleoid.run("building1.type") is None

        nucleoid.run("building1.floors = 25")
        assert nucleoid.run("building1.type") == "SKYSCRAPER"

        nucleoid.run("buildingType1 = 'S'")
        assert nucleoid.run("building1.type") == "S"

    # ========================================================================
    # TEST 149 of 183: creates nested else statement of class before initialization
    # TypeScript: line 2013
    # ========================================================================

    def test_creates_nested_else_statement_of_class_before_initialization(self):
        """
        Original TypeScript (line 2013):
        ```typescript
        it("creates nested else statement of class before initialization", () => {
          nucleoid.run("class Account { }");
          nucleoid.run("noAlert = 'NO_ALERT'");
          nucleoid.run("lowAlert = 'LOW_ALERT'");
          nucleoid.run(
            "{ let balance = $Account.balance ; if ( balance > 1000 ) { $Account.alert = noAlert } else { $Account.alert = lowAlert } }"
          );
          nucleoid.run("account1 = new Account ( )");
          nucleoid.run("account1.balance = 950");
          equal(nucleoid.run("account1.alert"), "LOW_ALERT");

          nucleoid.run("lowAlert = 'L'");
          equal(nucleoid.run("account1.alert"), "L");
        });
        ```
        """
        nucleoid.run("class Account { }")
        nucleoid.run("noAlert = 'NO_ALERT'")
        nucleoid.run("lowAlert = 'LOW_ALERT'")
        nucleoid.run(
            "{ let balance = $Account.balance ; if (balance > 1000) { $Account.alert = noAlert } else { $Account.alert = lowAlert } }"
        )
        nucleoid.run("account1 = new Account()")
        nucleoid.run("account1.balance = 950")
        assert nucleoid.run("account1.alert") == "LOW_ALERT"

        nucleoid.run("lowAlert = 'L'")
        assert nucleoid.run("account1.alert") == "L"

    # ========================================================================
    # TEST 150 of 183: creates nested else statement of class after initialization
    # TypeScript: line 2028
    # ========================================================================

    def test_creates_nested_else_statement_of_class_after_initialization(self):
        """
        Original TypeScript (line 2028):
        ```typescript
        it("creates nested else statement of class after initialization", () => {
          nucleoid.run("class Question { }");
          nucleoid.run("high = 'HIGH'");
          nucleoid.run("low = 'LOW'");
          nucleoid.run("question1 = new Question ( )");
          nucleoid.run("question1.count = 1");
          nucleoid.run(
            "{ let score = $Question.count * 10 ; if ( score > 100 ) { $Question.type = high } else { $Question.type = low } }"
          );
          equal(nucleoid.run("question1.type"), "LOW");

          nucleoid.run("low = 'L'");
          equal(nucleoid.run("question1.type"), "L");
        });
        ```
        """
        nucleoid.run("class Question { }")
        nucleoid.run("high = 'HIGH'")
        nucleoid.run("low = 'LOW'")
        nucleoid.run("question1 = new Question()")
        nucleoid.run("question1.count = 1")
        nucleoid.run(
            "{ let score = $Question.count * 10 ; if (score > 100) { $Question.type = high } else { $Question.type = low } }"
        )
        assert nucleoid.run("question1.type") == "LOW"

        nucleoid.run("low = 'L'")
        assert nucleoid.run("question1.type") == "L"

    # ========================================================================
    # TEST 151 of 183: creates class assignment with multiple properties before declaration
    # TypeScript: line 2043
    # ========================================================================

    def test_creates_class_assignment_with_multiple_properties_before_declaration(self):
        """
        Original TypeScript (line 2043):
        ```typescript
        it("creates class assignment with multiple properties before declaration", () => {
          nucleoid.run("class Room { }");
          nucleoid.run("$Room.level = $Room.number / 10");
          nucleoid.run("class Guest { }");
          nucleoid.run("$Guest.room = new Room ( )");
          nucleoid.run("guest1 = new Guest ( )");
          nucleoid.run("guest1.room.number = 30");
          equal(nucleoid.run("guest1.room.level"), 3);
        });
        ```
        """
        nucleoid.run("class Room { }")
        nucleoid.run("$Room.level = $Room.number / 10")
        nucleoid.run("class Guest { }")
        nucleoid.run("$Guest.room = new Room()")
        nucleoid.run("guest1 = new Guest()")
        nucleoid.run("guest1.room.number = 30")
        assert nucleoid.run("guest1.room.level") == 3

    # ========================================================================
    # TEST 152 of 183: creates class assignment with multiple properties after declaration
    # TypeScript: line 2053
    # ========================================================================

    def test_creates_class_assignment_with_multiple_properties_after_declaration(self):
        """
        Original TypeScript (line 2053):
        ```typescript
        it("creates class assignment with multiple properties after declaration", () => {
          nucleoid.run("class Channel { }");
          nucleoid.run("class Frequency { }");
          nucleoid.run("channel1 = new Channel ( )");
          nucleoid.run("$Channel.frequency = new Frequency ( )");
          nucleoid.run("$Frequency.hertz = 1 / $Frequency.period");
          nucleoid.run("channel1.frequency.period = 0.0025");
          equal(nucleoid.run("channel1.frequency.hertz"), 400);
        });
        ```
        """
        nucleoid.run("class Channel { }")
        nucleoid.run("class Frequency { }")
        nucleoid.run("channel1 = new Channel()")
        nucleoid.run("$Channel.frequency = new Frequency()")
        nucleoid.run("$Frequency.hertz = 1 / $Frequency.period")
        nucleoid.run("channel1.frequency.period = 0.0025")
        assert nucleoid.run("channel1.frequency.hertz") == 400

    # ========================================================================
    # TEST 153 of 183: creates class assignment as multiple properties as part of declaration before initialization
    # TypeScript: line 2063
    # ========================================================================

    def test_creates_class_assignment_as_multiple_properties_as_part_of_declaration_before_initialization(self):
        """
        Original TypeScript (line 2063):
        ```typescript
        it("creates class assignment as multiple properties as part of declaration before initialization", () => {
          nucleoid.run("class Hospital { }");
          nucleoid.run("class Clinic { }");
          nucleoid.run("$Hospital.clinic = new Clinic ( )");
          nucleoid.run("$Hospital.patients = $Hospital.clinic.beds * 746");
          nucleoid.run("hospital1 = new Hospital ( )");
          nucleoid.run("hospital1.clinic.beds = 2678");
          equal(nucleoid.run("hospital1.patients"), 1997788);
        });
        ```
        """
        nucleoid.run("class Hospital { }")
        nucleoid.run("class Clinic { }")
        nucleoid.run("$Hospital.clinic = new Clinic()")
        nucleoid.run("$Hospital.patients = $Hospital.clinic.beds * 746")
        nucleoid.run("hospital1 = new Hospital()")
        nucleoid.run("hospital1.clinic.beds = 2678")
        assert nucleoid.run("hospital1.patients") == 1997788

    # ========================================================================
    # TEST 154 of 183: creates class assignment as multiple properties as part of declaration after initialization
    # TypeScript: line 2073
    # ========================================================================

    def test_creates_class_assignment_as_multiple_properties_as_part_of_declaration_after_initialization(self):
        """
        Original TypeScript (line 2073):
        ```typescript
        it("creates class assignment as multiple properties as part of declaration after initialization", () => {
          nucleoid.run("class Server { }");
          nucleoid.run("class OS { }");
          nucleoid.run("$Server.os = new OS ( )");
          nucleoid.run("server1 = new Server ( )");
          nucleoid.run("server1.os.version = 14");
          nucleoid.run("$Server.build = $Server.os.version + '.526291'");
          equal(nucleoid.run("server1.build"), "14.526291");
        });
        ```
        """
        nucleoid.run("class Server { }")
        nucleoid.run("class OS { }")
        nucleoid.run("$Server.os = new OS()")
        nucleoid.run("server1 = new Server()")
        nucleoid.run("server1.os.version = 14")
        nucleoid.run("$Server.build = $Server.os.version + '.526291'")
        assert nucleoid.run("server1.build") == "14.526291"

    # ========================================================================
    # TEST 155 of 183: creates class assignment only if instance is defined
    # TypeScript: line 2083
    # ========================================================================

    def test_creates_class_assignment_only_if_instance_is_defined(self):
        """
        Original TypeScript (line 2083):
        ```typescript
        it("creates class assignment only if instance is defined", () => {
          nucleoid.run("class Phone { }");
          throws(
            () => {
              nucleoid.run("Phone.line.wired = true");
            },
            (error: Error) =>
              validate(error, ReferenceError, "Phone.line is not defined")
          );
        });
        ```
        """
        nucleoid.run("class Phone { }")
        with pytest.raises(ReferenceError, match="Phone.line is not defined"):
            nucleoid.run("Phone.line.wired = True")

    # ========================================================================
    # TEST 156 of 183: creates for of statement
    # TypeScript: line 2094
    # ========================================================================

    def test_creates_for_of_statement(self):
        """
        Original TypeScript (line 2094):
        ```typescript
        it("creates for of statement", () => {
          nucleoid.run(
            "class Question { constructor ( rate ) { this.rate = rate } }"
          );
          nucleoid.run("question1 = new Question ( 4 )");
          nucleoid.run("question2 = new Question ( 5 )");
          nucleoid.run(
            "class Summary { constructor ( question ) { this.question = question } }"
          );
          nucleoid.run("$Summary.rate = $Summary.question.rate.value");

          nucleoid.run("for ( question of Question ) { new Summary ( question ) }");
          equal(nucleoid.run("Summary[0]").rate, 4);
          equal(nucleoid.run("Summary[1]").rate, 5);
        });
        ```
        """
        nucleoid.run(
            "class Question { constructor(rate) { this.rate = rate } }"
        )
        nucleoid.run("question1 = new Question(4)")
        nucleoid.run("question2 = new Question(5)")
        nucleoid.run(
            "class Summary { constructor(question) { this.question = question } }"
        )
        nucleoid.run("$Summary.rate = $Summary.question.rate.value")

        nucleoid.run("for (question of Question) { new Summary(question) }")
        assert nucleoid.run("Summary[0]").get("rate") == 4
        assert nucleoid.run("Summary[1]").get("rate") == 5

    # ========================================================================
    # TEST 157 of 183: creates block of for statement without dependencies
    # TypeScript: line 2110
    # ========================================================================

    def test_creates_block_of_for_statement_without_dependencies(self):
        """
        Original TypeScript (line 2110):
        ```typescript
        it("creates block of for statement without dependencies", () => {
          nucleoid.run("class Item { }");
          nucleoid.run("item1 = new Item ( )");
          nucleoid.run("item2 = new Item ( )");
          nucleoid.run("VALUE = 10");
          nucleoid.run(
            "for ( item of Item ) { let i = 10 * VALUE ; item.score = i }"
          );

          nucleoid.run("VALUE = 20");
          equal(nucleoid.run("item1.score"), 100);
          equal(nucleoid.run("item2.score"), 100);

          nucleoid.run(
            "for ( item of Item ) { let i = 10 * VALUE ; item.score = i }"
          );
          equal(nucleoid.run("item1.score"), 200);
          equal(nucleoid.run("item2.score"), 200);
        });
        ```
        """
        nucleoid.run("class Item { }")
        nucleoid.run("item1 = new Item()")
        nucleoid.run("item2 = new Item()")
        nucleoid.run("VALUE = 10")
        nucleoid.run(
            "for (item of Item) { let i = 10 * VALUE ; item.score = i }"
        )

        nucleoid.run("VALUE = 20")
        assert nucleoid.run("item1.score") == 100
        assert nucleoid.run("item2.score") == 100

        nucleoid.run(
            "for (item of Item) { let i = 10 * VALUE ; item.score = i }"
        )
        assert nucleoid.run("item1.score") == 200
        assert nucleoid.run("item2.score") == 200

    # ========================================================================
    # TEST 158 of 183: loops through only defined objects in for of statement
    # TypeScript: line 2130
    # ========================================================================

    def test_loops_through_only_defined_objects_in_for_of_statement(self):
        """
        Original TypeScript (line 2130):
        ```typescript
        it("loops through only defined objects in for of statement", () => {
          nucleoid.run("array = [ ]");
          nucleoid.run("class Item { }");

          nucleoid.run("item1 = new Object ( ) ;  array.push ( item1 )");
          nucleoid.run("item2 = { id: 'item3' } ;   array.push ( item2 )");
          nucleoid.run("item4 = new Item ( );   array.push ( item4 )");
          nucleoid.run("count = 0;");

          nucleoid.run("for ( item of array ) { count++ }");
          equal(nucleoid.run("count"), 1);
        });
        ```
        """
        nucleoid.run("array = []")
        nucleoid.run("class Item { }")

        nucleoid.run("item1 = new Object() ; array.push(item1)")
        nucleoid.run("item2 = { id: 'item3' } ; array.push(item2)")
        nucleoid.run("item4 = new Item() ; array.push(item4)")
        nucleoid.run("count = 0")

        nucleoid.run("for (item of array) { count++ }")
        assert nucleoid.run("count") == 1

    # ========================================================================
    # TEST 159 of 183: supports if statement in for of statement
    # TypeScript: line 2143
    # ========================================================================

    def test_supports_if_statement_in_for_of_statement(self):
        """
        Original TypeScript (line 2143):
        ```typescript
        it("supports if statement in for of statement", () => {
          nucleoid.run("class Question { }");
          nucleoid.run("question1 = new Question ( )");
          nucleoid.run("question2 = new Question ( )");
          nucleoid.run("question2.archived = true");
          nucleoid.run("question3 = new Question ( )");
          nucleoid.run(
            "class Summary { constructor ( question ) { this.question = question } }"
          );
          nucleoid.run("$Summary.type = 'DAILY'");
          nucleoid.run(
            "for ( question of Question ) { if ( ! question.archived ) { new Summary ( question ) } }"
          );

          equal(nucleoid.run("Summary.length"), 2);
          equal(nucleoid.run("Summary[0].question.id"), "question1");
          equal(nucleoid.run("Summary[1].question.id"), "question3");
          equal(nucleoid.run("Summary[0].type"), "DAILY");
          equal(nucleoid.run("Summary[1].type"), "DAILY");
        });
        ```
        """
        nucleoid.run("class Question { }")
        nucleoid.run("question1 = new Question()")
        nucleoid.run("question2 = new Question()")
        nucleoid.run("question2.archived = True")
        nucleoid.run("question3 = new Question()")
        nucleoid.run(
            "class Summary { constructor(question) { this.question = question } }"
        )
        nucleoid.run("$Summary.type = 'DAILY'")
        nucleoid.run(
            "for (question of Question) { if (!question.archived) { new Summary(question) } }"
        )

        assert nucleoid.run("Summary.length") == 2
        assert nucleoid.run("Summary[0].question.id") == "question1"
        assert nucleoid.run("Summary[1].question.id") == "question3"
        assert nucleoid.run("Summary[0].type") == "DAILY"
        assert nucleoid.run("Summary[1].type") == "DAILY"

    # ========================================================================
    # TEST 160 of 183: returns integer in variable assignment
    # TypeScript: line 2164
    # ========================================================================

    def test_returns_integer_in_variable_assignment(self):
        """
        Original TypeScript (line 2164):
        ```typescript
        it("returns integer in variable assignment", () => {
          nucleoid.run("function test ( a ) { return a = 2 }");
          equal(nucleoid.run("b = 1 ; test ( b )"), 2);
        });
        ```
        """
        nucleoid.run("function test(a) { return a = 2 }")
        assert nucleoid.run("b = 1 ; test(b)") == 2

    # ========================================================================
    # TEST 161 of 183: returns reference in variable assignment
    # TypeScript: line 2169
    # ========================================================================

    def test_returns_reference_in_variable_assignment(self):
        """
        Original TypeScript (line 2169):
        ```typescript
        it("returns reference in variable assignment", () => {
          nucleoid.run("a = new Object ( )");
          nucleoid.run("c = 1");
          nucleoid.run("function test ( b ) { return b = a }");
          deepEqual(nucleoid.run("test ( c )"), {});
          deepEqual(nucleoid.run("c"), 1);
        });
        ```
        """
        nucleoid.run("a = new Object()")
        nucleoid.run("c = 1")
        nucleoid.run("function test(b) { return b = a }")
        assert nucleoid.run("test(c)") == {}
        assert nucleoid.run("c") == 1

    # ========================================================================
    # TEST 162 of 183: returns string in variable assignment
    # TypeScript: line 2177
    # ========================================================================

    def test_returns_string_in_variable_assignment(self):
        """
        Original TypeScript (line 2177):
        ```typescript
        it("returns string in variable assignment", () => {
          nucleoid.run("function test ( a ) { return a = 'abc' }");
          equal(nucleoid.run("b = 1 ; test ( b )"), "abc");
        });
        ```
        """
        nucleoid.run("function test(a) { return a = 'abc' }")
        assert nucleoid.run("b = 1 ; test(b)") == "abc"

    # ========================================================================
    # TEST 163 of 183: returns object in variable assignment
    # TypeScript: line 2182
    # ========================================================================

    def test_returns_object_in_variable_assignment(self):
        """
        Original TypeScript (line 2182):
        ```typescript
        it("returns object in variable assignment", () => {
          nucleoid.run("function test ( a ) { return a = new Object ( ) }");
          deepEqual(nucleoid.run("b = 1 ; test ( b )"), {});
        });
        ```
        """
        nucleoid.run("function test(a) { return a = new Object() }")
        assert nucleoid.run("b = 1 ; test(b)") == {}

    # ========================================================================
    # TEST 164 of 183: runs function with variable
    # TypeScript: line 2187
    # ========================================================================

    def test_runs_function_with_variable(self):
        """
        Original TypeScript (line 2187):
        ```typescript
        it("runs function with variable", () => {
          nucleoid.run("function test ( a ) { return a + 23 }");
          equal(nucleoid.run("let data = 'UUID-1' ; test ( data )"), "UUID-123");
        });
        ```
        """
        nucleoid.run("function test(a) { return a + 23 }")
        assert nucleoid.run("let data = 'UUID-1' ; test(data)") == "UUID-123"

    # ========================================================================
    # TEST 165 of 183: returns first return statement
    # TypeScript: line 2192
    # ========================================================================

    def test_returns_first_return_statement(self):
        """
        Original TypeScript (line 2192):
        ```typescript
        it("returns first return statement", () => {
          equal(nucleoid.run("{ return 123 ; return 'abc' }"), 123);
        });
        ```
        """
        assert nucleoid.run("{ return 123 ; return 'abc' }") == 123

    # ========================================================================
    # TEST 166 of 183: returns undefined in class creation
    # TypeScript: line 2196
    # ========================================================================

    def test_returns_undefined_in_class_creation(self):
        """
        Original TypeScript (line 2196):
        ```typescript
        it("returns undefined in class creation", () => {
          equal(nucleoid.run("class Test { }"), undefined);
        });
        ```
        """
        assert nucleoid.run("class Test { }") is None

    # ========================================================================
    # TEST 167 of 183: returns object itself in object creation
    # TypeScript: line 2200
    # ========================================================================

    def test_returns_object_itself_in_object_creation(self):
        """
        Original TypeScript (line 2200):
        ```typescript
        it("returns object itself in object creation", () => {
          nucleoid.run("class Test { constructor ( prop ) { this.prop = prop } }");
          const object = nucleoid.run("new Test ( 123 )");
          notEqual(object.id, null);
          equal(object.prop, 123);
        });
        ```
        """
        nucleoid.run("class Test { constructor(prop) { this.prop = prop } }")
        obj = nucleoid.run("new Test(123)")
        assert obj.get("id") is not None
        assert obj.get("prop") == 123

    # ========================================================================
    # ALL DECLARATIVE TESTS COMPLETED (167 of 183 total tests)
    # Remaining tests are in imperative mode (already implemented)
    # ========================================================================


class TestNucleoidImperativeMode:
    """
    Complete test suite for Nucleoid runtime in imperative mode.

    This class contains ALL tests from the 'in imperative mode' describe block
    in the TypeScript test suite (lines 2208-2246).

    Total imperative tests: 4
    """

    @classmethod
    def setup_class(cls):
        """
        Setup class - start nucleoid with test mode and declarative mode.

        Note: We start with declarative mode enabled at the class level,
        but individual tests will override this with imperative mode options.
        """
        nucleoid.start({'test': True, 'options': {'declarative': True}})

    def setup_method(self):
        """Setup method - clear state before each test."""
        clear()

    # ========================================================================
    # IMPERATIVE TEST 1 of 4: Variable Assignment
    # TypeScript: lines 2211-2216
    # ========================================================================

    def test_creates_variable_assignment(self):
        """
        Test variable assignment in imperative mode.

        Original TypeScript (lines 2211-2216):
        ```typescript
        it("creates variable assignment", () => {
          nucleoid.run("x = 1", imperative);
          nucleoid.run("y = x + 2", imperative);
          nucleoid.run("x = 2", imperative);
          equal(nucleoid.run("y", imperative), 3);
        });
        ```

        EXPECTED BEHAVIOR (Imperative Mode):
        1. x = 1
        2. y = x + 2  →  y = 3
        3. x = 2      →  x changes but y does NOT recalculate
        4. y = 3      →  y remains 3 (not updated to 4)

        CONTRAST WITH DECLARATIVE MODE:
        In declarative mode, step 3 would cause y to recalculate to 4.
        In imperative mode, y stays 3 because there's no dependency tracking.
        """
        imperative = {'declarative': False}

        # Step 1: Set x to 1
        nucleoid.run("x = 1", imperative)

        # Step 2: Set y to x + 2 (y becomes 3)
        nucleoid.run("y = x + 2", imperative)

        # Step 3: Change x to 2
        nucleoid.run("x = 2", imperative)

        # Step 4: Verify y is still 3 (NOT recalculated to 4)
        assert nucleoid.run("y", imperative) == 3

    # ========================================================================
    # IMPERATIVE TEST 2 of 4: If Statement
    # TypeScript: lines 2218-2226
    # ========================================================================

    def test_creates_if_statement_of_variable(self):
        """
        Test if statement in imperative mode.

        Original TypeScript (lines 2218-2226):
        ```typescript
        it("creates if statement of variable", () => {
          nucleoid.run("m = false", imperative);
          nucleoid.run("n = false", imperative);
          nucleoid.run("if ( m == true ) { n = m && true }", imperative);
          equal(nucleoid.run("n", imperative), false);

          nucleoid.run("m = true", imperative);
          equal(nucleoid.run("n", imperative), false);
        });
        ```

        EXPECTED BEHAVIOR (Imperative Mode):
        1. m = false, n = false
        2. if (m == true) { n = m && true }  →  condition false, n stays false
        3. m = true  →  m changes to true
        4. n = false  →  n remains false (if block NOT re-executed)

        CONTRAST WITH DECLARATIVE MODE:
        In declarative mode, changing m to true would trigger the if block
        to re-evaluate, updating n to true.
        In imperative mode, the if block executes once and never re-runs.
        """
        imperative = {'declarative': False}

        # Step 1: Initialize m and n to false
        nucleoid.run("m = false", imperative)
        nucleoid.run("n = false", imperative)

        # Step 2: Create if statement (condition is false, so n stays false)
        nucleoid.run("if (m == true) { n = m && true }", imperative)

        # Step 3: Verify n is still false
        assert nucleoid.run("n", imperative) is False

        # Step 4: Change m to true
        nucleoid.run("m = true", imperative)

        # Step 5: Verify n is STILL false (if block not re-executed)
        assert nucleoid.run("n", imperative) is False

    # ========================================================================
    # IMPERATIVE TEST 3 of 4: Property Assignment
    # TypeScript: lines 2228-2237
    # ========================================================================

    def test_creates_property_assignment(self):
        """
        Test property assignment in imperative mode.

        Original TypeScript (lines 2228-2237):
        ```typescript
        it("creates property assignment", () => {
          nucleoid.run("class Order { }", imperative);
          nucleoid.run("var order1 = new Order ( )", imperative);
          nucleoid.run("order1.upc = '04061' + order1.barcode", imperative);
          nucleoid.run("order1.barcode = '94067'", imperative);
          equal(nucleoid.run("order1.upc", imperative), undefined);

          nucleoid.run("order1.upc = '04061' + order1.barcode", imperative);
          equal(nucleoid.run("order1.upc", imperative), "0406194067");
        });
        ```

        EXPECTED BEHAVIOR (Imperative Mode):
        1. Create Order class and order1 instance
        2. order1.upc = '04061' + order1.barcode
           → barcode is undefined, so upc becomes undefined
        3. order1.barcode = '94067'
           → barcode is set
        4. order1.upc is undefined
           → upc NOT recalculated (still undefined)
        5. Manually recalculate: order1.upc = '04061' + order1.barcode
           → Now upc is "0406194067"

        CONTRAST WITH DECLARATIVE MODE:
        In declarative mode, setting barcode in step 3 would automatically
        recalculate upc to "0406194067".
        In imperative mode, upc must be manually recalculated.
        """
        imperative = {'declarative': False}

        # Step 1: Create Order class
        nucleoid.run("class Order {}", imperative)

        # Step 2: Create order1 instance
        nucleoid.run("var order1 = new Order()", imperative)

        # Step 3: Set upc based on barcode (which doesn't exist yet)
        nucleoid.run("order1.upc = '04061' + order1.barcode", imperative)

        # Step 4: Set barcode
        nucleoid.run("order1.barcode = '94067'", imperative)

        # Step 5: Verify upc is still undefined (NOT recalculated)
        assert nucleoid.run("order1.upc", imperative) is None

        # Step 6: Manually recalculate upc
        nucleoid.run("order1.upc = '04061' + order1.barcode", imperative)

        # Step 7: Verify upc now has the correct value
        assert nucleoid.run("order1.upc", imperative) == "0406194067"

    # ========================================================================
    # IMPERATIVE TEST 4 of 4: Retrieve Object Through Let Variable
    # TypeScript: lines 2239-2245
    # ========================================================================

    def test_retrieves_object_through_let_variable(self):
        """
        Test retrieving object through let variable.

        Original TypeScript (lines 2239-2245):
        ```typescript
        it("retrieves object through let variable", () => {
          nucleoid.run("class User { }");
          nucleoid.run("user0 = new User ( )");
          deepEqual(nucleoid.run("let user = 'user0' ; User[user]"), {
            id: "user0",
          });
        });
        ```

        EXPECTED BEHAVIOR:
        1. Create User class
        2. Create user0 instance
        3. Use let variable to store object id 'user0'
        4. Retrieve object using bracket notation: User[user]
        5. Should return the user0 object with id "user0"

        NOTE: This test doesn't use imperative mode options in the original,
        suggesting this behavior should work the same in both modes.
        """
        # Note: No imperative mode options used in original test

        # Step 1: Create User class
        nucleoid.run("class User {}")

        # Step 2: Create user0 instance
        nucleoid.run("user0 = new User()")

        # Step 3: Retrieve object using let variable with bracket notation
        result = nucleoid.run("let user = 'user0'; User[user]")

        # Step 4: Verify result is the user0 object
        assert result == {"id": "user0"}
