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

```
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

    checks_total.......: 27588  88.947149/s
    checks_succeeded...: 99.83% 27542 out of 27588
    checks_failed......: 0.16%  46 out of 27588

    ✗ 200
      ↳  99% — ✓ 27542 / ✗ 46

    HTTP
    http_req_duration..............: avg=1.64s  min=33.5ms med=198.37ms max=21.1s  p(90)=7.1s   p(95)=13.71s
      { expected_response:true }...: avg=1.64s  min=33.5ms med=198.53ms max=21.1s  p(90)=7.1s   p(95)=13.71s
    http_req_failed................: 0.14% 46 out of 32186
    http_reqs......................: 32186 103.771674/s

    EXECUTION
    iteration_duration.............: avg=16.52s min=6.66s  med=14.23s   max=28.57s p(90)=21.43s p(95)=27.06s
    iterations.....................: 4598  14.824525/s
    vus............................: 37    min=37          max=250
    vus_max........................: 250   min=250         max=250

    NETWORK
    data_received..................: 24 MB 77 kB/s
    data_sent......................: 16 MB 51 kB/s

running (5m10.2s), 000/250 VUs, 4598 complete and 0 interrupted iterations

created 4592 users in 310.2 seconds
```

```
$ cast balance 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 --ether
9999.882806411469403675

# at avg gas cost of 0.075 gwei,
# cost 0.1171935885304265 ETH to sign up 4592 accounts -> at current (4/12/26) ETH price ~ $260
# ~ 340266 gas / account
# so cost about $0.06 per account
```

```
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

    checks_total.......: 29124  89.410661/s
    checks_succeeded...: 99.73% 29047 out of 29124
    checks_failed......: 0.26%  77 out of 29124

    ✗ 200
      ↳  99% — ✓ 29047 / ✗ 77

    HTTP
    http_req_duration..............: avg=1.58s  min=33.24ms med=195.74ms max=23.35s p(90)=7.05s  p(95)=7.56s
      { expected_response:true }...: avg=1.59s  min=33.24ms med=195.94ms max=23.35s p(90)=7.05s  p(95)=7.57s
    http_req_failed................: 0.22% 77 out of 33978
    http_reqs......................: 33978 104.312438/s

    EXECUTION
    iteration_duration.............: avg=16.13s min=5.49s   med=14.04s   max=31.59s p(90)=22.23s p(95)=27.99s
    iterations.....................: 4854  14.901777/s
    vus............................: 34    min=34          max=250
    vus_max........................: 250   min=250         max=250

    NETWORK
    data_received..................: 25 MB 77 kB/s
    data_sent......................: 17 MB 51 kB/s



                                                                                                                                                                                                                                      
running (5m25.7s), 000/250 VUs, 4854 complete and 0 interrupted iterations                                                                                                                                                            
default ✓ [======================================] 250 VUs  5m0

0.043605478558674804 ETH spent, 

SELECT COUNT(*) FROM "user" u LEFT JOIN device d ON u.id = d.user_id WHERE d.x25519 IS NOT NULL;
5095
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