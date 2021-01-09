// A very simple IIFE JavaScript module.
var modhello = (function () {

  var me = {};

  me.echo = function(input) {
    return input;
  }

  return me;
}());

if (typeof module === "object" && module.exports) {
  module.exports = modhello;
}
