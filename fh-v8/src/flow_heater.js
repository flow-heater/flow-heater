async function main(fh, request) {
    await fh.log(`DENO: Got request body: ${request.body}, content-type header: ${request.headers['content-type']}, method: ${request.method}`);

    // modify the requests data and return it back to the rust-context
    request.method = 'POST';
    request.body = "this is the patched body";
    request.headers['content-type'] = 'application/json';

    await fh.dispatch_request("http://httpbin.org/anything", request);
    await fh.log("Hello from Deno!");
}
