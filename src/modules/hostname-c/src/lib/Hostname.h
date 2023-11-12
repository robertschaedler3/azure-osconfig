// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

#ifndef HOSTNAME_H
#define HOSTNAME_H

#include <utils.h>

typedef struct Context
{
    unsigned int maxPayloadSizeBytes;
} Context;

int GetName(char** name);
int SetName(const char* name);

#endif // HOSTNAME_H