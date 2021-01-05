/**
 *
 * Forward TTN payloads to Kotori for Hiveeyes.
 *
 * - https://community.hiveeyes.org/t/ttn-daten-an-kotori-weiterleiten/1422/5
 * - https://community.hiveeyes.org/t/datenweiterleitung-via-ttn-lora-zu-hiveeyes-bob-und-beep-einrichten/3197
 * - https://github.com/hiveeyes/terkin-datalogger/tree/0.10.0/client/TTN
 *
**/

// Preamble.
Deno.core.ops();
let request = Deno.core.jsonOpSync("get_request", []);

// Main code.
const input = JSON.parse(request.body);

var output = {};

output      = input.payload_fields;
output.sf   = Number(input.metadata.data_rate.split('BW')[0].substring(2));
output.bw   = Number(input.metadata.data_rate.split('BW')[1]);
output.freq = Number(input.metadata.frequency);
output.counter = Number(input.counter);
output.gtw_count = Number(input.metadata.gateways.length);

for (i = 0; i < input.metadata.gateways.length; i++) {
  output["gw_" + input.metadata.gateways[i].gtw_id + "_rssi"] = input.metadata.gateways[i].rssi;
  output["gw_" + input.metadata.gateways[i].gtw_id + "_snr"]  = input.metadata.gateways[i].snr;
}

const URL = "https://swarm.hiveeyes.org/api/" + input.dev_id.replace(/-/g, '/') + "/data";

//request.body = output;
request.body = JSON.stringify(output);

// FIXME: Need to `dispatch_request` here.
//request.forwardTo = URL;


// Epilogue.
Deno.core.print(JSON.stringify(request));
