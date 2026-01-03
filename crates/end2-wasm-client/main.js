import { add_rs, End2ClientSession } from './pkg/end2_wasm_client.js';

console.log("1 + 2 = " + add_rs(1, 2));

let client = End2ClientSession.new();

let id = client.get_identity_keys();
let otks = client.generate_otks(3);

console.log(id);
console.log(otks);

try {
    const payload = client.get_registration_payload("Maki");
    console.log(payload)
    const response = await fetch('http://localhost:8081/api/auth/register', {
        method: 'POST',
        headers: {
            'content-type': 'application/json'
        },
        body: JSON.stringify(payload)
    });

    if (!response.ok) {
        const text = await response.text();
        throw new Error(`Server Error: ${response.status} ${text}`);
    }

    console.log(await response.json());
} catch (err) {
    console.error("Registration failed:", err);
}

try {
    const challenge = await fetch(`http://localhost:8081/api/auth/challenge?user=${"Maki"}`, {
        method: 'GET'
    })

    if (!challenge.ok) {
        const text = await challenge.text();
        throw new Error(`Server Error: ${challenge.status} ${text}`);
    }

    const challengeData = await challenge.json();
    console.log(challengeData);
    const signature = client.sign_challenge(challengeData);

    console.log(JSON.stringify({
        id: challengeData.id,
        signature: signature
    }))

    const response = await fetch(`http://localhost:8081/api/auth/challenge`, {
        method: 'POST',
        headers: {
            'content-type': 'application/json'
        },
        credentials: 'include',
        body: JSON.stringify({
            id: challengeData.id,
            signature: signature
        })
    });

    if (!response.ok) {
        const text = await response.text();
        throw new Error(`Server Error: ${response.status} ${text}`);
    }
} catch (err) {
    console.error("Challenge failed:", err);
}

try {
    const me = await fetch(`http://localhost:8081/api/auth/me`, {
        method: 'GET',
        credentials: 'include'
    });

    if (!me.ok) {
        const text = await me.text();
        throw new Error(`Server Error: ${me.status} ${text}`);
    }

    const meData = await me.json();
    console.log(meData);
} catch (err) {
    console.error("Auth failed:", err);
}