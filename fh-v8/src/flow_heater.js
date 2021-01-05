

async function dispatch_request(url, request) {
    const spec = {
        "url": url,
        "request": request
    };

    await Deno.core.jsonOpAsync("dispatch_request", spec);
}

async function main() {
    Deno.core.ops();

    // run the get_request function (provided by the surrounding rust ecosystem)
    let request = Deno.core.jsonOpSync("get_request", []);
    Deno.core.print(`DENO: Got request body: ${request.body}, content-type header: ${request.headers['content-type']}, method: ${request.method}\n`);

    // modify the requests data and return it back to the rust-context
    request.method = 'POST';
    request.body = "this is the patched body";
    request.headers['content-type'] = 'application/json';

    await dispatch_request("http://httpbin.org/anything", request);
    await dispatch_request("http://httpbin.org/anything", request);
}

main();