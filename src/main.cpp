#include <Python.h>
#include <iostream>

int main() {
    std::cout << "`nuc` runtime is started" << std::endl;

    const char *pythonHome = std::getenv("PYTHON_HOME");
    std::wstring wPythonHome(pythonHome, pythonHome + strlen(pythonHome));
    Py_SetPythonHome(wPythonHome.data());

    Py_Initialize();

    PyRun_SimpleString("import sys; print(f'Python {sys.version}')");

    Py_Finalize();

    return 0;
}
