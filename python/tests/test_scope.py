"""
Tests for Scope class variable retrieval.

This test suite corresponds to the TypeScript test file:
typescript/src/test/scope.spec.ts

The Scope class handles variable lookup across nested scopes with
proper traversal through prior scopes.
"""

import pytest
from nucleoid.ast.__Identifier__ import Identifier
from nucleoid.scope import Scope


class TestScope:
    """
    Test suite for Scope class variable retrieval.

    This class contains all tests from the 'Scope' describe block
    in the TypeScript test suite.

    Total tests: 3
    """

    # ========================================================================
    # SCOPE TEST 1 of 3: Local Variable Retrieval
    # TypeScript: lines 6-19
    # ========================================================================

    def test_retrieves_local_variable(self):
        """
        Test retrieving a local variable from the current scope.

        Original TypeScript (lines 6-19):
        ```typescript
        it("retrieves local variable", () => {
          const scope = new Scope(null, {});

          const variable = new Identifier("a");

          scope.graph.a = {};
          const identifier = scope.retrieve(variable);

          if (identifier === null) {
            throw new Error("Identifier is null");
          }

          equal(identifier.toString(), "scope.local.a");
        });
        ```

        EXPECTED BEHAVIOR:
        1. Create a scope with no prior scope
        2. Add variable 'a' to scope's graph
        3. Retrieve identifier for variable 'a'
        4. Identifier path should be "scope.local.a"
        """
        # Step 1: Create scope with no prior scope
        scope = Scope(prior=None, local={})

        # Step 2: Create variable identifier
        variable = Identifier("a")

        # Step 3: Add variable to scope's graph
        scope.graph['a'] = {}

        # Step 4: Retrieve identifier
        identifier = scope.retrieve(variable)

        # Step 5: Verify identifier is not None
        assert identifier is not None, "Identifier should not be None"

        # Step 6: Verify identifier path
        assert str(identifier) == "scope.local.a"

    # ========================================================================
    # SCOPE TEST 2 of 3: Prior Scope Variable Retrieval
    # TypeScript: lines 21-34
    # ========================================================================

    def test_retrieves_local_variable_from_prior_scope(self):
        """
        Test retrieving a variable from a prior scope (grandparent).

        Original TypeScript (lines 21-34):
        ```typescript
        it("retrieves local variable from prior scope", () => {
          const scope = new Scope(null, {});
          const scope2 = new Scope(scope, {});
          const scope3 = new Scope(scope2, {});

          const variable = new Identifier("a");

          scope.graph.a = {};
          const identifier = scope3.retrieve(variable);
          if (identifier === null) {
            throw new Error("Identifier is null");
          }
          equal(identifier.toString(), "scope.prior.prior.local.a");
        });
        ```

        EXPECTED BEHAVIOR:
        1. Create nested scopes: scope1 → scope2 → scope3
        2. Add variable 'a' to scope1's graph
        3. Retrieve from scope3 (should traverse back through scope2 to scope1)
        4. Identifier path should be "scope.prior.prior.local.a"
        """
        # Step 1: Create nested scopes
        scope1 = Scope(prior=None, local={})
        scope2 = Scope(prior=scope1, local={})
        scope3 = Scope(prior=scope2, local={})

        # Step 2: Create variable identifier
        variable = Identifier("a")

        # Step 3: Add variable to scope1's graph (grandparent scope)
        scope1.graph['a'] = {}

        # Step 4: Retrieve from scope3 (should look back to scope1)
        identifier = scope3.retrieve(variable)

        # Step 5: Verify identifier is not None
        assert identifier is not None, "Identifier should not be None"

        # Step 6: Verify identifier path includes two 'prior' traversals
        assert str(identifier) == "scope.prior.prior.local.a"

    # ========================================================================
    # SCOPE TEST 3 of 3: Property Path Retrieval from Prior Scope
    # TypeScript: lines 36-50
    # ========================================================================

    def test_retrieves_local_property_from_prior_scope(self):
        """
        Test retrieving a property path from a prior scope.

        Original TypeScript (lines 36-50):
        ```typescript
        it("retrieves local property from prior scope", () => {
          const scope1 = new Scope(null, {});
          const scope2 = new Scope(scope1, {});
          const scope3 = new Scope(scope2, {});

          scope1.graph.a = {};
          const variable = new Identifier("a.b.c");
          const identifier = scope3.retrieve(variable);

          if (identifier === null) {
            throw new Error("Identifier is null");
          }

          equal(identifier.toString(), "scope.prior.prior.local.a.b.c");
        });
        ```

        EXPECTED BEHAVIOR:
        1. Create nested scopes: scope1 → scope2 → scope3
        2. Add variable 'a' to scope1's graph
        3. Retrieve property path 'a.b.c' from scope3
        4. Identifier path should be "scope.prior.prior.local.a.b.c"

        NOTE: This tests that property paths are preserved during scope traversal.
        The variable 'a' is in scope1, but we're accessing 'a.b.c' from scope3.
        """
        # Step 1: Create nested scopes
        scope1 = Scope(prior=None, local={})
        scope2 = Scope(prior=scope1, local={})
        scope3 = Scope(prior=scope2, local={})

        # Step 2: Add variable to scope1's graph
        scope1.graph['a'] = {}

        # Step 3: Create property path identifier
        variable = Identifier("a.b.c")

        # Step 4: Retrieve from scope3
        identifier = scope3.retrieve(variable)

        # Step 5: Verify identifier is not None
        assert identifier is not None, "Identifier should not be None"

        # Step 6: Verify full property path is preserved
        assert str(identifier) == "scope.prior.prior.local.a.b.c"


