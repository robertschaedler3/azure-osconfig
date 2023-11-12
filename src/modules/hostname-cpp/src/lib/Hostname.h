// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

#include <stdexcept>
#include <string>

class Hostname
{
public:
    std::string Get();
    void Set(const std::string hostName);
};
