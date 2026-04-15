# Run stats

## Get device

```
$ k6 run --vus 5 --iterations 50 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 150     11.662285/s
    checks_succeeded...: 100.00% 150 out of 150
    checks_failed......: 0.00%   0 out of 150

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=84.68ms  min=56ms    med=74.5ms   max=122ms p(90)=117.1ms p(99)=121.02ms



$ k6 run --vus 5 --iterations 50 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 150     10.099162/s
    checks_succeeded...: 100.00% 150 out of 150
    checks_failed......: 0.00%   0 out of 150

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=84.84ms  min=54ms     med=77ms     max=118ms p(90)=112.3ms p(99)=117.51ms



$ k6 run --vus 5 --iterations 50 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js  

  █ TOTAL RESULTS

    checks_total.......: 150     11.493538/s
    checks_succeeded...: 100.00% 150 out of 150
    checks_failed......: 0.00%   0 out of 150

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=95.04ms  min=59ms     med=104.5ms  max=129ms p(90)=117ms p(99)=124.09ms
```

```
$ k6 run --vus 10 --iterations 100 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 300     22.7281/s
    checks_succeeded...: 100.00% 300 out of 300
    checks_failed......: 0.00%   0 out of 300

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=85.17ms  min=54ms    med=83.5ms   max=120ms p(90)=112.1ms p(99)=119.01ms



$ k6 run --vus 10 --iterations 100 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js      

  █ TOTAL RESULTS

    checks_total.......: 300     23.514523/s
    checks_succeeded...: 100.00% 300 out of 300
    checks_failed......: 0.00%   0 out of 300

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=87.94ms  min=60ms     med=84ms     max=118ms p(90)=112.1ms p(99)=118ms



$ k6 run --vus 10 --iterations 100 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 300     23.367118/s
    checks_succeeded...: 100.00% 300 out of 300
    checks_failed......: 0.00%   0 out of 300

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=86.4ms   min=53ms     med=82.5ms   max=124ms p(90)=114.1ms p(99)=124ms
```

```
$ k6 run --vus 25 --iterations 250 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 750     59.693893/s
    checks_succeeded...: 100.00% 750 out of 750
    checks_failed......: 0.00%   0 out of 750

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=77.91ms  min=53ms     med=73ms    max=153ms p(90)=101ms p(99)=121.01ms



$ k6 run --vus 25 --iterations 250 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 750     56.153984/s
    checks_succeeded...: 100.00% 750 out of 750
    checks_failed......: 0.00%   0 out of 750

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=78.16ms  min=53ms     med=73ms     max=136ms p(90)=101ms p(99)=121.01ms



$ k6 run --vus 25 --iterations 250 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 750     58.78245/s
    checks_succeeded...: 100.00% 750 out of 750
    checks_failed......: 0.00%   0 out of 750

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=78.34ms min=56ms     med=74ms     max=141ms p(90)=99ms  p(99)=121.51ms
```

```
$ k6 run --vus 50 --iterations 500 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 1500    112.05654/s
    checks_succeeded...: 100.00% 1500 out of 1500
    checks_failed......: 0.00%   0 out of 1500

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=75.65ms  min=53ms    med=72ms     max=151ms p(90)=98ms  p(99)=135.01ms



$ k6 run --vus 50 --iterations 500 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 1500    109.735195/s
    checks_succeeded...: 100.00% 1500 out of 1500
    checks_failed......: 0.00%   0 out of 1500

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=76.56ms  min=54ms     med=72ms     max=132ms p(90)=96ms     p(99)=121.01ms



$ k6 run --vus 50 --iterations 500 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 1500    115.024712/s
    checks_succeeded...: 100.00% 1500 out of 1500
    checks_failed......: 0.00%   0 out of 1500

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=81.12ms  min=54ms     med=73ms    max=208ms p(90)=108.1ms p(99)=193ms
```

