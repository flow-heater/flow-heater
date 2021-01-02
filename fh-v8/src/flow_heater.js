// invoke ops
Deno.core.ops();

// run the get_request function (provided by the surrounding rust ecosystem)
let request = Deno.core.jsonOpSync("get_request", []);
Deno.core.print(`DENO: Got request body: ${request.body}, content-type header: ${request.headers['content-type']}, method: ${request.method}\n`);

// modify the requests data and return it back to the rust-context
request.method = 'POST';
request.body = "this is the patched body";
request.headers['content-type'] = 'application/json';

Deno.core.jsonOpSync("dispatch_request", request);
