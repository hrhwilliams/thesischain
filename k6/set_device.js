import http from 'k6/http';
import { check } from 'k6';
import { Trend } from 'k6/metrics';

const BASE = 'https://chat.fiatlux.dev';
const uploadDuration = new Trend('upload_device_ms', true);

const DEVICE_KEYS = {
  x25519: "LR4dZunFFF69UCugXHL3/zNTJXDDmDfVttl5ORXfkXA",
  ed25519: "93Vusp00BP+WIQErG4R98Qcy1Tv5yy/3yhfmx1wLtAI",
  signature: "kLGPLMish6immKlr6ETukGIAGWAXW1jiBb/qEi+N/ITtHtpXJEgxDLX3zY/R9E0sswfLJq7fAZAz3wPq9GLzDA",
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
}
