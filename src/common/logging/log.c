// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

#include "log.h"

char* GetFormattedTime()
{
    static char g_logTime[TIME_FORMAT_STRING_LENGTH] = { 0 };
    time_t rawTime = { 0 };
    struct tm* timeInfo = NULL;
    time(&rawTime);
    timeInfo = localtime(&rawTime);
    strftime(g_logTime, ARRAY_SIZE(g_logTime), "%Y-%m-%d %H:%M:%S", timeInfo);
    return g_logTime;
}