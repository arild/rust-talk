cmake_minimum_required(VERSION 3.16)
project(parcel-api-c C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_INTERPROCEDURAL_OPTIMIZATION TRUE)

# yyjson — pinned, fetched at configure time. Small (~10K LOC); compiles in
# under a second so it's not worth wrestling with distro packaging.
include(FetchContent)
FetchContent_Declare(
    yyjson
    GIT_REPOSITORY https://github.com/ibireme/yyjson.git
    GIT_TAG 0.10.0
)
FetchContent_MakeAvailable(yyjson)

# h2o (evloop variant) from the system. Provided by `libh2o-evloop-dev` on
# Debian. We look for it via pkg-config first, then fall back to a manual
# search so the build still works on hosts where the .pc file isn't installed.
find_package(PkgConfig)
if(PkgConfig_FOUND)
    pkg_check_modules(H2O libh2o-evloop)
endif()
if(NOT H2O_FOUND)
    find_path(H2O_INCLUDE_DIRS NAMES h2o.h)
    find_library(H2O_LIBRARIES NAMES h2o-evloop h2o)
    if(NOT H2O_INCLUDE_DIRS OR NOT H2O_LIBRARIES)
        message(FATAL_ERROR "h2o not found — install libh2o-evloop-dev")
    endif()
endif()

find_package(OpenSSL REQUIRED)
find_package(Threads REQUIRED)
find_library(ZLIB_LIB NAMES z)

add_executable(parcel-api
    src/main.c
    src/service.c
    src/features.c
)

target_include_directories(parcel-api PRIVATE ${H2O_INCLUDE_DIRS})
target_link_libraries(parcel-api PRIVATE
    yyjson
    ${H2O_LIBRARIES}
    OpenSSL::SSL OpenSSL::Crypto
    ${ZLIB_LIB}
    Threads::Threads
    m
)
# Tell h2o.h to use its own evloop instead of pulling in libuv. The
# libh2o-evloop pkg-config file ships empty Cflags so we set this here
# unconditionally.
target_compile_definitions(parcel-api PRIVATE H2O_USE_LIBUV=0)
if(H2O_CFLAGS_OTHER)
    target_compile_options(parcel-api PRIVATE ${H2O_CFLAGS_OTHER})
endif()

# User asked for "max optimize": -O3, link-time optimization, native ISA
# (the docker image runs on the same host that built it, so `-march=native`
# is safe). `-fno-plt` saves a load per extern call once the loader has
# resolved everything.
target_compile_options(parcel-api PRIVATE
    -O3 -march=native -flto -fno-plt -fomit-frame-pointer
    -Wall -Wextra -Wno-unused-parameter -Wno-missing-field-initializers
)
target_link_options(parcel-api PRIVATE -flto)

install(TARGETS parcel-api RUNTIME DESTINATION bin)
