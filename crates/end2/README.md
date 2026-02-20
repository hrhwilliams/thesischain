# End2

An end-to-end encrypted (E2EE) chat application with support for centralized (single-entity trust) key distribution and distributed ledger key distribution.

## Motivation

One key source of distrust in E2EE chat applications is the offloading of key distribution to the entity in control of the application. This is done because it is far more convenient to the user than manually verifying and distributing keys themselves, such as with OpenPGP. However, the trade-off in this is that the user now must trust the service to distribute authentic keys from the other users they wish to talk to.

To somewhat mitigate this concern, many services now allow users to compare keys when they are in physical proximity. However, this requires manual intervention by the users involved, knowledge of public-key cryptography, and does not scale when the vast majority of users are not often in physical proximity with the users they are talking with.

The goal of End2 is to provide a solution that does scale by offloading key distribution to a user-auditable distributed ledger. The server still plays the role of entity authentication and message relay, but is unable to distribute all of the cryptographic material involved in establishing E2EE sessions. Instead, the server only authenticates users involved in a session, and users request keys from a user-maintained and fully auditable distributed ledger. This means that the server alone is incapable of compromising sessions by distributing inauthentic keys.

## Design

End2 was created to compare two different methods of key distribution for E2EE: centralized and decentralized. The former method is what is found in most popular E2EE chat applications, such as Telegram, WhatsApp, and Facebook Messenger. It relies on a single entity (*e.g.* WhatsApp) to distribute cryptographic keys for establishing sessions, which is problematic for user trust. The latter method offloads key distribution to a distributed network of peers storing cryptographic keys via a distributed ledger protocol.

The rest of this document will discuss only End2 with decentralized key distribution.

### Security goals

As stated above, the main motivation for creating End2 is to provide an E2EE chat application that does not require the user to trust the service to properly distribute keys. This eliminates a key attack vector that is exploitable by other E2EE chat applications using a centralized model of key distribution.

#### Threat model

End2's threat model assumes even the service itself can act adversarially and thus is designed to minimize possible attacks it can take against its own users.

### System architecture

The End2 backend is split into three main services: entity authentication, key distribution, and message relay. Entity authentication involves registration of users and provenance of user identity. Key distribution involves distributing cryptographic keys used for E2EE session establishment and continuation. Finally, message relay involves how messages are forwarded to their intended recipients.

#### Centralized key distribution

When running with centralized key distribution, the server controls user authentication, message relay, and both long-term key and one-time key distribution. Device long-term keys and one-time keys are stored in the backend's PostgreSQL database.

#### Blockchain-based key distribution

When running with blockchain-based key distribution, the server controls user authentication, message relay, and one-time key distribution, but not long-term key distribution.

## Flow

1. A user visits the `/register` endpoint, enters a username, generates and 
   stores an ed25519 and an x25519 key pair, and sends the public keys of those
   key pairs to the server.
2. The user visits the `/login` endpoint, enters their username again, and then
   the server sends a Challenge for the user to sign and return with their
   ed25519 key. If the server is able to verify the signature with the user's
   public ed25519 key, the login succeeds. The login does not use a password
   because possession of the private signing key is enough to prove
   authenticity.
   - On success, the server returns a WebSession token, which authenticates the
     user for all future requests until the token expires.
   - On failure, the user returns to the `/login` endpoint to try again
3. Once the user is logged in, the user can visit the `/messages` endpoint to
   see all Channels that they are currently in or initiate a new Channel with
   another user.
   - To initiate a new Channel with another user, the user looks up the
     recipient's identity key from the server, requests an unused One-Time Key
     from the server, and then is able to create the channel
4. The user visits the `/messages/{channel_id}` endpoint to send messages
   through a channel to another user.
   - A user acting as the sender (first messager) creates an outbound OlmSession
     from the recipient's public curve25519 key and one of their OTKs which the
     sender requests from the server.
   - A user acting as the recipient creates an inbound OlmSession