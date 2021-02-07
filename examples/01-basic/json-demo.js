// A basic example demonstrating `JSON.stringify()` and `JSON.parse()`.

const data = { "a": "b" };
const stringified = JSON.stringify(data);
const matches = (obj, source) =>
    Object.keys(source).every(key => obj.hasOwnProperty(key) && obj[key] === source[key]);

Deno.core.print(`Stringify: ${stringified}\n`);

if (matches(JSON.parse(stringified), data)) {
    Deno.core.print(`Parse works, too\n`);
} else {
    Deno.core.print(`PvD: ${JSON.parse(stringified)} v ${data}\n`);
}
