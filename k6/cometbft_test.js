import http from 'k6/http';
import { check } from 'k6';
import { SharedArray } from 'k6/data';
import { Trend } from 'k6/metrics';

const BASE = 'https://chat.fiatlux.dev';

const uploadDuration = new Trend('upload_device_ms', true);
const getDeviceDuration = new Trend('get_device_ms', true);
const historyDuration = new Trend('get_device_history_ms', true);

const DEVICE_KEYS = {
  ed25519: "PbCMOYxMo6vTYC9uGiGcqHpJIIpjWOtvWG2sVSX+TKg",
  x25519:  "WS2NBb75L0Gcw96kNGfJgh56aPrYxRgjJG3hOifEQ04",
  signature: "NPA6Z/4omyLxqz3SACx/NDqifqutUxyCwcL0KBAdCUm4SfZ7vvZ1p2nU6o5VTC9afUUqFhTLPEXZfx8GllLGAA",
};

const data = new SharedArray('user_devices', function () {
  return JSON.parse(open('./user_devices.json'));
});

let hasSession = false;

export const options = {
  discardResponseBodies: true,
  scenarios: {
    sustained_load: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '30m', target: 100 },
        { duration: '23h', target: 100 },
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
  check(upload, { 'put device 200': (r) => r.status === 200 });
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
    if (!ids) return;
    uploadKeys(ids.device_id);
    return;
  }

  const target = data[Math.floor(Math.random() * data.length)];

  if (prob >= 0.10 && prob < 0.90) {
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