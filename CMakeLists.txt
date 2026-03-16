cmake_minimum_required(VERSION 4.1)
project(nuc)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_SOURCE_DIR}/bin)

add_executable(nuc src/main.cpp)

find_package(Python3 REQUIRED COMPONENTS Development)

target_link_libraries(nuc PRIVATE Python3::Python)
