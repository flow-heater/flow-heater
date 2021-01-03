# API spec for fh-http

## Authentication
- TODO

## Admin endpoints

### Request Processor Object

```json
{
    "id": "<uuid>",             // optional: is generated on `POST`
    "name": "string",           // name / descriptor, has no detailed meaning
    "language": "js",           // one of js or ts
    "runtime": "v8",            // one of wasm or v8
    "code": "code goes here..." //full code blob to execute
}
```

### Endpoints
**Get Request Processor**

*Fetches information for an existing request processor*

- Request: `GET /admin/processor/{processor_id}`
- Response: Request Processor Object

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