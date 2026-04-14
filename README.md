# End2

An experimental user-auditable end-to-end encrypted messaging application. End2
breaks down end-to-end messaging into three parts: entity authentication, key
distribution, and message relay. The key directory is managed by the users of
the service with a distributed ledger, preventing the service from maliciously
distributing keys and thus compromising the security of its users intentionally
or due to a compromise.

## Organization

|crate|path|description|
|-----|---|----------|
|end2 |[crates/end2](crates/end2)|backend |
|end2-wasm-client|[crates/end2-wasm-client](crates/end2-wasm-client)|frontend and client-side encryption WASM library|
|end2-api-client|[crates/end2](crates/end2-api-client)|purely CLI-based client|

### Stats

```
Monday, March 16, 2026 5:47:28 PM PDT

$ k6 run --vus 250 --duration 5m get_test.js    

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

    checks_total.......: 66444  216.588969/s
    checks_succeeded...: 99.63% 66201 out of 66444
    checks_failed......: 0.36%  243 out of 66444

    ✗ 200
      ↳  99% — ✓ 66201 / ✗ 243

    HTTP
    http_req_duration..............: avg=272ms    min=30.49ms med=158.93ms max=2.24s p(90)=668.52ms p(95)=861.63ms
      { expected_response:true }...: avg=272.28ms min=30.49ms med=159.64ms max=2.24s p(90)=669.39ms p(95)=862.08ms
    http_req_failed................: 0.31% 243 out of 77518
    http_reqs......................: 77518 252.68713/s

    EXECUTION
    iteration_duration.............: avg=6.91s    min=5.37s   med=7.07s    max=8.15s p(90)=7.25s    p(95)=7.33s
    iterations.....................: 11074 36.098161/s
    vus............................: 215   min=215          max=250
    vus_max........................: 250   min=250          max=250

    NETWORK
    data_received..................: 54 MB 175 kB/s
    data_sent......................: 38 MB 122 kB/s

running (5m06.8s), 000/250 VUs, 11074 complete and 0 interrupted iterations 

created 11056 users
```

#### Ethereum block time (12s)

gas fee seems to be 140275 to upload new device key

```
anvil --host 0.0.0.0 --block-time 12 --block-base-fee-per-gas 150000000 &
k6 run --vus 250 --duration 5m get_test.js  

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

    checks_total.......: 26592  82.076293/s
    checks_succeeded...: 99.79% 26537 out of 26592
    checks_failed......: 0.20%  55 out of 26592

    ✗ 200
      ↳  99% — ✓ 26537 / ✗ 55

    HTTP
    http_req_duration..............: avg=1.76s min=34.17ms med=219.89ms max=28.23s p(90)=7.11s  p(95)=13.99s
      { expected_response:true }...: avg=1.77s min=34.17ms med=220.19ms max=28.23s p(90)=7.11s  p(95)=13.99s
    http_req_failed................: 0.17% 55 out of 31024
    http_reqs......................: 31024 95.755675/s

    EXECUTION
    iteration_duration.............: avg=17.4s min=5.55s   med=14.27s   max=36.43s p(90)=22.45s p(95)=27.43s
    iterations.....................: 4432  13.679382/s
    vus............................: 1     min=1           max=250
    vus_max........................: 250   min=250         max=250

    NETWORK
    data_received..................: 23 MB 71 kB/s
    data_sent......................: 15 MB 47 kB/s

running (5m24.0s), 000/250 VUs, 4432 complete and 0 interrupted iterations

created 4427 users -> 140275 * 4427 = 620997425 gas
```

```
$ k6 run --vus 250 --duration 5m get_test.js  

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

    checks_total.......: 27240  85.753123/s
    checks_succeeded...: 99.53% 27112 out of 27240
    checks_failed......: 0.46%  128 out of 27240

    ✗ 200
      ↳  99% — ✓ 27112 / ✗ 128

    HTTP
    http_req_duration..............: avg=1.68s  min=33.74ms med=215.86ms max=21.18s p(90)=7.1s   p(95)=13.27s
      { expected_response:true }...: avg=1.69s  min=33.74ms med=217.04ms max=21.18s p(90)=7.1s   p(95)=13.28s
    http_req_failed................: 0.40% 128 out of 31780
    http_reqs......................: 31780 100.04531/s

    EXECUTION
    iteration_duration.............: avg=16.81s min=5.56s   med=14.26s   max=29.73s p(90)=22.13s p(95)=27.28s
    iterations.....................: 4540  14.292187/s
    vus............................: 37    min=37           max=250
    vus_max........................: 250   min=250          max=250

    NETWORK
    data_received..................: 24 MB 74 kB/s
    data_sent......................: 16 MB 49 kB/s


running (5m17.7s), 000/250 VUs, 4540 complete and 0 interrupted iterations

created 4529 users -> 140275 * 4529 = 635305475 gas
```

```
$ k6 run --vus 250 --duration 5m get_test.js  

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

    checks_total.......: 26802  84.615286/s
    checks_succeeded...: 99.76% 26738 out of 26802
    checks_failed......: 0.23%  64 out of 26802

    ✗ 200
      ↳  99% — ✓ 26738 / ✗ 64

    HTTP
    http_req_duration..............: avg=1.76s  min=36.07ms med=204.12ms max=21.83s p(90)=7.1s   p(95)=14.06s
      { expected_response:true }...: avg=1.76s  min=36.07ms med=204.27ms max=21.83s p(90)=7.11s  p(95)=14.06s
    http_req_failed................: 0.20% 64 out of 31269
    http_reqs......................: 31269 98.717834/s

    EXECUTION
    iteration_duration.............: avg=17.36s min=5.52s   med=14.35s   max=30.12s p(90)=22.51s p(95)=27.05s
    iterations.....................: 4467  14.102548/s
    vus............................: 37    min=37          max=250
    vus_max........................: 250   min=250         max=250

    NETWORK
    data_received..................: 23 MB 73 kB/s
    data_sent......................: 15 MB 49 kB/s

running (5m16.8s), 000/250 VUs, 4467 complete and 0 interrupted iterations 

created 4455 users in 316.8s -> 140275 * 4427 = 624925125 gas
at 2 gwei/gas, 1.249 ETH or $2784.61, ~ $0.62 per user on 4/13/2026  ($6.2e-1)
at 0.2 gwei/gas, 0.1249 ETH or $278.46, ~ $0.06 per user on 4/13/2026 ($6.2e-2)

Amazon gp2 storage is $0.115 per GB-month
a set of device keys for a user is 32 + 32 + 16 + 16 (2 keys, 2 128-bit uuids),
so 96 bytes.
1 GB is 1,000,000,000, so
0.115 / 1,000,000,000 = x / 96 = $1.1e-8 to store device keys of one user for
a month.

i.e., storage on ethereum blockchain is anywhere from 1 million to 10 million times as expensive
```

#### Base block time (2s)

```
```

```
$ cast balance 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266

```

#### Arbitrum block time (0.25s)

```
```

```
$ cast balance 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266

```