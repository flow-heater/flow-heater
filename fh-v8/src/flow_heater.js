// invoke ops
Deno.core.ops();

// run the get_request function (provided by the surrounding rust ecosystem)
let request = Deno.core.jsonOpSync("get_request", []);
Deno.core.print(`Got request body: ${request.body}, headers: ${request.headers}, method: ${request.method}\n`);

// modify the requests method
request.method = 'POST';
// request.body = "this is the patched body";
// request.headers['Content-Type'] = 'application/json';

Deno.core.jsonOpSync("dispatch_request", request);
