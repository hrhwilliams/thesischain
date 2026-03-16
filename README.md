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

### Stats

```
> k6 run --vus 250 --duration 5m get_test.js    

         /\      Grafana   /‾‾/  
    /\  /  \     |\  __   /  /   
   /  \/    \    | |/ /  /   ‾‾\ 
  /          \   |   (  |  (‾)  |
 / __________ \  |_|\_\  \_____/ 


     execution: local
        script: get_test.js
        output: -

     scenarios: (100.00%) 1 scenario, 250 max VUs, 5m30s max duration (incl. graceful stop):
              * default: 250 looping VUs for 5m0s (gracefulStop: 30s)



  █ TOTAL RESULTS

    checks_total.......: 62292  204.475316/s
    checks_succeeded...: 99.69% 62102 out of 62292
    checks_failed......: 0.30%  190 out of 62292

    ✗ 200
      ↳  99% — ✓ 62102 / ✗ 190

    HTTP
    http_req_duration..............: avg=330.97ms min=18.65ms med=166.6ms  max=2.05s p(90)=872.22ms p(95)=1.05s
      { expected_response:true }...: avg=331.09ms min=29.5ms  med=166.64ms max=2.05s p(90)=873.15ms p(95)=1.06s
    http_req_failed................: 0.26% 190 out of 72674
    http_reqs......................: 72674 238.554535/s

    EXECUTION
    iteration_duration.............: avg=7.32s    min=5.38s   med=7.38s    max=8.01s p(90)=7.57s    p(95)=7.65s
    iterations.....................: 10382 34.079219/s
    vus............................: 234   min=234          max=250
    vus_max........................: 250   min=250          max=250

    NETWORK
    data_received..................: 51 MB 166 kB/s
    data_sent......................: 35 MB 116 kB/s
```