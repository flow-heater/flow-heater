class Fh {
    constructor() {
        Deno.core.ops();
    };

    async log(data) {
        // if (typeof data !== 'string') {
        // TODO: error handling
        // }

        // wrap everything so we can unpack it in rust in a structured way
        const spec = {
            data: data
        };

        await Deno.core.jsonOpAsync("fh_log", spec);
        Deno.core.print(`${data}\n`);
    };

    async dispatch_request(url, request) {
        // wrap everything so we can unpack it in rust
        const spec = {
            "url": url,
            "request": request
        };

        return await Deno.core.jsonOpAsync("dispatch_request", spec);
    };

    get_request() {
        return Deno.core.jsonOpSync("get_request", []);
    };
}

async function prelude() {
    let fh = new Fh();
    await main(fh, fh.get_request());
}

async function main(fh, request) {
    // Dummy implementation ... to be overwritten by the user
}