```
$ k6 run --vus 100 --iterations 1000 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 3000    167.282868/s
    checks_succeeded...: 100.00% 3000 out of 3000
    checks_failed......: 0.00%   0 out of 3000

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=126.38ms min=49ms     med=79.5ms   max=423ms p(90)=269ms p(99)=355.04ms



$ k6 run --vus 100 --iterations 1000 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 3000    207.96991/s
    checks_succeeded...: 100.00% 3000 out of 3000
    checks_failed......: 0.00%   0 out of 3000

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=96.95ms  min=53ms    med=74ms     max=495ms p(90)=169ms    p(99)=322.16ms



$ k6 run --vus 100 --iterations 1000 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 3000    213.542566/s
    checks_succeeded...: 100.00% 3000 out of 3000
    checks_failed......: 0.00%   0 out of 3000

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=101.7ms  min=49ms     med=73ms     max=423ms p(90)=190.5ms  p(99)=374ms
```

## Set device

```
$ k6 run --vus 5 --iterations 50 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js

  █ TOTAL RESULTS

    checks_total.......: 50      3.972261/s
    checks_succeeded...: 100.00% 50 out of 50
    checks_failed......: 0.00%   0 out of 50

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=1.04s    min=819ms   med=1.07s    max=1.32s p(90)=1.11s p(99)=1.32s



$ k6 run --vus 5 --iterations 50 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js   

  █ TOTAL RESULTS

    checks_total.......: 50      4.002884/s
    checks_succeeded...: 100.00% 50 out of 50
    checks_failed......: 0.00%   0 out of 50

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=1.04s    min=824ms   med=1.07s    max=1.33s p(90)=1.11s p(99)=1.22s



$ k6 run --vus 5 --iterations 50 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js

  █ TOTAL RESULTS

    checks_total.......: 50      4.071767/s
    checks_succeeded...: 100.00% 50 out of 50
    checks_failed......: 0.00%   0 out of 50

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=1.01s    min=818ms   med=1.07s    max=1.12s p(90)=1.08s p(99)=1.12s
```

```
$ k6 run --vus 10 --iterations 100 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js

  █ TOTAL RESULTS

    checks_total.......: 100     7.965447/s
    checks_succeeded...: 100.00% 100 out of 100
    checks_failed......: 0.00%   0 out of 100

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=1.03s    min=822ms   med=1.07s    max=1.32s p(90)=1.11s p(99)=1.13s



$ k6 run --vus 10 --iterations 100 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js

  █ TOTAL RESULTS

    checks_total.......: 100     7.786161/s
    checks_succeeded...: 100.00% 100 out of 100
    checks_failed......: 0.00%   0 out of 100

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=1.06s    min=813ms   med=1.07s    max=1.34s p(90)=1.31s p(99)=1.33s



$ k6 run --vus 10 --iterations 100 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js

  █ TOTAL RESULTS

    checks_total.......: 100     7.715793/s
    checks_succeeded...: 100.00% 100 out of 100
    checks_failed......: 0.00%   0 out of 100

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=1.06s    min=826ms   med=1.08s    max=1.13s p(90)=1.12s p(99)=1.13s
```

```
$ k6 run --vus 25 --iterations 250 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js

  █ TOTAL RESULTS

    checks_total.......: 250     19.977092/s
    checks_succeeded...: 100.00% 250 out of 250
    checks_failed......: 0.00%   0 out of 250

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=1.01s    min=623ms    med=1.07s    max=1.34s p(90)=1.1s  p(99)=1.32s



$ k6 run --vus 25 --iterations 250 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js

  █ TOTAL RESULTS

    checks_total.......: 250     18.483291/s
    checks_succeeded...: 100.00% 250 out of 250
    checks_failed......: 0.00%   0 out of 250

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=1.11s    min=816ms   med=1.08s    max=1.63s p(90)=1.35s p(99)=1.59s



$ k6 run --vus 25 --iterations 250 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js

  █ TOTAL RESULTS

    checks_total.......: 250     19.059564/s
    checks_succeeded...: 100.00% 250 out of 250
    checks_failed......: 0.00%   0 out of 250

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=1.06s    min=818ms   med=1.08s    max=1.34s p(90)=1.32s p(99)=1.33s
```

