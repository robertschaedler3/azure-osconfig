// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

#ifndef MMI_H
#define MMI_H

// typedef void* MMI_HANDLE;

// Plus any error codes from errno.h
// #define MMI_OK 0

// Not null terminated, UTF-8, JSON formatted string
// typedef char* MMI_JSON_STRING;

#ifdef __cplusplus
extern "C"
{
#endif

int MmiGetInfo(
    const char* clientName,
    char** payload,
    int* payloadSizeBytes);
void* MmiOpen(
    const char* clientName,
    const unsigned int maxPayloadSizeBytes);
void MmiClose(void* clientSession);
int MmiSet(
    void* clientSession,
    const char* componentName,
    const char* objectName,
    const char* payload,
    const int payloadSizeBytes);
int MmiGet(
    void* clientSession,
    const char* componentName,
    const char* objectName,
    char** payload,
    int* payloadSizeBytes);
void MmiFree(char* payload);

#ifdef __cplusplus
}
#endif

#endif // MMI_H