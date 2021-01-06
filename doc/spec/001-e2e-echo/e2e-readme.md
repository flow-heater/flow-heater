# Flow Heater e2e testing specification


## Introduction
As outlined within https://github.com/flow-heater/fh-core/pull/3#issuecomment-753533297:

> The machinery will have to be improved in upcoming iterations to have this
> actually make sense. For example, we a) need to invoke specific Javascript 
> files and b) return their transformation outcome using a special echo mode.

On this matter, it would be nice to be able to invoke the machinery by e.g. saying
```bash
cargo run --bin fh-http --echo --recipe=examples/scenario-01.js
```
in order to designate it should forward incoming requests to the recipe loaded 
ad hoc from `examples/scenario-01.js` and echo the transformation outcome to 
its HTTP response to be able to test it.

That would be a kind of special isolated testing mode independently from the 
"production mode" wired to the storage subsystem you are planning, where the 
machinery would load recipes from a database and would forward the request 
to another site (#1).

The "echo" feature might work similar to what [httpbin](https://httpbin.org/) 
is offering. Some examples based on [HTTPie](https://httpie.io/) to get an 
idea how its response format looks like:
```bash
http POST https://httpbin.org/post foo=bar
http POST https://httpbin.org/post?foo=bar baz==qux Content-Type:abc
```


## Elaboration
We concluded that it would be nice to echo/return the content of 
a) multiple requests created through multiple invocations of `dispatch_request` and
b) a single response through the invocation of `respond_with`.

Apart from being able to invoke the machinery from the command line 
as outlined above, these mechanics might also well be implemented 
through the real HTTP API, where just a single special parameter 
would have to be attached as query argument like `?__fh-echo__=true`.


## Specification

### Magic request parameter 

A full example would look like this:
```
http POST 'http://flow-heater.example.org/p/{processor_id}/run?__fh-echo__=true' foo=bar baz==qux
```

We used the "dunder" (double underscore) notation known from Python magic methods
for designating this parameter in order to reduce the chance of collision with
regular request parameters as we figured just using `echo` here might trigger
that case more often than not.

Alternatively, that magic parameter might as well be implemented
as a special request header like `FH-Echo: true`.


### Response payload
We included three files `e2e-echo-full.json5`, `e2e-echo-request-binary.json5`
and `e2e-echo-response-binary.json5` which outline appropriate JSON payloads 
how things might look like on the response side.

The `request` snippets have been derived from responses to HTTPBin's
`/anything` endpoint.

As a bonus, each object might optionally attach a `timestamp` attribute in
RFC3339 format.

#### Encoding binary data bodies
We concluded that the `body` attribute of the `response` object should be able
to also carry responses in binary/non-ASCII format independently of the original 
`Content-Type` or `Content-Encoding` response headers in order to be able to 
serialize binary contents into the JSON file.

`e2e-echo-response-binary.json5` gives an example for that. It aligns with
`e2e-echo-request-binary.json5`, both use a prefix like
`data:application/octet-stream;base64,...`, thus applying the [data URI scheme]. 

[data URI scheme]: https://en.wikipedia.org/wiki/Data_URI_scheme

#### Notes
While we wrote those specification files in JSON5 format for being able
to include inline comments, the real implementation will probably use
plain JSON.
