# Thesischain

- Two types of node: Client node and Miner node
- Client node messages on behalf of user, Miner node interfaces with the Blockchain
  - Nodes communicate via libp2p
- Pending messages can be stored encrypted for a time via IPFS, solving asynchronicity
- biggest two challenges:
  - getting valid identity/public key on the chain - solved by proof-of-possession
  - finding and establishing a channel between two users whose identities are on the
    chain

https://docs.ipfs.tech/concepts/ipns/#mutability-in-ipfs