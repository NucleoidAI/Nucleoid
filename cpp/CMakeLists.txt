cmake_minimum_required(VERSION 3.20)
project(nucleoid VERSION 1.0.0 LANGUAGES C CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

include(FetchContent)

# ── nlohmann/json ──────────────────────────────────────────────────────────────
FetchContent_Declare(
    nlohmann_json
    GIT_REPOSITORY https://github.com/nlohmann/json.git
    GIT_TAG        v3.11.3
    GIT_SHALLOW    TRUE
)
FetchContent_MakeAvailable(nlohmann_json)

# ── cpp-httplib ────────────────────────────────────────────────────────────────
FetchContent_Declare(
    httplib
    GIT_REPOSITORY https://github.com/yhirose/cpp-httplib.git
    GIT_TAG        v0.15.3
    GIT_SHALLOW    TRUE
)
FetchContent_MakeAvailable(httplib)

# ── QuickJS ────────────────────────────────────────────────────────────────────
FetchContent_Declare(
    quickjs
    GIT_REPOSITORY https://github.com/bellard/quickjs.git
    GIT_TAG        master
    GIT_SHALLOW    TRUE
)
FetchContent_GetProperties(quickjs)
if(NOT quickjs_POPULATED)
    FetchContent_Populate(quickjs)
endif()

# libbf.c was merged into quickjs.c in newer releases — collect what exists.
set(QJS_SOURCES
    ${quickjs_SOURCE_DIR}/quickjs.c
    ${quickjs_SOURCE_DIR}/libregexp.c
    ${quickjs_SOURCE_DIR}/libunicode.c
    ${quickjs_SOURCE_DIR}/cutils.c
)
foreach(opt libbf.c dtoa.c)
    if(EXISTS "${quickjs_SOURCE_DIR}/${opt}")
        list(APPEND QJS_SOURCES "${quickjs_SOURCE_DIR}/${opt}")
    endif()
endforeach()

add_library(quickjs STATIC ${QJS_SOURCES})
# Keep the quickjs source dir PRIVATE so its bare "version" file never
# enters the C++ include search path (it would shadow <version>).
target_include_directories(quickjs PRIVATE ${quickjs_SOURCE_DIR})
target_compile_definitions(quickjs PRIVATE _GNU_SOURCE CONFIG_VERSION="2025-09-13")
if(NOT MSVC)
    target_compile_options(quickjs PRIVATE -w -Wno-error)
endif()

# Copy quickjs.h into our own include tree so C++ code can reach it
# without exposing the entire quickjs source directory.
file(MAKE_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}/include/third_party")
configure_file(
    "${quickjs_SOURCE_DIR}/quickjs.h"
    "${CMAKE_CURRENT_SOURCE_DIR}/include/third_party/quickjs.h"
    COPYONLY
)

# ── Nucleoid core library ──────────────────────────────────────────────────────
file(GLOB_RECURSE NUCLEOID_SOURCES
    "src/*.cpp"
    "src/nuc/*.cpp"
    "src/arc/*.cpp"
    "src/server/*.cpp"
    "src/server/routes/*.cpp"
)

add_library(nucleoid_lib STATIC ${NUCLEOID_SOURCES})
target_include_directories(nucleoid_lib PUBLIC include)
target_link_libraries(nucleoid_lib
    PUBLIC  quickjs nlohmann_json::nlohmann_json
    PRIVATE httplib::httplib
)

# ── CLI executable ─────────────────────────────────────────────────────────────
add_executable(nucleoid main.cpp)
target_link_libraries(nucleoid PRIVATE nucleoid_lib httplib::httplib)
target_include_directories(nucleoid PRIVATE include)

install(TARGETS nucleoid RUNTIME DESTINATION bin)
