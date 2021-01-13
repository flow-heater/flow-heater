async function main(fh, request) {
    await fh.log(`DENO: Got request body: ${request.body}, content-type header: ${request.headers}, method: ${request.method}`);

    // modify the requests data and return it back to the rust-context
    request.method = 'POST';
    request.body = "this is the patched body";
    request.headers['content-type'] = ['application/json'];

    await fh.dispatch_request("http://httpbin.org/anything", request);
    await fh.log("Hello from Deno!");

    await fh.respond_with({
        code: 204,
        headers: {
            "content-type": ["application/xml"],
        },
        body: "xxx",
        version: "HTTP/1.1",
    })
}
