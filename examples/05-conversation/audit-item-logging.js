// A basic example fh.log() functionality

async function main(fh, request) {
    await fh.log("Hello, World");
    await fh.log(`Body is: ${request.body}`);
}
