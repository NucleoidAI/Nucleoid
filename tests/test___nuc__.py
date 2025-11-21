"""
Tests for $nuc statement storage and restoration.

This test suite corresponds to the TypeScript test file:
typescript/src/test/$nuc.spec.ts

NOTE: In Python, we use __nuc__ instead of $nuc to follow Python naming conventions.
"""

import pytest
from nucleoid import nucleoid
from nucleoid.lib.test import clear
from nucleoid.stack import process


class TestNucStorage:
    """
    Test suite for $nuc statement storage and datastore functionality.

    This class contains the test from the '$nuc' describe block
    in the TypeScript test suite.
    """

    @classmethod
    def setup_class(cls):
        """Setup class - start nucleoid with test mode and declarative mode."""
        nucleoid.start({'test': True, 'options': {'declarative': True}})

    def setup_method(self):
        """Setup method - clear state before each test."""
        clear()

    def test_stores_nuc(self):
        """
        Test that $nuc statements are stored and can be restored from datastore.

        Original TypeScript (lines 16-55):
        ```typescript
        it("stores $nuc", () => {
          nucleoid.run("a = 1");
          nucleoid.run("b = a + 2");
          nucleoid.run("a = 2");

          nucleoid.run("arr  = [ 1, 2, 3 ]");
          nucleoid.run("arr.push ( 4 )");

          nucleoid.run(
            "class User { constructor ( name, createdAt ) { this.name = name; this.createdAt = createdAt } }"
          );
          nucleoid.run("$User.active = true");
          nucleoid.run("if ( $User.name === 'Test' ) { $User.mode = 'TEST' }");
          nucleoid.run("new User ( 'Test', Date.now() )");

          const expectedUserList = nucleoid.run("User");

          const statements = nucleoid.datastore
            .read()
            .flatMap((stmt: any) => stmt.$)
            .filter(Boolean);

          test.clear();

          process(statements, null, { declarative: true });

          equal(nucleoid.run("a"), 2);
          equal(nucleoid.run("b"), 4);

          nucleoid.run("c = b + 3");
          nucleoid.run("a = 3");

          equal(nucleoid.run("b"), 5);
          equal(nucleoid.run("c"), 8);

          deepEqual(nucleoid.run("arr"), [1, 2, 3, 4]);

          const actualUserList = nucleoid.run("User");
          deepEqual(actualUserList, expectedUserList);
        });
        ```

        EXPECTED BEHAVIOR:
        1. Create variables with dependencies (a, b)
        2. Create array and modify it
        3. Create class with $Class properties and rules
        4. Read all $nuc statements from datastore
        5. Clear state and replay statements
        6. Verify state is restored correctly
        7. Verify dependency tracking still works after restoration
        """
        # Step 1: Create variable dependencies
        nucleoid.run("a = 1")
        nucleoid.run("b = a + 2")
        nucleoid.run("a = 2")

        # Step 2: Create and modify array
        nucleoid.run("arr = [1, 2, 3]")
        nucleoid.run("arr.push(4)")

        # Step 3: Create class with $Class properties
        nucleoid.run(
            "class User { constructor(name, createdAt) { this.name = name; this.createdAt = createdAt } }"
        )
        nucleoid.run("$User.active = True")
        nucleoid.run("if ($User.name == 'Test') { $User.mode = 'TEST' }")

        # Note: Using a fixed timestamp for testing instead of Date.now()
        nucleoid.run("new User('Test', 1234567890)")

        # Step 4: Store expected User list
        expected_user_list = nucleoid.run("User")

        # Step 5: Read statements from datastore
        datastore_data = nucleoid.datastore.read()
        statements = []
        for stmt in datastore_data:
            if hasattr(stmt, '$') and stmt['$'] is not None:
                statements.extend(stmt['$'] if isinstance(stmt['$'], list) else [stmt['$']])

        # Filter out None/null values
        statements = [s for s in statements if s is not None]

        # Step 6: Clear state
        clear()

        # Step 7: Replay statements to restore state
        process(statements, None, {'declarative': True})

        # Step 8: Verify state is restored
        assert nucleoid.run("a") == 2
        assert nucleoid.run("b") == 4

        # Step 9: Create new variables to test dependency tracking
        nucleoid.run("c = b + 3")
        nucleoid.run("a = 3")

        # Step 10: Verify dependency tracking works after restoration
        assert nucleoid.run("b") == 5  # b should recalculate to a + 2 = 5
        assert nucleoid.run("c") == 8  # c should recalculate to b + 3 = 8

        # Step 11: Verify array is restored
        assert nucleoid.run("arr") == [1, 2, 3, 4]

        # Step 12: Verify User class and instances are restored
        actual_user_list = nucleoid.run("User")
        assert actual_user_list == expected_user_list
