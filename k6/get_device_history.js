import http from 'k6/http';
import { check } from 'k6';
import { Trend } from 'k6/metrics';

const BASE = 'https://chat.fiatlux.dev';
const historyDuration = new Trend('get_device_history_ms', true);

const DEVICE_KEYS = {
  ed25519: "PbCMOYxMo6vTYC9uGiGcqHpJIIpjWOtvWG2sVSX+TKg",
  x25519:  "WS2NBb75L0Gcw96kNGfJgh56aPrYxRgjJG3hOifEQ04",
  signature: "NPA6Z/4omyLxqz3SACx/NDqifqutUxyCwcL0KBAdCUm4SfZ7vvZ1p2nU6o5VTC9afUUqFhTLPEXZfx8GllLGAA",
};

export default function () {
  const username = `vu_${Math.random().toString(36)}`;
  const password = '1234';

  http.post(`${BASE}/api/auth/register`,
    JSON.stringify({ username, password, confirm_password: password }),
    { headers: { 'Content-Type': 'application/json' } }
  );

  const newDevice = http.post(`${BASE}/api/me/device`);
  let device_id, user_id;
  try {
    ({ device_id, user_id } = JSON.parse(newDevice.body));
  } catch (_) {
    console.error(`VU ${__VU} iter ${__ITER}: failed to create device: ${newDevice.body}`);
    return;
  }

  const put = http.put(`${BASE}/api/me/device`,
    JSON.stringify({ device_id, ...DEVICE_KEYS }),
    { headers: { 'Content-Type': 'application/json' } }
  );
  if (put.status !== 200) {
    console.error(`VU ${__VU} iter ${__ITER}: PUT failed: ${put.body}`);
    return;
  }

  const start = Date.now();
  const history = http.get(`${BASE}/api/user/${user_id}/device/${device_id}/history`);
  historyDuration.add(Date.now() - start);

  check(history, {
    'get history 200': (r) => r.status === 200,
    'has entries':     (r) => JSON.parse(r.body).length > 0,
  });
  if (history.status !== 200) {
    console.error(`VU ${__VU} iter ${__ITER}: ${history.body}`);
  }
}
