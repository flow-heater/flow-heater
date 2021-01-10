// A basic example fh.dispatch_request() functionality

async function main(fh, request) {
    await fh.dispatch_request("http://httpbin.org/anything", request);
}
