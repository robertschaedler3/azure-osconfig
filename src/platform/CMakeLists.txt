# Copyright (c) Microsoft Corporation. All rights reserved.
# Licensed under the MIT License.

corrosion_import_crate(MANIFEST_PATH "Cargo.toml")

install(IMPORTED_RUNTIME_ARTIFACTS osconfig-platform RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR})
install(FILES daemon/osconfig-platform.service DESTINATION ${CMAKE_INSTALL_SYSCONFDIR}/systemd/system)