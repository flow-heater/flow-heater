async function dispatch_request(url, request) {
    const spec = {
        "url": url,
        "request": request
    };

    await Deno.core.jsonOpAsync("dispatch_request", spec);
}

async function prelude() {
    Deno.core.ops();

    // run the get_request function (provided by the surrounding rust ecosystem)
    let request = Deno.core.jsonOpSync("get_request", []);
    await main(request);
}

async function main(request) {
    // Dummy implementation ... to be overwritten by the user
}
