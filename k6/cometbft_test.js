import http from 'k6/http';
import { check, sleep } from 'k6';
import { SharedArray } from 'k6/data';
import { Trend } from 'k6/metrics';

const BASE = 'https://chat.fiatlux.dev';

const uploadDuration = new Trend('upload_device_ms', true);
const updateDuration = new Trend('update_device_ms', true);
const getDeviceDuration = new Trend('get_device_ms', true);
const historyDuration = new Trend('get_device_history_ms', true);

// It can't actually update these keys lol
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

const data = new SharedArray('user_devices', function () {
  return JSON.parse(open('./user_devices.json'));
});

let hasSession = false;

// Because the device keys are hard-coded, a device can only be updated once.
const updatableDeviceIds = [];

export const options = {
  discardResponseBodies: true,
  scenarios: {
    sustained_load: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '1m', target: 150 },
        { duration: '23h', target: 150 },
        { duration: '30m', target: 0 },
      ],
    },
  },
};

function setupUserAndDevice() {
  const username = `vu_${Math.random().toString(36)}`;
  const password = '1234';

  http.post(`${BASE}/api/auth/register`,
    JSON.stringify({ username, password, confirm_password: password }),
    { headers: { 'Content-Type': 'application/json' } }
  );

  const newDevice = http.post(`${BASE}/api/me/device`, null, { responseType: 'text' });
  
  try {
    return JSON.parse(newDevice.body);
  } catch (_) {
    return null;
  }
}

function uploadKeys(device_id) {
  const start = Date.now();
  const upload = http.put(
    `${BASE}/api/me/device`,
    JSON.stringify({ device_id, ...DEVICE_KEYS }),
    { headers: { 'Content-Type': 'application/json' } }
  );
  uploadDuration.add(Date.now() - start);
  const ok = check(upload, { 'put device 200': (r) => r.status === 200 });
  if (ok) {
    updatableDeviceIds.push(device_id);
  }
}

function updateKeys(device_id) {
  const body = {
    device_id,
    ed25519: DEVICE_KEYS_2.ed25519,
    x25519: DEVICE_KEYS_2.x25519,
    signature: DEVICE_KEYS_2.signature,
    authorization: {
      authorizing_device_id: device_id,
      signature: DEVICE_KEYS_2.authorization,
    },
  };
  const start = Date.now();
  const update = http.put(
    `${BASE}/api/me/device`,
    JSON.stringify(body),
    { headers: { 'Content-Type': 'application/json' } }
  );
  updateDuration.add(Date.now() - start);
  check(update, { 'update device 200': (r) => r.status === 200 });
}

export default function () {
if (!hasSession) {
    const username = `reader_${__VU}_${Math.random().toString(36)}`;
    const password = '1234';
    http.post(`${BASE}/api/auth/register`,
      JSON.stringify({ username, password, confirm_password: password }),
      { headers: { 'Content-Type': 'application/json' } }
    );
    hasSession = true;
  }

  const prob = Math.random();

  if (prob < 0.10) {
    const ids = setupUserAndDevice();
    if (ids) uploadKeys(ids.device_id);
  } else {
    const target = data[Math.floor(Math.random() * data.length)];

    if (prob < 0.90) {
      const start = Date.now();
      const device = http.get(`${BASE}/api/user/usr_${target.user_id}/device/dev_${target.device_id}`, { responseType: 'text' });
      getDeviceDuration.add(Date.now() - start);

      check(device, {
        'get device 200': (r) => r.status === 200,
        'has x25519':     (r) => r.status === 200 && !!JSON.parse(r.body).x25519,
      });
    } else {
      const start = Date.now();
      const history = http.get(`${BASE}/api/user/usr_${target.user_id}/device/dev_${target.device_id}/history`, { responseType: 'text' });
      historyDuration.add(Date.now() - start);

      check(history, {
        'get history 200': (r) => r.status === 200,
        'has entries':     (r) => r.status === 200 && JSON.parse(r.body).length > 0,
      });
    }
  }

  sleep(0.2);
}