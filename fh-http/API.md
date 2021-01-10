# API spec for fh-http

## RequestProcessor Object

```json5
{
    "id": "<uuid>",             // optional: is generated on `POST`
    "name": "<string>",         // name / descriptor, has no detailed meaning
    "language": "<string>",     // one of js or ts
    "runtime": "<string>",      // one of wasm or v8
    "code": "<string>"          // full code blob to execute
}
```

## RequestConversation Object
```json5
{
    "id": "<uuid>",                         // `RequestConversation` UUID
    "created_at": "<string>",               // date in RFC3339 (e.g. 2021-01-09T23:45:48.562721Z)
    "request_processor_id": "<uuid>",       // `RequestProcessor` UUID
    "audit_items": [                        // chronologically sorted list of `AuditItem`s
        // `AuditItem` Objects ...
    ],             
}
```

## AuditItem Object
```json5
{
    "kind": "<string>",             // indicates, which of which kind this object is: "request", "response", "log"
    "id": "<uuid>",                 // `AuditItem` UUID
    "created_at": "string",         // date in RFC3339 (e.g. 2021-01-09T23:45:48.562721Z)
    "conversation_id": "<uuid>",    // `RequestConversation` UUID
    "payload": "<string>|object",   // actual payload of the item, depends on the items `kind`-field
    "inc": 0,                       // only for kind `request`: counter indicating in which order the requests were issued
    "request": "<uuid>",            // only for kind `response`: the request UUID, for which the response was returned
}
```

## Public endpoints
**Run Request Processor**

*Runs a previously stored request processor*

- Request: `GET|POST|PUT|PATCH|DELETE|... /processor/{processor_id}/run`
- Response: TBD

**Run Request Processor With FH prelude**

*Runs a previously stored request processor, but wraps the given code in the prelude, which provides methods like `fh.log()` and `fh.dispatch_request()`*

- Request: `GET|POST|PUT|PATCH|DELETE|... /processor/{processor_id}/run_with_prelude`
- Response: TBD

**Get Request Conversation**

*Fetches information for an existing request processor*

- Request: `GET /conversation/{conversation_id}`
- Response: `RequestConversation` Object

**Get Request Conversation Audit Items**

*Fetches information for an existing request processor*

- Request: `GET /conversation/{conversation_id}/audit_item`
- Response: List of `AuditItem` Object

## Admin endpoints

### Authentication
- TODO
### Endpoints
**Create Request Processor**

*Creates a new request processor and returns the object including a newly created UUID*

- Request: `POST /admin/processor`
    JSON Request body:
    ```json
    {
        "name": "string",
        "language": "js",
        "runtime": "v8",
        "code": "code goes here..."
    }
    ```

- Response:

    JSON Response body:
    ```json
    {
        "id": "<uuid>",
        "name": "string",
        "language": "js",
        "runtime": "v8",
        "code": "code goes here..."
    }
    ```

**Get Request Processor**

*Fetches information for an existing request processor*

- Request: `GET /admin/processor/{processor_id}`
- Response: `RequestProcessor` Object

**Update Request Processor**

*Updates an existing request processor, requires all object properties to be present*

- Request: `PUT /admin/processor/{processor_id}`

    JSON Request body:
    ```json
    {
        "name": "string",
        "language": "js",
        "runtime": "v8",
        "code": "code goes here..."
    }
    ```

- Response:

    JSON Response body:
    ```json
    {
        "id": "<uuid>",
        "name": "string",
        "language": "js",
        "runtime": "v8",
        "code": "code goes here..."
    }
    ```

**Delete Request Processor**

*Deletes an existing request processor*

- Request: `DELETE /admin/processor/{processor_id}`
- Response: ... no content
