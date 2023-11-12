# Copyright (c) Microsoft Corporation. All rights reserved.
# Licensed under the MIT License.

project(hostname_rust)

corrosion_import_crate(MANIFEST_PATH "Cargo.toml")

# # Copy the module shared-object to build/modules/bin for convenience
# set(TARGET_OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/lib${PROJECT_NAME}.so)
# set(TARGET_OUTPUT_COPY ${MODULES_BUILD_BIN_DIR}/${PROJecT_NAME}.so)

# add_custom_command(
#     OUTPUT ${TARGET_OUTPUT}
#     POST_BUILD
#     COMMAND ${CMAKE_COMMAND} -E copy ${TARGET_OUTPUT} ${TARGET_OUTPUT_COPY}
#     COMMENT "Copying ${PROJECT_NAME} to ${TARGET_OUTPUT_COPY}"
# )