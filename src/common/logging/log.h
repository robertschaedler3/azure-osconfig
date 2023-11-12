// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

#ifndef LOG_H
#define LOG_H

#include <string.h>
#include <time.h>

#ifdef __cplusplus
extern "C"
{
#endif

#define TIME_FORMAT_STRING_LENGTH 20

#define ARRAY_SIZE(a) (sizeof(a) / sizeof(a[0]))

char* GetFormattedTime();

#define __SHORT_FILE__ (strrchr(__FILE__, '/') ? strrchr(__FILE__, '/') + 1 : __FILE__)
#define __LOG__(fmt, level, ...) printf("[%s] [%s:%d] [%s] " fmt "\n", GetFormattedTime(), __SHORT_FILE__, __LINE__, level, ##__VA_ARGS__)

#define LOG_INFO(fmt, ...) __LOG__(fmt, "INFO", ##__VA_ARGS__)
#define LOG_ERROR(fmt, ...) __LOG__(fmt, "ERROR", ##__VA_ARGS__)
#define LOG_TRACE(fmt, ...) __LOG__(fmt, "TRACE", ##__VA_ARGS__)

#ifdef __cplusplus
}
#endif

#endif // LOG_H