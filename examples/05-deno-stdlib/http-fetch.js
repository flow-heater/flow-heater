// deno run --allow-net examples/05-deno-stdlib/http-fetch.js

Deno.core.ops();

const url = "https://example.org";
const promise = fetch(url);
console.log(promise);

const response = await promise;
console.log(response);

const body = new Uint8Array(await response.arrayBuffer());
Deno.stdout.write(body);
