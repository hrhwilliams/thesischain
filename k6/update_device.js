import http from 'k6/http';
import { check } from 'k6';
import { Trend } from 'k6/metrics';

const BASE = 'https://chat.fiatlux.dev';
const uploadDuration = new Trend('upload_device_ms', true);

const DEVICE_KEYS = {
  x25519: "gzpLJ9mEPG3hz5zBpz4jlcRlMkS6gW0p093Kl+5hJRo",
  ed25519: "lQVAT2liS82yLLrlSzO8LBfXkffRvhKcFMOi4zkw9JM",
  signature: "LPxZkj+a44bkqWJQAwfWatbKV6iQ4lqL6DEV6+B9kdPV1A1PNKC8QNliO24l+9fAwLRz9iDJQegrvWNnzMZZCw",
};

const DEVICE_KEYS_2 = {
  x25519: "kU6sga8vcY25x2ML3Y31mjtjmZEAL9i8fDOtbb1c1xg",
  ed25519: "huJuHz7Y70Uay06AKlrAf5sLnNUk/XAv710TFH7Et3A",
  signature: "fGceS7RH1KRg9UgqCOzf/LeBKrgjuXZrO332r2Q4LJ10k63QJ6OR1UUIwgrHQel6wAA7urqvTWcBXcR1jAFkBw",
  authorization: "9VbUmEsAMU1mHcAMb4SBpOTJ9Zgk84f0+5M214SKiLbXrL6mBc3sOI6/aArVq5SzGPMK3hRiWi98JaHEYti6Ag",
};

export default function () {
  const username = `vu_${Math.random().toString(36)}`;
  const password = '1234';

  http.post(`${BASE}/api/auth/register`,
    JSON.stringify({ username, password, confirm_password: password }),
    { headers: { 'Content-Type': 'application/json' } }
  );

  const newDevice = http.post(`${BASE}/api/me/device`);
  let device_id;
  try {
    device_id = JSON.parse(newDevice.body).device_id;
  } catch (_) {
    console.error(`VU ${__VU} iter ${__ITER}: failed to create device: ${newDevice.body}`);
    return;
  }

  const start = Date.now();
  const upload = http.put(
    `${BASE}/api/me/device`,
    JSON.stringify({ device_id, ...DEVICE_KEYS }),
    { headers: { 'Content-Type': 'application/json' } }
  );
  uploadDuration.add(Date.now() - start);

  check(upload, { 'put device 200': (r) => r.status === 200 });
  if (upload.status !== 200) {
    console.error(`VU ${__VU} iter ${__ITER}: ${upload.body}`);
  }

  const updated_keys = {
    ed25519: DEVICE_KEYS_2.ed25519,
    x25519: DEVICE_KEYS_2.x25519,
    signature: DEVICE_KEYS_2.signature,
    authorization: {
      authorizing_device_id: device_id,
      signature: DEVICE_KEYS_2.authorization,
    },
  };

  const start2 = Date.now();
  const upload2 = http.put(
    `${BASE}/api/me/device`,
    JSON.stringify({ device_id, ...updated_keys }),
    { headers: { 'Content-Type': 'application/json' } }
  );
  uploadDuration.add(Date.now() - start2);

  check(upload2, { 'update device 200': (r) => r.status === 200 });
  if (upload2.status !== 200) {
    console.error(`VU ${__VU} iter ${__ITER}: ${upload2.body}`);
  }
}
