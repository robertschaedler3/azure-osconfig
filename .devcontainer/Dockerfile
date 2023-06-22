FROM mcr.microsoft.com/devcontainers/cpp:0-ubuntu-20.04

ARG REINSTALL_CMAKE_VERSION_FROM_SOURCE="3.22.2"
COPY ./reinstall-cmake.sh /tmp/
RUN if [ "${REINSTALL_CMAKE_VERSION_FROM_SOURCE}" != "none" ]; then \
        chmod +x /tmp/reinstall-cmake.sh && /tmp/reinstall-cmake.sh ${REINSTALL_CMAKE_VERSION_FROM_SOURCE}; \
    fi \
    && rm -f /tmp/reinstall-cmake.sh

RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
    apt-transport-https \
    build-essential \
    ca-certificates \
    curl \
    gcovr \
    gdb \
    libcurl4-openssl-dev \
    libgmock-dev \
    libgtest-dev \
    libssl-dev \
    ninja-build \
    rapidjson-dev \
    uuid-dev \
    wget
