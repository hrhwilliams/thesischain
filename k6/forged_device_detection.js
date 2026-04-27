// Demonstrates client-side detection of a malicious server that forges
// extra device entries (or rotates device keys) on the chain without a
// valid authorization signature from a previously trusted device.
//
// Setup:
//   - Backend wired with `ForgingDeviceKeyService` (see main.rs).
//   - Each victim key upload triggers an extra forged device add. The
//     forged add carries attacker keys but no `authorization` field.
//
// Detection rule:
//   - The first device a user enrolls has no authorization (nothing to
//     sign with yet). Every subsequent first-add for a device must carry
//     an authorization signed by a prior valid device's ed25519. If a
//     client walks every device's history and finds more than one
//     unauthorized key_add, the directory has been tampered with.
//
// Running:
//   - `k6 run --vus 10 --duration 30s forged_device_detection.js`
/*
  █ TOTAL RESULTS 

    checks_total.......: 720     23.598135/s
    checks_succeeded...: 100.00% 720 out of 720
    checks_failed......: 0.00%   0 out of 720

    ✓ victim registered
    ✓ list devices 200
    ✓ forged device present
    ✓ attacker key on chain
    ✓ multiple unauthorized first-adds
    ✓ client detected forgery

    CUSTOM
    detected_forgery...............: 120    3.933023/s

    HTTP
    http_req_duration..............: avg=420.73ms min=44.04ms med=64.08ms max=2.42s p(90)=2.12s p(95)=2.16s
      { expected_response:true }...: avg=420.73ms min=44.04ms med=64.08ms max=2.42s p(90)=2.12s p(95)=2.16s
    http_req_failed................: 0.00%  0 out of 720
    http_reqs......................: 720    23.598135/s

    EXECUTION
    iteration_duration.............: avg=2.53s    min=2.21s   med=2.52s   max=2.8s  p(90)=2.72s p(95)=2.75s
    iterations.....................: 120    3.933023/s
    vus............................: 10     min=10       max=10
    vus_max........................: 10     min=10       max=10

    NETWORK
    data_received..................: 511 kB 17 kB/s
    data_sent......................: 124 kB 4.1 kB/s
*/

import http from 'k6/http';
import { check } from 'k6';
import { Counter } from 'k6/metrics';

const BASE = 'https://chat.fiatlux.dev';

const detectedForgery = new Counter('detected_forgery');
const undetectedForgery = new Counter('undetected_forgery');

const VICTIM_KEYS = {
  ed25519: "PbCMOYxMo6vTYC9uGiGcqHpJIIpjWOtvWG2sVSX+TKg",
  x25519: "WS2NBb75L0Gcw96kNGfJgh56aPrYxRgjJG3hOifEQ04",
  signature: "NPA6Z/4omyLxqz3SACx/NDqifqutUxyCwcL0KBAdCUm4SfZ7vvZ1p2nU6o5VTC9afUUqFhTLPEXZfx8GllLGAA",
};

const ATTACKER_KEYS = {
  x25519: "IeyxjOtLNBl9EZe9f0T/i9LBthJp1HicM1Sjd0/Lw3w",
  ed25519: "8dvhUrR02tiWTf5IKQTD2l0pXPm6Ja/+Bzbnvrub468",
  signature: "heo2mtH9lguiG0EqHaR6FPeUvgKmNVoHKuEUkVFS88a9SMB7vyr2RbUZLNxQ2wiwUs+hkus7qGyLbUCizDLaBQ",
}

export const options = {
  vus: 1,
  iterations: 1,
};

function rand() {
  return Math.random().toString(36).slice(2);
}

export default function () {
  // 1. Victim registers and uploads keys for their first device. The
  //    forging server piggybacks an extra forged device on this upload.
  const username = `victim_${rand()}`;
  const password = '1234';

  const reg = http.post(
    `${BASE}/api/auth/register`,
    JSON.stringify({ username, password, confirm_password: password }),
    { headers: { 'Content-Type': 'application/json' } },
  );
  check(reg, { 'victim registered': (r) => r.status === 200 || r.status === 201 });

  const created = http.post(`${BASE}/api/me/device`);
  let device_id, user_id;
  try {
    ({ device_id, user_id } = JSON.parse(created.body));
  } catch (_) {
    console.error(`failed to create device: ${created.status} ${created.body}`);
    return;
  }

  const put = http.put(
    `${BASE}/api/me/device`,
    JSON.stringify({ device_id, ...VICTIM_KEYS }),
    { headers: { 'Content-Type': 'application/json' } },
  );
  if (put.status !== 200) {
    console.error(`upload keys failed: ${put.status} ${put.body}`);
    return;
  }

  // 2. Client walks every device the directory claims for this user.
  const all = http.get(`${BASE}/api/user/${user_id}/devices`);
  check(all, { 'list devices 200': (r) => r.status === 200 });
  const devices = JSON.parse(all.body);
  console.log(`directory reports ${devices.length} device(s) for victim`);

  // 3. For each device, fetch history. Count first key_add events that
  //    lack an authorization signature.
  let unauthorizedAdds = 0;
  let attackerKeySeen = false;

  for (const d of devices) {
    const did = d.device_id;
    const hRes = http.get(`${BASE}/api/user/${user_id}/device/${did}/history`);
    if (hRes.status !== 200) {
      console.error(`history fetch failed for ${did}: ${hRes.status}`);
      continue;
    }
    const history = JSON.parse(hRes.body);
    if (!Array.isArray(history) || history.length === 0) continue;

    history.sort((a, b) => a.chain_height - b.chain_height);
    const first = history[0];

    if (!first.authorization) {
      unauthorizedAdds += 1;
      console.log(`device ${did} first add at height ${first.chain_height} has NO authorization`);
    } else {
      console.log(`device ${did} first add at height ${first.chain_height} authorized by ${first.authorization.authorizing_device_id}`);
    }

    if (first.x25519 === ATTACKER_KEYS.x25519) {
      attackerKeySeen = true;
      console.log(`device ${did} carries attacker x25519`);
    }
  }

  // 4. Detection: more than one unauthorized first-add means the server
  //    enrolled a device without proof of consent from a prior device.
  const tampered = unauthorizedAdds > 1;

  check(null, {
    'forged device present': () => devices.length > 1,
    'attacker key on chain': () => attackerKeySeen,
    'multiple unauthorized first-adds': () => unauthorizedAdds > 1,
    'client detected forgery': () => tampered,
  });

  if (tampered) {
    detectedForgery.add(1);
  } else {
    undetectedForgery.add(1);
  }
}
