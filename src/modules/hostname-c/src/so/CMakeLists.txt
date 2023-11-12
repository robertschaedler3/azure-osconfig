# Copyright (c) Microsoft Corporation. All rights reserved.
# Licensed under the MIT License.

project(hostname_c)

add_library(${PROJECT_NAME} SHARED HostnameModule.c)
target_include_directories(${PROJECT_NAME} PUBLIC ${MODULES_INC_DIR})

set_target_properties(${PROJECT_NAME}
    PROPERTIES
        PREFIX ""
        POSITION_INDEPENDENT_CODE ON
    )

target_link_libraries(${PROJECT_NAME}
    PRIVATE
        commonutils
        logging
        parsonlib
    PUBLIC
        libhostname_c
    )

install(TARGETS ${PROJECT_NAME} DESTINATION ${MODULES_INSTALL_DIR})
