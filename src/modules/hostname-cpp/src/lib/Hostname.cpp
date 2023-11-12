// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

#include "Hostname.h"

std::string Exec(const char* cmd) {
    char buffer[128] = { 0 };

    std::string result;
    FILE* pipe = popen(cmd, "r");

    if (!pipe) throw std::runtime_error("popen() failed!");

    try {
        while (fgets(buffer, sizeof buffer, pipe) != NULL) {
            result += buffer;
        }
    }
    catch (...) {
        pclose(pipe);
        throw;
    }

    pclose(pipe);

    const char* space = " \t\v\r\n";
    std::size_t start = result.find_first_not_of(space);
    std::size_t end = result.find_last_not_of(space);

    return start == end ? std::string() : result.substr(start, end - start + 1);
}

std::string Hostname::Get()
{
    const char* command = "cat /etc/hostname";
    return Exec(command);
}

void Hostname::Set(const std::string name)
{
    const char* command = "hostnamectl set-hostname ";
    std::string cmd = command + name;
    Exec(cmd.c_str());
}
