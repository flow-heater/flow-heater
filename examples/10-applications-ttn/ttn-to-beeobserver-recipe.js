/**
 *
 * Forward TTN payloads to BEEP for Bee Observer (BOB).
 *
 * - https://community.hiveeyes.org/t/ttn-daten-an-kotori-weiterleiten/1422/5
 * - https://community.hiveeyes.org/t/datenweiterleitung-via-ttn-lora-zu-hiveeyes-bob-und-beep-einrichten/3197
 * - https://github.com/hiveeyes/terkin-datalogger/tree/0.10.0/client/TTN
 *
**/
// ------------------------------------
// B O B
// https://bee-observer.org/api/sensors
// ------------------------------------

// Main code.
const input = JSON.parse(request.body);
const payload_ttn = input.payload_fields;

var output_bob = {};

// Those are randomized accounts. No worries.
switch (input.dev_id) {
    case "hiveeyes-testdrive-site42-hive1":
        output_bob.key = "SikUrjAkOjLyThoa";
        break;
    case "hiveeyes-testdrive-site42-hive2":
        output_bob.key = "okMiKiTwawCauc";
        break;
}

for (var key in payload_ttn) {
    if (payload_ttn.hasOwnProperty(key)) {
        if (/load/.test(key)) {
            output_bob.weight_kg = payload_ttn[key];
        } else if (/temperature_5/.test(key)) {
            output_bob.t = payload_ttn[key];
        } else if (/relative_humidity_5/.test(key)) {
            output_bob.h = payload_ttn[key];
        } else if (/barometric_pressure_5/.test(key)) {
            output_bob.p = payload_ttn[key];
        } else if (/voltage_0/.test(key)) {
            output_bob.bv = payload_ttn[key];
        } else if (/temperature_1/.test(key)) {
            i = parseInt(key.split("_")[1], 10);
            output_bob["t_i_" + (i - 9)] = payload_ttn[key];
        }
    }
}

output_bob.rssi = input.metadata.gateways[0].rssi;

for (i = 1; i < input.metadata.gateways.length; i++) {
    if (input.metadata.gateways[i].rssi > output_bob.rssi) {
        output_bob.rssi = input.metadata.gateways[i].rssi;
    }
}

//response.body = output_bob;
//request.body = output_bob;
request.body = JSON.stringify(output_bob);

// FIXME: Need to `dispatch_request` here.
//request.forwardTo = 'https://bee-observer.org/api/sensors';


// Epilogue.
await fh.log(JSON.stringify(request));
