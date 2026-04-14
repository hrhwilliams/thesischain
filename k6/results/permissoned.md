# Run stats

```
Tuesday, April 14, 2026 11:44:00 PM PDT

$ k6 run --vus 250 --duration 5m create_account.js

     execution: local
        script: create_account.js
        output: -

     scenarios: (100.00%) 1 scenario, 250 max VUs, 5m30s max duration (incl. graceful stop):
              * default: 250 looping VUs for 5m0s (gracefulStop: 30s)



  █ TOTAL RESULTS

    checks_total.......: 51210  167.796617/s
    checks_succeeded...: 99.61% 51014 out of 51210
    checks_failed......: 0.38%  196 out of 51210

    ✗ 200
      ↳  99% — ✓ 51014 / ✗ 196

    HTTP
    http_req_duration..............: avg=560.83ms min=35.54ms med=406.37ms max=3.05s  p(90)=1.24s p(95)=1.41s
      { expected_response:true }...: avg=561.45ms min=35.54ms med=407.01ms max=3.05s  p(90)=1.24s p(95)=1.41s
    http_req_failed................: 0.32% 196 out of 59745
    http_reqs......................: 59745 195.76272/s

    EXECUTION
    iteration_duration.............: avg=8.93s    min=5.94s   med=8.95s    max=10.63s p(90)=9.17s p(95)=9.26s
    iterations.....................: 8535  27.966103/s
    vus............................: 200   min=200          max=250
    vus_max........................: 250   min=250          max=250

    NETWORK
    data_received..................: 44 MB 143 kB/s
    data_sent......................: 29 MB 95 kB/s

running (5m05.2s), 000/250 VUs, 8535 complete and 0 interrupted iterations

end2=# select count(*) from device where x25519 is not null;
 count 
-------
  8505

end2=# select count(*) from device where x25519 is null;
 count 
-------
     3
```