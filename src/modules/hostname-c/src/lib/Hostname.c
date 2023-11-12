// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

#include "Hostname.h"

int GetName(char** name)
{
    const char* command = "cat /etc/hostname";

    int status = 0;
    char* buffer = NULL;

    if (0 == (status = Exec(command, &buffer, NULL)))
    {
        if (buffer)
        {
            *name = strdup(buffer);
            free(buffer);
        }
        else
        {
            LOG_ERROR("Failed to get the hostname");
            status = ENOENT;
        }
    }
    else
    {
        LOG_ERROR("Failed to execute %s", command);
        status = ENOENT;
    }

    return status;
}

int SetName(const char* name)
{
    const char* template = "hostnamectl set-hostname --static \"%s\"";

    int status = 0;
    char* command = NULL;

    if (NULL != (command = (char*)malloc(strlen(template) + strlen(name) + 1)))
    {
        sprintf(command, template, name);

        if (0 != Exec(command, NULL, NULL))
        {
            LOG_ERROR("Failed to set the hostname: '%s'", name);
            status = ENOENT;
        }
    }
    else
    {
        LOG_ERROR("Failed to allocate memory for command");
        status = ENOMEM;
    }

    return status;
}
