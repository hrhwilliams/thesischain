import http from 'k6/http';
import { check, sleep } from 'k6';
import { SharedArray } from 'k6/data';
import { Trend } from 'k6/metrics';

const BASE = 'https://chat.fiatlux.dev';

const uploadDuration = new Trend('upload_device_ms', true);
const updateDuration = new Trend('update_device_ms', true);
const getDeviceDuration = new Trend('get_device_ms', true);
const historyDuration = new Trend('get_device_history_ms', true);

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
        { duration: '1m', target: 150 },
        { duration: '23h', target: 150 },
        { duration: '30m', target: 0 },
      ],
    },
  },
};

export default function () {
  const target = data[Math.floor(Math.random() * data.length)];
  const start = Date.now();

  const device = http.get(`${BASE}/api/user/usr_${target.user_id}/device/dev_${target.device_id}`, { responseType: 'text' });

  getDeviceDuration.add(Date.now() - start);

  check(device, {
    'get device 200': (r) => r.status === 200,
    'has x25519':     (r) => r.status === 200 && !!JSON.parse(r.body).x25519,
  });

  sleep(0.2);
}