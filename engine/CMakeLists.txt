cmake_minimum_required(VERSION 3.20)
project(jseek_engine CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(re2 CONFIG QUIET)
if(NOT re2_FOUND)
  find_package(PkgConfig REQUIRED)
  pkg_check_modules(RE2 REQUIRED IMPORTED_TARGET re2)
  add_library(re2::re2 ALIAS PkgConfig::RE2)
endif()
find_package(Threads REQUIRED)

file(GLOB ENGINE_SRCS CONFIGURE_DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/Core/*.cpp)
if(NOT APPLE)
  # macOS-only modules (FSEvents watcher, ServiceEngine) compile to empty units elsewhere;
  # HttpServer and InstanceLock use POSIX sockets/flock and are skipped on Windows.
  if(WIN32)
    list(FILTER ENGINE_SRCS EXCLUDE REGEX "(HttpServer|InstanceLock)\\.cpp$")
  endif()
endif()

add_library(jseek_engine STATIC ${ENGINE_SRCS})
target_include_directories(jseek_engine PUBLIC ${CMAKE_CURRENT_SOURCE_DIR}/Core ${CMAKE_CURRENT_SOURCE_DIR})
target_link_libraries(jseek_engine PUBLIC re2::re2 Threads::Threads)
if(APPLE)
  target_link_libraries(jseek_engine PUBLIC "-framework CoreServices")
endif()
if(MSVC)
  target_compile_options(jseek_engine PUBLIC /utf-8 /permissive- /Zc:__cplusplus /EHsc)
  target_compile_definitions(jseek_engine PUBLIC NOMINMAX WIN32_LEAN_AND_MEAN _CRT_SECURE_NO_WARNINGS)
endif()

add_executable(test_all test_all.cpp)
target_link_libraries(test_all PRIVATE jseek_engine)
