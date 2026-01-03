import { add_rs, End2ClientSession } from './pkg/end2_wasm_client.js';

console.log("1 + 2 = " + add_rs(1, 2));

let client = End2ClientSession.new();

let id = client.get_identity_keys();
let otks = client.generate_otks(3);

console.log(id);
console.log(otks);

try {
    const stuff = client.get_registration_payload("Maki")
    console.log(stuff)
    const response = await fetch('http://localhost:8081/api/auth/register', {
        method: 'POST',
        headers: {
            'content-type': 'application/json'
        },
        body: stuff
    })

    if (!response.ok) {
        const text = await response.text();
        throw new Error(`Server Error: ${response.status} ${text}`);
    }

    console.log(await response.json())
} catch (err) {
    console.error("Registration failed:", err);
}