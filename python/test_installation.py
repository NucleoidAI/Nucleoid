#!/usr/bin/env python3
"""Test script to verify Nucleoid AI installation."""

def test_imports():
    """Test all main imports."""
    try:
        from nucleoidai import start, run, register, create_app, Config
        from nucleoidai import datastore, openapi, test
        print("[PASS] All main imports successful")
        return True
    except ImportError as e:
        print(f"[FAIL] Import failed: {e}")
        return False

def test_basic_functionality():
    """Test basic functionality."""
    try:
        from nucleoidai import start, run
        
        # Start runtime in test mode
        start({"test": True, "cache": True})
        print("[PASS] Runtime started successfully")
        
        # Test running a simple statement
        result = run("x = 42")
        print(f"[PASS] Statement execution successful: {result}")
        
        return True
    except Exception as e:
        print(f"[FAIL] Basic functionality test failed: {e}")
        return False

def test_web_app():
    """Test web application creation."""
    try:
        from nucleoidai import create_app
        app = create_app()
        
        if app:
            print("[PASS] Web application created successfully")
            return True
        else:
            print("[FAIL] Web application creation failed")
            return False
    except Exception as e:
        print(f"[FAIL] Web app test failed: {e}")
        return False

def main():
    """Run all tests."""
    print("Testing Nucleoid AI Python Installation...")
    print("=" * 50)
    
    tests = [
        ("Import Tests", test_imports),
        ("Basic Functionality", test_basic_functionality), 
        ("Web Application", test_web_app),
    ]
    
    passed = 0
    total = len(tests)
    
    for test_name, test_func in tests:
        print(f"\n{test_name}:")
        try:
            if test_func():
                passed += 1
            else:
                print(f"  [FAIL] {test_name} failed")
        except Exception as e:
            print(f"  [FAIL] {test_name} failed with exception: {e}")
    
    print("\n" + "=" * 50)
    print(f"Tests completed: {passed}/{total} passed")
    
    if passed == total:
        print("SUCCESS: All tests passed! Nucleoid AI is ready to use.")
        return 0
    else:
        print("WARNING: Some tests failed. Please check the installation.")
        return 1

if __name__ == "__main__":
    import sys
    sys.exit(main())