/**
 *
 * An example using an IIFE JavaScript module for parsing HTML.
 *
 * It will include the "Pure JavaScript XML (pjxml)" library.
 * https://github.com/smeans/pjxml
 *
 * This demonstrates the non-standard ``@fh:include``
 * directive to include external JavaScript code.
 *
 * Currently, the machinery does not support Deno's Standard Library.
 * Thus, its "import" directive is not available.
 *
**/

// Include library from filesystem (inactive).
/* // @fh:include("./lib/pjxml.js") */

// Include library from the Web.
// @fh:include("https://raw.githubusercontent.com/smeans/pjxml/4ea9516d/js/pjxml.js")

var xml = '<document attribute="value"><name>David Bowie</name></document>';
var doc = pjXML.parse(xml);
await fh.log(JSON.stringify(doc));
