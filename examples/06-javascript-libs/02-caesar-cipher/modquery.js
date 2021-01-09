/**
 *
 * An IIFE JavaScript module implementing querystring encode/decode functions.
 *
 * Currently, the machinery does not support Deno's Standard Library.
 * Thus, it's needed to supply respective polyfills.
 *
 * - https://github.com/denoland/deno/blob/master/std/node/querystring.ts
 * - https://stackoverflow.com/questions/901115/how-can-i-get-query-string-values-in-javascript/2880929#2880929
 *
**/
const modquery = (function () {

  const this_ = {

    parse: function(query) {
      var match,
        pl     = /\+/g,  // Regex for replacing addition symbol with a space
        search = /([^&=]+)=?([^&]*)/g,
        decode = function (s) { return decodeURIComponent(s.replace(pl, " ")); },

      urlParams = {};
      while (match = search.exec(query))
        urlParams[decode(match[1])] = decode(match[2]);

      return urlParams;
    },

  };

  return this_;

}());

if (typeof module === "object" && module.exports) {
  module.exports = modquery;
}
