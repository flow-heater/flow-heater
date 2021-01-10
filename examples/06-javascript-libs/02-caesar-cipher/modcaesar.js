/**
 *
 * An IIFE JavaScript module implementing Caesar's cipher.
 *
 * - https://en.wikipedia.org/wiki/Caesar_cipher
 * - https://codereview.stackexchange.com/questions/132125/rot13-javascript/252691#252691
 *
**/
const modcaesar = (function () {

  const this_ = {

    caesar: function(string, shift) {

      // Alphabet
      const alphabet = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ';

      // Encoded Text
      let encodedText = '';

      // Adjust Shift (Over 26 Characters)
      if (shift > 26) {
        // Assign Remainder As Shift
        shift = shift % 26;
      }

      // Iterate Over Data
      let i = 0;
      while (i < string.length) {

        let character = (string[i]).toUpperCase();

        // Valid Alphabet Characters
        if (alphabet.indexOf(character) !== -1) {
          // Find Alphabet Index
          const alphabetIndex = alphabet.indexOf(character);

          // Alphabet Index Is In Alphabet Range
          if (alphabet[alphabetIndex + shift]) {
            // Append To String
            encodedText += alphabet[alphabetIndex + shift];
          }
          // Alphabet Index Out Of Range (Adjust Alphabet By 26 Characters)
          else {
            // Append To String
            encodedText += alphabet[alphabetIndex + shift - 26];
          }
        }
        // Special Characters
        else {
          // Append To String
          encodedText += string[i];

        }

        // Increase I
        i++;
      }

      return encodedText;
    },

    rot12_encode: function(string) {
      return this_.caesar(string, 12);
    },

    rot12_decode: function(string) {
      return this_.caesar(string, 14);
    }

  };

  return this_;

}());

if (typeof module === "object" && module.exports) {
  module.exports = modcaesar;
}
