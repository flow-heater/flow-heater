/**
 *
 * A rot12 encoder/decoder example using an IIFE JavaScript
 * module implementing Caesar's cipher.
 *
 * Also, it uses a pure JavaScript ``querystring.parse()`` function.
 *
 * This demonstrates the non-standard ``@fh:include``
 * directive to include external JavaScript code.
 *
 * Currently, the machinery does not support Deno's Standard Library.
 * Thus, its "import" directive is not available.
 *
**/

// @fh:include("./modquery.js")
// @fh:include("./modcaesar.js")

// Decode "x-www-form-urlencoded" form data.
// https://morioh.com/p/480aef8e92cd
let data = modquery.parse(request.body);

// Use "payload" field;
let payload = data.payload;

// Apply rot12 encoding/decoding to payload content.
var encoded = modcaesar.rot12_encode(payload);
Deno.core.print("encoded: " + encoded + "\n");

var decoded = modcaesar.rot12_decode(encoded);
Deno.core.print("decoded: " + decoded + "\n");
