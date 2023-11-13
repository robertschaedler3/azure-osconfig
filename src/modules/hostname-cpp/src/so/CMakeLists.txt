# Copyright (c) Microsoft Corporation. All rights reserved.
# Licensed under the MIT License.

project(hostname_cpp)

add_library(${PROJECT_NAME} SHARED HostnameModule.cpp)
target_include_directories(${PROJECT_NAME} PUBLIC ${MODULES_INC_DIR} ${RAPIDJSON_INCLUDE_DIRS})

set_target_properties(${PROJECT_NAME}
    PROPERTIES
        POSITION_INDEPENDENT_CODE ON
    )

target_link_libraries(${PROJECT_NAME}
    PRIVATE
        commonutils
        logging
    PUBLIC
        libhostname_cpp
    )

install(TARGETS ${PROJECT_NAME} DESTINATION ${MODULES_INSTALL_DIR})
