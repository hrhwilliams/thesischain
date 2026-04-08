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
|miner|[crates/miner](crates/miner)|code for miner node|

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

default ✓ [======================================] 250 VUs  5m0s


created 11056 users
```