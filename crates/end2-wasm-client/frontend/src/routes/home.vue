<script setup lang="js">
import enUsPatterns from 'hyphenation.en-us'
import { createHyphenator, justifyContent } from 'tex-linebreak'
import { onMounted } from 'vue'

const hyphenate = createHyphenator(enUsPatterns)

// onMounted(async () => {
//     await document.fonts.ready
//     const paragraphs = Array.from(document.querySelectorAll('p'))
//     justifyContent(paragraphs, hyphenate)
// })
</script>

<template>
    <article>
    <h2>About</h2>
    <p>
        Hi! This is End2, a very basic <a href="https://en.wikipedia.org/wiki/End-to-end_encryption">end-to-end encrypted</a>
        messaging app I made as part of my thesis. End to end encryption means that messages you send are encrypted on your side,
        and only ever able to be decrypted by the other person you&#x2019;re talking with. This works by generating cryptographic
        key pairs on your device which are used to encrypt and decrypt messages. The private parts of these keys never leave your
        device, ensuring that messages sent to that device can only be read on that device.
    </p>
    <p>
        You can either make an account with a username and password, or by linking your Discord. Accounts are used to track which
        devices belong to what user, so that you can send and receive messages from any device you log in on.
    </p>
    <h3>Technical details</h3>
    <p>
        End2 has a classic frontend-backend architecture, with a frontend written in Vue calling a backend API written in Rust.
        All encryption and decryption occur exclusively on the client. On their browser, the client runs a
        <a href="https://en.wikipedia.org/wiki/WebAssembly">WebAssembly</a> module that wraps 
        <a href="https://github.com/matrix-org/vodozemac">vodozemac</a> for running the
        <a href="https://signal.org/docs/specifications/doubleratchet/">Double Ratchet algorithm</a>, and device context such as
        encrypted session state and device keys. The backend stores only public parts of keys that the user uploads, and encrypted
        chat messages for relay to chat participants who were offline when messages were sent.
    </p>
    <p>
        On setup, the user generates an Ed25519 key pair and <a href="https://en.wikipedia.org/wiki/Curve25519">X25519</a> key pair and sends the public parts to the server. The user then
        initializes a device context with the private parts of these keys. These key pairs act as long-term identity keys which do
        not change after setup. Next, the user generates and uploads one-time keys which are used in the pre-key phase of the Double
        Ratchet algorithm. After this, the user's device is considered initialized, and other users wishing to send them messages
        can request their public X25519 key and a one-time key from the server to create an end-to-end encrypted session with them.
    </p>
    <p>
        The Double Ratchet algorithm utilizes two cryptographic ratchets to provide security. One is the symmetric ratchet, which derives
        a new key after every message, ensuring that past messages cannot be read on compromise of the symmetric encryption key. The other
        is the <a href="https://en.wikipedia.org/wiki/Diffie%E2%80%93Hellman_key_exchange">Diffie-Hellman</a> ratchet, which establishes a new and cryptographically random symmetric encryption key every message, ensuring
        that future messages also cannot be read upon compromise. To establish a session, the algorithm requires both an identity key 
        (X25519 public key) and one-time key from the intended recipient. Once a reply back from the other user is received, the session
        is considered established.
    </p>
    <p>
        Here is the following message as an encrypted packet:
    </p>
        <blockquote>
            Here is an example of a message which will be sent as ciphertext to the server, which will only be possible to decrypt on the
            recipient's device.
        </blockquote>
    <pre>
     {
       "message_id":"019ba919-f8cc-7a13-ad5f-12f944452bf1",
       "device_id":"019ba700-b5f1-706e-ba96-261bba532a06",
       "channel_id":"019ba700-f0b7-70c6-8484-115ac9283390",
       "ciphertext":"BAogJ3kjPX/D+a6Mu79fc0vg2lW6w0iPPPAnqYQeh2fvTF
       0QASKQAcKwu7ND4EpVahgmdAxmizLSDse4U03zxC1kIvEgGYNs8iKKTleTKV
       Up+ch4p8UPcuN6fv6vTLAzBcE2H5gS9d4uNp8w8hc2fKSUZr++AdbI5qr1rG
       VaOZuArWjWKRV9UtMvRWTeL2Y4NBc3BNjo7wz11w4rbUPv8aH26i2DJfm5F/
       KmYuFaZ3W6OWndQcHP3UTIua5FNLycYYs5gXdsKnQBaT2tOHlHLyQCx6XwDk
       zC",
       "timestamp":"2026-01-10T18:10:10.256008Z",
       "is_pre_key":true
     }
    </pre>
    </article>
</template>

<style scoped>
article {
    text-align: justify;
    /* hyphens: auto; */
    max-width: 660px;
    margin: auto;
}
</style>