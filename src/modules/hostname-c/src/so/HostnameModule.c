// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

#include <parson.h>
#include <utils.h>

#include <Hostname.h>
#include <Mmi.h>

static const char* g_component = "Hostname-C";
static const char* g_reportedPropertyName = "name";
static const char* g_desiredPropertyName = "desiredName";

static const char* g_moduleInfo = "{"
    "\"Name\": \"Hostname\","
    "\"Description\": \"Provides functionality to observe and configure network Hostname and hosts\","
    "\"Manufacturer\": \"Microsoft\","
    "\"VersionMajor\": 1,"
    "\"VersionMinor\": 0,"
    "\"VersionInfo\": \"Nickel\","
    "\"Components\": [\"Hostname-C\"],"
    "\"Lifetime\": 2,"
    "\"UserAccount\": 0"
"}";

int MmiGetInfo(const char* client, char** payload, int* size)
{
    int status = 0;

    (void)(client);

    if ((NULL == payload) || (NULL == size))
    {
        return EINVAL;
    }

    *payload = NULL;
    *size = (int)strlen(g_moduleInfo);

    *payload = (char*)malloc(*size);

    if (*payload)
    {
        memcpy(*payload, g_moduleInfo, *size);
    }
    else
    {
        LOG_ERROR("MmiGetInfo: failed to allocate %d bytes", *size);
        *size = 0;
        status = ENOMEM;
    }

    return status;
}

void* MmiOpen(const char* client, const unsigned int maxPayloadSizeBytes)
{
    Context* handle = NULL;

    if (NULL == client)
    {
        LOG_ERROR("MmiOpen() called with NULL client");
        return NULL;
    }

    handle = (Context*)malloc(sizeof(Context));

    if (handle)
    {
        handle->maxPayloadSizeBytes = maxPayloadSizeBytes;
    }
    else
    {
        LOG_ERROR("MmiOpen() failed to allocate memory for handle");
    }

    return (void*)handle;
}

void MmiClose(void* handle)
{
    Context* context = (Context*)handle;

    if (NULL == context)
    {
        LOG_ERROR("MmiClose() called with NULL handle");
        return;
    }

    free(context);
}

int MmiSet(void* handle, const char* component, const char* property, const char* payload, const int size)
{
    int status = 0;
    JSON_Value* value = NULL;
    char* json = NULL;

    if ((NULL == handle) || (NULL == component) || (NULL == property) || (NULL == payload))
    {
        LOG_ERROR("MmiGet(%p, %s, %s, %p, %d) called with invalid arguments", handle, component, property, payload, size);
        return EINVAL;
    }

    if (NULL != (json = (char*)malloc(size + 1)))
    {
        memcpy(json, payload, size);
        json[size] = '\0';

        if (NULL == (value = json_parse_string(json)))
        {
            LOG_ERROR("Failed to parse JSON object");
            status = EINVAL;
        }
    }
    else
    {
        LOG_ERROR("Failed to allocate memory for payload");
        status = ENOMEM;
    }

    if ((status == 0) && (0 == strcmp(component, g_component)))
    {
        if (0 == strcmp(property, g_desiredPropertyName))
        {
            if (json_value_get_type(value) == JSONString)
            {
                const char* name = json_value_get_string(value);

                if (name)
                {
                    status = SetName(name);
                }
                else
                {
                    LOG_ERROR("MmiSet: failed to get name from JSON object");
                    status = EINVAL;
                }
            }
            else
            {
                LOG_ERROR("MmiSet: JSON object is not a string");
                status = EINVAL;
            }
        }
        else
        {
            LOG_ERROR("MmiSet called for an invalid object name (%s)", property);
            status = EINVAL;
        }
    }
    else
    {
        LOG_ERROR("MmiSet called for an invalid component name (%s)", component);
        status = EINVAL;
    }

    return status;
}

int MmiGet(void* handle, const char* component, const char* property, char** payload, int* size)
{
    int status = 0;
    Context* context = (Context*)handle;
    JSON_Value* value = NULL;
    char* name = NULL;
    char* json = NULL;

    if ((NULL == handle) || (NULL == component) || (NULL == property) || (NULL == payload) || (NULL == size))
    {
        LOG_ERROR("MmiGet(%p, %s, %s, %p, %p) called with invalid arguments", handle, component, property, payload, size);
        return EINVAL;
    }

    *payload = NULL;
    *size = 0;

    if (0 == strcmp(component, g_component))
    {
        if (0 == strcmp(property, g_reportedPropertyName))
        {
            status = GetName(&name);

            if ((0 == status) && (NULL != name))
            {
                value = json_value_init_string(name);

                if (NULL == value)
                {
                    LOG_ERROR("Failed to create JSON value");
                    status = ENOMEM;
                }
            }
        }
        else
        {
            LOG_ERROR("MmiGet called for an invalid object name (%s)", property);
            status = EINVAL;
        }
    }
    else
    {
        LOG_ERROR("MmiGet called for an invalid component name (%s)", component);
        status = EINVAL;
    }

    if ((0 == status) && (NULL != value))
    {
        json = json_serialize_to_string(value);

        if (NULL == json)
        {
            LOG_ERROR("Failed to serialize JSON object");
            status = ENOMEM;
        }
        else if ((context->maxPayloadSizeBytes > 0) && (context->maxPayloadSizeBytes < strlen(json)))
        {
            LOG_ERROR("Payload size exceeds maximum size");
            status = E2BIG;
        }
        else
        {
            *size = strlen(json);
            *payload = (char*)malloc(*size);

            if (NULL == *payload)
            {
                LOG_ERROR("Failed to allocate memory for payload");
                status = ENOMEM;
            }
            else
            {
                memcpy(*payload, json, *size);
            }
        }
    }

    return status;
}

void MmiFree(char* value)
{
    if (value)
    {
        free(value);
    }
}
