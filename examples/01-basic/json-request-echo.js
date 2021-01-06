// A basic example returning the request details as JSON on STDOUT.

Deno.core.ops();
let request = Deno.core.jsonOpSync("get_request", []);
Deno.core.print(JSON.stringify(request));
