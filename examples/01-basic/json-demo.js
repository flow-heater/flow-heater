// A basic example demonstrating `JSON.stringify()` and `JSON.parse()`.

const data = { "a": "b" };
const stringified = JSON.stringify(data);
const matches = (obj, source) =>
    Object.keys(source).every(key => obj.hasOwnProperty(key) && obj[key] === source[key]);

await fh.log(`Stringify: ${stringified}\n`);

if (matches(JSON.parse(stringified), data)) {
    await fh.log(`Parse works, too\n`);
} else {
    await fh.log(`PvD: ${JSON.parse(stringified)} v ${data}\n`);
}
