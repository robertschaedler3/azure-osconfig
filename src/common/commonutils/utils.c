// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

#include <ctype.h>

#include "CommonUtils.h"
#include "utils.h"

int Trim(char* str, char** outbuf)
{
    int status = 0;
    char* start = str;
    char* end = str + strlen(str) - 1;

    if (NULL == str)
    {
        LOG_ERROR("Invalid argument: str is NULL");
        return EINVAL;
    }

    if (NULL == outbuf)
    {
        LOG_ERROR("Invalid argument: outbuf is NULL");
        return EINVAL;
    }

    *outbuf = NULL;

    while (isspace(*start))
    {
        start++;
    }

    while (isspace(*end))
    {
        end--;
    }

    size_t len = end - start + 1;

    if (len > 0)
    {
        if (NULL != (*outbuf = (char*)malloc(len + 1)))
        {
            memcpy(*outbuf, start, len);
            (*outbuf)[len] = '\0';
        }
        else
        {
            LOG_ERROR("Failed to allocate memory for trimmed string");
            status = ENOMEM;
        }
    }
    else
    {
        *outbuf = NULL;
    }

    return status;
}

int Exec(const char* command, char** outbuf, char** errbuf)
{
    UNUSED(errbuf);

    char* buf = NULL;

    int status = ExecuteCommand(NULL, command, true, true, 0, 0, &buf, NULL, NULL);

    if (0 == status)
    {
        if (NULL != outbuf && NULL != buf)
        {
            status = Trim(buf, outbuf);
        }
        else
        {
            free(buf);
        }
    }

    return status;
}