```
$ k6 run --vus 50 --iterations 500 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js

  █ TOTAL RESULTS

    checks_total.......: 500     28.005107/s
    checks_succeeded...: 100.00% 500 out of 500
    checks_failed......: 0.00%   0 out of 500

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=1.06s    min=574ms   med=1.08s   max=1.96s p(90)=1.33s p(99)=1.59s



$ k6 run --vus 50 --iterations 500 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js

  █ TOTAL RESULTS

    checks_total.......: 500     39.365594/s
    checks_succeeded...: 100.00% 500 out of 500
    checks_failed......: 0.00%   0 out of 500

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=986.36ms min=812ms   med=1.07s    max=1.34s p(90)=1.09s p(99)=1.18s



$ k6 run --vus 50 --iterations 500 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js

  █ TOTAL RESULTS

    checks_total.......: 500     40.438532/s
    checks_succeeded...: 100.00% 500 out of 500
    checks_failed......: 0.00%   0 out of 500

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=948.78ms min=567ms   med=867ms    max=1.34s p(90)=1.11s p(99)=1.21s
```

```
$ k6 run --vus 100 --iterations 1000 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js

  █ TOTAL RESULTS

    checks_total.......: 1000    70.797613/s
    checks_succeeded...: 100.00% 1000 out of 1000
    checks_failed......: 0.00%   0 out of 1000

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=918.61ms min=561ms    med=842ms    max=2.22s p(90)=1.16s    p(99)=1.92s



$ k6 run --vus 100 --iterations 1000 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js

  █ TOTAL RESULTS

    checks_total.......: 1000    62.363144/s
    checks_succeeded...: 100.00% 1000 out of 1000
    checks_failed......: 0.00%   0 out of 1000

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=1.01s    min=560ms    med=847ms    max=2.33s p(90)=1.83s p(99)=2.08s



$ k6 run --vus 100 --iterations 1000 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js 

  █ TOTAL RESULTS

    checks_total.......: 1000    66.877801/s
    checks_succeeded...: 100.00% 1000 out of 1000
    checks_failed......: 0.00%   0 out of 1000

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=919.28ms min=559ms    med=833ms    max=2.29s p(90)=1.58s    p(99)=2s
```

## Device history

```
$ k6 run --vus 50 --iterations 500 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device_history.js  

  █ TOTAL RESULTS

    checks_total.......: 1000    74.182105/s
    checks_succeeded...: 100.00% 1000 out of 1000
    checks_failed......: 0.00%   0 out of 1000

    ✓ get history 200
    ✓ has entries

    CUSTOM
    get_device_history_ms..........: avg=66.15ms  min=42ms     med=61ms    max=143ms p(90)=91.1ms p(99)=120.04ms

$ k6 run --vus 50 --iterations 500 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device_history.js  

  █ TOTAL RESULTS

    checks_total.......: 1000    78.829737/s
    checks_succeeded...: 100.00% 1000 out of 1000
    checks_failed......: 0.00%   0 out of 1000

    ✓ get history 200
    ✓ has entries

    CUSTOM
    get_device_history_ms..........: avg=65.56ms  min=42ms    med=62ms     max=124ms p(90)=83ms  p(99)=109.02ms

$ k6 run --vus 50 --iterations 500 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device_history.js  

  █ TOTAL RESULTS

    checks_total.......: 1000    74.089526/s
    checks_succeeded...: 100.00% 1000 out of 1000
    checks_failed......: 0.00%   0 out of 1000

    ✓ get history 200
    ✓ has entries

    CUSTOM
    get_device_history_ms..........: avg=64.5ms   min=46ms     med=60ms     max=171ms p(90)=73ms  p(99)=148.07ms

$ k6 run --vus 50 --iterations 500 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device_history.js

  █ TOTAL RESULTS

    checks_total.......: 1000    72.935599/s
    checks_succeeded...: 100.00% 1000 out of 1000
    checks_failed......: 0.00%   0 out of 1000

    ✓ get history 200
    ✓ has entries

    CUSTOM
    get_device_history_ms..........: avg=65.78ms  min=47ms     med=60ms     max=162ms p(90)=85ms  p(99)=139.01ms



$ k6 run --vus 50 --iterations 500 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device_history.js

  █ TOTAL RESULTS

    checks_total.......: 1000    73.245906/s
    checks_succeeded...: 100.00% 1000 out of 1000
    checks_failed......: 0.00%   0 out of 1000

    ✓ get history 200
    ✓ has entries

    CUSTOM
    get_device_history_ms..........: avg=67.4ms   min=44ms     med=62ms     max=180ms p(90)=86.1ms p(99)=145.01ms
```