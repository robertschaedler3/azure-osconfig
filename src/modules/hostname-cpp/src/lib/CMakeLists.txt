# Copyright (c) Microsoft Corporation. All rights reserved.
# Licensed under the MIT License.

project(libhostname_cpp)

add_library(${PROJECT_NAME} STATIC Hostname.cpp)
set_property(TARGET ${PROJECT_NAME} PROPERTY POSITION_INDEPENDENT_CODE ON)

target_link_libraries(${PROJECT_NAME} PRIVATE logging commonutils)
target_include_directories(${PROJECT_NAME}
    PUBLIC
        ${MODULES_INC_DIR}
        ${CMAKE_CURRENT_SOURCE_DIR}
    )