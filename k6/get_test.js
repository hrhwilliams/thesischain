import http from 'k6/http';
import { check, sleep } from 'k6';

/*
  pub device_id: Option<DeviceId>,
  pub ed25519: String,
  pub x25519: String,
  pub signature: String,
*/

// InboundDevice { 
//   device_id: "019cf7a5-4d9c-7350-b991-aa1d84f95ba7", 
//   ed25519: "PbCMOYxMo6vTYC9uGiGcqHpJIIpjWOtvWG2sVSX+TKg", 
//   x25519: "WS2NBb75L0Gcw96kNGfJgh56aPrYxRgjJG3hOifEQ04", 
//   signature: "NPA6Z/4omyLxqz3SACx/NDqifqutUxyCwcL0KBAdCUm4SfZ7vvZ1p2nU6o5VTC9afUUqFhTLPEXZfx8GllLGAA"
// }

const otks = {
  created: [
    "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"
  ],
  removed: [],
  created_signature: "9DspD7IpxEtvDNmmGiEIUBuyRevpY1+WFYlgUrfBSob8J8CjYdMbsEhwLAxDkCSaFv7h1DPJNXR7aKv2cgdUBQ",
  removed_signature: null
};

export default function () {
  const jar = http.cookieJar();
  // set session key
  http.get('https://chat.fiatlux.dev/');

  const username = `test_user_${Math.floor(Math.random() * 1000000000)}`;

  const register = http.post('https://chat.fiatlux.dev/api/auth/register',
    JSON.stringify({ username: username, password: '1234', confirm_password: '1234' }),
    { headers: { 'Content-Type': 'application/json' } }
  );

  check(register, {
    "200": (r) => r.status === 200
  });

  const login = http.post('https://chat.fiatlux.dev/api/auth/login',
    JSON.stringify({ username: username, password: '1234' }),
    { headers: { 'Content-Type': 'application/json' } }
  );

  check(login, {
    "200": (r) => r.status === 200
  });

  const me = http.get('https://chat.fiatlux.dev/api/me');

  check(me, {
    "200": (r) => r.status === 200
  });

  const get_device_id = http.post('https://chat.fiatlux.dev/api/me/device');

  check(get_device_id, {
    "200": (r) => r.status === 200
  });

  try {
    const get_device_id_json = JSON.parse(get_device_id.body);
    
      const upload_device = http.put('https://chat.fiatlux.dev/api/me/device',
        JSON.stringify({
          device_id: get_device_id_json.device_id,
          ed25519: "PbCMOYxMo6vTYC9uGiGcqHpJIIpjWOtvWG2sVSX+TKg",
          x25519: "WS2NBb75L0Gcw96kNGfJgh56aPrYxRgjJG3hOifEQ04",
          signature: "NPA6Z/4omyLxqz3SACx/NDqifqutUxyCwcL0KBAdCUm4SfZ7vvZ1p2nU6o5VTC9afUUqFhTLPEXZfx8GllLGAA"
        }),
        { headers: { 'Content-Type': 'application/json' } }
      )
    
      check(upload_device, {
        "200": (r) => r.status === 200
      });

      const upload_otks = http.post(`https://chat.fiatlux.dev/api/me/device/${get_device_id_json.device_id}/otks`,
        JSON.stringify(otks),
        { headers: { 'Content-Type': 'application/json' } }
      );

      check(upload_otks, {
        "200": (r) => r.status === 200
      });
  } catch (e) {
    console.error(get_device_id.body)
    console.error(e)
  }

  sleep(5)
}