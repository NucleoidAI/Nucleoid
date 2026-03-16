#include <Python.h>
#include <iostream>

int main() {
    std::cout << "Start\n";
    Py_Initialize();

    PyRun_SimpleString("print('Hello from Python')");

    Py_Finalize();
}
