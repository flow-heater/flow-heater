/**
 *
 * A basic example using an IIFE JavaScript module.
 *
 * This demonstrates the non-standard ``@fh:include``
 * directive to include external JavaScript code.
 *
 * Currently, the machinery does not support Deno's Standard Library.
 * Thus, its "import" directive is not available.
 *
**/

// @fh:include("./modhello.js")

var output = modhello.echo("Hello world.");
Deno.core.print(output);