# ============================================================================
# SUMMARY: Scope Variable Retrieval Tests
# ============================================================================
"""
✅ COMPLETE SCOPE TEST COVERAGE

From typescript/src/test/scope.spec.ts (lines 5-51):

1. ✅ test_retrieves_local_variable (line 6)
   - Tests retrieval from current scope
   - Path: scope.local.a

2. ✅ test_retrieves_local_variable_from_prior_scope (line 21)
   - Tests retrieval from grandparent scope
   - Path: scope.prior.prior.local.a

3. ✅ test_retrieves_local_property_from_prior_scope (line 36)
   - Tests property path retrieval from grandparent scope
   - Path: scope.prior.prior.local.a.b.c

TOTAL: 3 tests (100% coverage of Scope tests)

────────────────────────────────────────────────────────────────────────────

SCOPE TRAVERSAL MECHANISM:

The Scope class implements a linked-list-like structure for nested scopes.
Each scope has:
- graph: Variables defined in this scope
- local: Local variables dictionary
- prior: Reference to parent scope (or None for root)

VARIABLE LOOKUP ALGORITHM:

┌──────────────────────────────────────────────────────────────────────────┐
│ When retrieving variable 'a' from scope3:                               │
│                                                                          │
│ 1. Check scope3.graph for 'a'                                          │
│    → Not found, move to prior scope                                    │
│                                                                          │
│ 2. Check scope2.graph for 'a'                                          │
│    → Not found, move to prior scope                                    │
│                                                                          │
│ 3. Check scope1.graph for 'a'                                          │
│    → Found! Build path: scope.prior.prior.local.a                      │
└──────────────────────────────────────────────────────────────────────────┘

IDENTIFIER PATH FORMAT:

scope.prior.prior.local.a.b.c
│     │     │     │     └─┴─┴─ Property path (a.b.c)
│     │     │     └────────── Found in local graph
│     └─────┴────────────── Two scope traversals (grandparent)
└──────────────────────────── Current scope reference

────────────────────────────────────────────────────────────────────────────

EXAMPLE USAGE:

```python
# Create nested scopes (like nested function calls)
global_scope = Scope(prior=None, local={})
function_scope = Scope(prior=global_scope, local={})
inner_scope = Scope(prior=function_scope, local={})

# Define variable in global scope
global_scope.graph['x'] = {'value': 10}

# Look up from inner scope
variable = Identifier('x')
identifier = inner_scope.retrieve(variable)

# Result: "scope.prior.prior.local.x"
#         └─ Found 2 scopes up (in global_scope)
```

────────────────────────────────────────────────────────────────────────────

USE CASES:

1. Closure Implementation: Inner functions access outer variables
2. Block Scoping: Variables in nested blocks
3. Function Parameters: Parameter lookup in nested calls
4. Module System: Module-level vs global variables
5. Class Hierarchies: Method lookup through inheritance chain

────────────────────────────────────────────────────────────────────────────
"""
