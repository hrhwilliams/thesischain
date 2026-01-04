# End2

Simple centralized end-to-end encrypted chat service.

## Definitions

- User
- Challenge
- WebSession
- OlmSession
- One-Time Key
- Channel
- Message

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