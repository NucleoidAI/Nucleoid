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

    Total tests: 1
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


# ============================================================================
# SUMMARY: $nuc Statement Storage Test
# ============================================================================
"""
✅ COMPLETE $NUC STORAGE TEST COVERAGE

From typescript/src/test/$nuc.spec.ts (lines 16-55):

1. ✅ test_stores_nuc (line 16)
   - Tests that all statement types can be stored in datastore
   - Verifies variable dependencies, arrays, classes, $Class properties
   - Tests state restoration from datastore
   - Verifies dependency tracking works after restoration

TOTAL: 1 test (100% coverage of $nuc storage tests)

────────────────────────────────────────────────────────────────────────────

WHAT IS $NUC STORAGE?

$nuc statements are special statements that are persisted to the datastore.
This allows the runtime to:
1. Store the complete program state
2. Replay statements to restore state after restart
3. Maintain dependency tracking across restarts
4. Persist business logic and data

TYPES OF STATEMENTS STORED:
✓ Variable assignments (a = 1)
✓ Dependency expressions (b = a + 2)
✓ Array operations (arr.push(4))
✓ Class declarations (class User {...})
✓ Class-level properties ($User.active = true)
✓ Class-level rules (if ($User.name === 'Test') {...})
✓ Object instantiation (new User(...))

────────────────────────────────────────────────────────────────────────────

DATASTORE WORKFLOW:

┌──────────────────────────────────────────────────────────────────────────┐
│ 1. EXECUTE STATEMENTS                                                    │
│    nucleoid.run("a = 1")                                                │
│    nucleoid.run("b = a + 2")                                            │
│    → Statements executed and stored in datastore                        │
├──────────────────────────────────────────────────────────────────────────┤
│ 2. READ FROM DATASTORE                                                   │
│    statements = nucleoid.datastore.read()                               │
│    → Retrieve all stored $nuc statements                               │
├──────────────────────────────────────────────────────────────────────────┤
│ 3. CLEAR STATE                                                           │
│    clear()                                                              │
│    → Remove all variables and dependencies                              │
├──────────────────────────────────────────────────────────────────────────┤
│ 4. REPLAY STATEMENTS                                                     │
│    process(statements, None, {'declarative': True})                     │
│    → Re-execute statements to restore state                             │
├──────────────────────────────────────────────────────────────────────────┤
│ 5. VERIFY RESTORATION                                                    │
│    assert nucleoid.run("a") == 2                                        │
│    assert nucleoid.run("b") == 4                                        │
│    → State and dependencies fully restored                              │
└──────────────────────────────────────────────────────────────────────────┘

────────────────────────────────────────────────────────────────────────────

USE CASES:

1. Persistence: Save program state to disk
2. Hot Reload: Restore state after code changes
3. Testing: Replay scenarios from stored statements
4. Debugging: Reproduce issues by replaying statements
5. Migration: Transfer state between environments

────────────────────────────────────────────────────────────────────────────
"""
