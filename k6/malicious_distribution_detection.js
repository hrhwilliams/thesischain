// Demonstrates client-side detection of a malicious server that distributes
// attacker-controlled keys when one user looks up another user's device.
//
// Setup:
//   - Backend wired with `MaliciousDeviceKeyService` (see main.rs).
//   - Lookups via `/api/user/{user_id}/device/{device_id}` return the
//     attacker keys hardcoded in services/malicious.rs.
//   - History via `/api/user/{user_id}/device/{device_id}/history`
//     passes through to the chain unchanged. In a real attack the client
//     would query the CometBFT node directly to bypass the server entirely;
//     here the history endpoint stands in for that direct query.
//
// Detection:
//   - The x25519 returned by GET device must match the latest x25519 in
//     the history. If it doesn't, the server is lying.
//
// Running:
//   - `k6 run --vus 10 --duration 30s .\malicious_distribution_detection.js`

/*
  █ TOTAL RESULTS 

    checks_total.......: 1680    54.604644/s
    checks_succeeded...: 100.00% 1680 out of 1680
    checks_failed......: 0.00%   0 out of 1680

    ✓ victim registered
    ✓ lookup 200
    ✓ history 200
    ✓ server distributed attacker x25519
    ✓ server distributed attacker ed25519
    ✓ chain history matches victim upload
    ✓ client detected tampering

    CUSTOM
    detected_attack................: 240    7.800663/s

    HTTP
    http_req_duration..............: avg=254.36ms min=47.82ms med=67.5ms max=1.32s p(90)=880.44ms p(95)=1.07s
      { expected_response:true }...: avg=254.36ms min=47.82ms med=67.5ms max=1.32s p(90)=880.44ms p(95)=1.07s
    http_req_failed................: 0.00%  0 out of 1200
    http_reqs......................: 1200   39.003317/s

    EXECUTION
    iteration_duration.............: avg=1.27s    min=1.1s    med=1.21s  max=1.61s p(90)=1.43s    p(95)=1.44s
    iterations.....................: 240    7.800663/s
    vus............................: 10     min=10        max=10
    vus_max........................: 10     min=10        max=10

    NETWORK
    data_received..................: 761 kB 25 kB/s
    data_sent......................: 208 kB 6.8 kB/s
*/

import http from 'k6/http';
import { check } from 'k6';
import { Counter } from 'k6/metrics';

const BASE = 'https://chat.fiatlux.dev';

const detectedAttack = new Counter('detected_attack');
const undetectedAttack = new Counter('undetected_attack');

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
  // 1. Victim registers, creates a device, uploads real keys.
  const victimUsername = `victim_${rand()}`;
  const password = '1234';

  const reg = http.post(
    `${BASE}/api/auth/register`,
    JSON.stringify({ username: victimUsername, password, confirm_password: password }),
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

  // 2. Attacker (or any third party) looks up the victim's device.
  // The malicious server returns substituted keys.
  const distributed = http.get(`${BASE}/api/user/${user_id}/device/${device_id}`);
  check(distributed, { 'lookup 200': (r) => r.status === 200 });
  const distributedBody = JSON.parse(distributed.body);

  // 3. Client fetches device history (in a real attack: queried from a
  // CometBFT node directly to bypass the server).
  const histRes = http.get(`${BASE}/api/user/${user_id}/device/${device_id}/history`);
  check(histRes, { 'history 200': (r) => r.status === 200 });
  const history = JSON.parse(histRes.body);
  if (!Array.isArray(history) || history.length === 0) {
    console.error('history empty — cannot perform detection');
    return;
  }

  // 4. Compare the distributed key against the latest on-chain entry.
  const latest = history[history.length - 1];

  const serverX = distributedBody.x25519;
  const serverEd = distributedBody.ed25519;
  const chainX = latest.x25519;
  const chainEd = latest.ed25519;

  console.log(`distributed x25519: ${serverX}`);
  console.log(`chain x25519:       ${chainX}`);
  console.log(`victim uploaded:    ${VICTIM_KEYS.x25519}`);

  const tampered = serverX !== chainX || serverEd !== chainEd;

  check(null, {
    'server distributed attacker x25519':  () => serverX === ATTACKER_KEYS.x25519,
    'server distributed attacker ed25519': () => serverEd === ATTACKER_KEYS.ed25519,
    'chain history matches victim upload': () => chainX === VICTIM_KEYS.x25519,
    'client detected tampering':           () => tampered,
  });

  if (tampered) {
    detectedAttack.add(1);
  } else {
    undetectedAttack.add(1);
  }
}
