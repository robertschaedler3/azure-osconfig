// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

#include <rapidjson/document.h>
#include <rapidjson/stringbuffer.h>
#include <rapidjson/writer.h>
#include <utils.h>

#include <Hostname.h>
#include <Mmi.h>

static const std::string g_componentName = "Hostname-CPP";
static const std::string g_reportedPropertyName = "name";
static const std::string g_desiredPropertyName = "desiredName";

static const char* g_moduleInfo = "{"
    "\"Name\": \"Hostname\","
    "\"Description\": \"Provides functionality to observe and configure network Hostname and hosts\","
    "\"Manufacturer\": \"Microsoft\","
    "\"VersionMajor\": 1,"
    "\"VersionMinor\": 0,"
    "\"VersionInfo\": \"Nickel\","
    "\"Components\": [\"Hostname-CPP\"],"
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

void* MmiOpen(
    const char* client,
    const unsigned int maxPayloadSizeBytes)
{
    void* handle = nullptr;

    (void)(maxPayloadSizeBytes);

    if (nullptr != client)
    {
        Hostname* hostname = new (std::nothrow) Hostname();

        if (nullptr == hostname)
        {
            LOG_ERROR("MmiOpen failed to allocate memory");
        }
        else
        {
            handle = reinterpret_cast<void*>(hostname);
        }
    }
    else
    {
        LOG_ERROR("MmiOpen called with nullptr client");
    }

    return handle;
}

void MmiClose(void* handle)
{
    if (nullptr != handle)
    {
        Hostname* hostname = reinterpret_cast<Hostname*>(handle);
        delete hostname;
    }
}

int MmiSet(
    void* handle,
    const char* component,
    const char* property,
    const char* payload,
    const int size)
{
    int status = 0;

    Hostname* hostname = reinterpret_cast<Hostname*>(handle);

    if ((nullptr == handle) || (nullptr == component) || (nullptr == property) || (nullptr == payload) || (0 >= size))
    {
        LOG_ERROR("MmiGet(%p, %s, %s, %p, %d) called with invalid arguments", handle, component, property, payload, size);
        return EINVAL;
    }

    rapidjson::Document document;
    document.Parse(payload, size);

    if (document.HasParseError())
    {
        LOG_ERROR("Invalid JSON payload: %s", payload);
        return EINVAL;
    }

    try
    {
        if (g_componentName == component)
        {
            if (g_desiredPropertyName == property)
            {
                std::string name = document.GetString();
                hostname->Set(name);
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
    }
    catch (const std::exception& e)
    {
        LOG_ERROR("%s", e.what());
        status = EINTR;
    }

    return status;
}

int MmiGet(
    void* handle,
    const char* component,
    const char* property,
    char** payload,
    int* size)
{
    int status = 0;
    Hostname* hostname = reinterpret_cast<Hostname*>(handle);

    if ((nullptr == handle) || (nullptr == component) || (nullptr == property) || (nullptr == payload) || (nullptr == size))
    {
        LOG_ERROR("MmiGet(%p, %s, %s, %p, %p) called with invalid arguments", handle, component, property, payload, size);
        return EINVAL;
    }

    rapidjson::StringBuffer buffer;
    rapidjson::Writer<rapidjson::StringBuffer> writer(buffer);

    try
    {
        if (g_componentName == component)
        {
            if (g_reportedPropertyName == property)
            {
                std::string name = hostname->Get();
                writer.String(name.c_str());
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
    }
    catch (const std::exception& e)
    {
        LOG_ERROR("%s", e.what());
        status = EINTR;
    }

    if (0 == status)
    {
        *size = buffer.GetSize();
        *payload = new (std::nothrow) char[*size];

        if (*payload)
        {
            memcpy(*payload, buffer.GetString(), *size);
        }
        else
        {
            LOG_ERROR("MmiGet failed to allocate memory for payload string");
            status = ENOMEM;
        }
    }

    return status;
}

void MmiFree(char* payload)
{
    if (!payload)
    {
        return;
    }
    delete[] payload;
}