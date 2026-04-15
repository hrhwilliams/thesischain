# Run stats

## Get device

```
$ k6 run --vus 5 --iterations 50 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 150     47.145247/s
    checks_succeeded...: 100.00% 150 out of 150
    checks_failed......: 0.00%   0 out of 150

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=56.2ms   min=42ms     med=56ms     max=76ms     p(90)=65.2ms   p(99)=74.03ms



$ k6 run --vus 5 --iterations 50 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 150     49.820393/s
    checks_succeeded...: 100.00% 150 out of 150
    checks_failed......: 0.00%   0 out of 150

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=53.92ms  min=43ms     med=54ms     max=61ms     p(90)=58.1ms   p(99)=61ms



$ k6 run --vus 5 --iterations 50 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 150     49.092107/s
    checks_succeeded...: 100.00% 150 out of 150
    checks_failed......: 0.00%   0 out of 150

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=53.88ms  min=43ms     med=54ms     max=69ms     p(90)=60ms     p(99)=68.02ms
```

```
$ k6 run --vus 10 --iterations 100 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 300     97.413885/s
    checks_succeeded...: 100.00% 300 out of 300
    checks_failed......: 0.00%   0 out of 300

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=55.44ms  min=41ms     med=55ms     max=77ms     p(90)=62ms     p(99)=72.05ms



$ k6 run --vus 10 --iterations 100 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 300     102.283939/s
    checks_succeeded...: 100.00% 300 out of 300
    checks_failed......: 0.00%   0 out of 300

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=53.59ms  min=41ms     med=54ms     max=68ms     p(90)=60.1ms   p(99)=63.05ms



$ k6 run --vus 10 --iterations 100 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 300     99.620023/s
    checks_succeeded...: 100.00% 300 out of 300
    checks_failed......: 0.00%   0 out of 300

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=53.36ms  min=40ms     med=52ms     max=72ms     p(90)=63ms     p(99)=69.03ms
```

```
$ k6 run --vus 25 --iterations 250 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 750     228.803527/s
    checks_succeeded...: 100.00% 750 out of 750
    checks_failed......: 0.00%   0 out of 750

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=53.03ms  min=36ms     med=54ms     max=73ms     p(90)=60.09ms  p(99)=67ms



$ k6 run --vus 25 --iterations 250 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 750     218.443747/s
    checks_succeeded...: 100.00% 750 out of 750
    checks_failed......: 0.00%   0 out of 750

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=54.06ms  min=45ms     med=54ms     max=76ms     p(90)=61ms     p(99)=71ms



$ k6 run --vus 25 --iterations 250 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 750     235.976187/s
    checks_succeeded...: 100.00% 750 out of 750
    checks_failed......: 0.00%   0 out of 750

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=54.52ms  min=41ms     med=54ms     max=89ms     p(90)=61ms     p(99)=72ms
```

```
$ k6 run --vus 50 --iterations 500 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 1500    365.495206/s
    checks_succeeded...: 100.00% 1500 out of 1500
    checks_failed......: 0.00%   0 out of 1500

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=65.06ms  min=41ms     med=55ms     max=200ms    p(90)=97.2ms   p(99)=156.01ms



$ k6 run --vus 50 --iterations 500 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 1500    362.539832/s
    checks_succeeded...: 100.00% 1500 out of 1500
    checks_failed......: 0.00%   0 out of 1500

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=63.27ms  min=38ms     med=55ms    max=165ms    p(90)=91ms     p(99)=120.01ms



$ k6 run --vus 50 --iterations 500 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 1500    371.064545/s
    checks_succeeded...: 100.00% 1500 out of 1500
    checks_failed......: 0.00%   0 out of 1500

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=61.88ms min=38ms     med=56ms     max=166ms    p(90)=83ms     p(99)=146.03ms
```

```
$ k6 run --vus 100 --iterations 1000 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 3000    418.510087/s
    checks_succeeded...: 100.00% 3000 out of 3000
    checks_failed......: 0.00%   0 out of 3000

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=105.02ms min=36ms    med=83ms     max=437ms    p(90)=198ms    p(99)=365.02ms



$ k6 run --vus 100 --iterations 1000 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 3000    403.992029/s
    checks_succeeded...: 100.00% 3000 out of 3000
    checks_failed......: 0.00%   0 out of 3000

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=106.32ms min=40ms     med=80ms     max=456ms    p(90)=203.4ms  p(99)=369.15ms



$ k6 run --vus 100 --iterations 1000 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 3000    414.460163/s
    checks_succeeded...: 100.00% 3000 out of 3000
    checks_failed......: 0.00%   0 out of 3000

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=100.5ms  min=40ms     med=76.5ms   max=433ms    p(90)=199ms    p(99)=336ms
```

## Set device

```
$ k6 run --vus 5 --iterations 50 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js    

  █ TOTAL RESULTS

    checks_total.......: 50      20.19028/s
    checks_succeeded...: 100.00% 50 out of 50
    checks_failed......: 0.00%   0 out of 50

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=57.74ms  min=42ms     med=58.5ms   max=72ms     p(90)=66ms     p(99)=69.54ms


$ k6 run --vus 5 --iterations 50 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js

  █ TOTAL RESULTS

    checks_total.......: 50      19.665276/s
    checks_succeeded...: 100.00% 50 out of 50
    checks_failed......: 0.00%   0 out of 50

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=59.32ms  min=44ms    med=58ms     max=95ms     p(90)=68ms     p(99)=85.19ms



$ k6 run --vus 5 --iterations 50 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js

  █ TOTAL RESULTS

    checks_total.......: 50      20.293444/s
    checks_succeeded...: 100.00% 50 out of 50
    checks_failed......: 0.00%   0 out of 50

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=57.96ms  min=50ms     med=57ms     max=69ms     p(90)=66ms     p(99)=68.5ms
```

```
$ k6 run --vus 10 --iterations 100 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js

  █ TOTAL RESULTS

    checks_total.......: 100     37.69505/s
    checks_succeeded...: 100.00% 100 out of 100
    checks_failed......: 0.00%   0 out of 100

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=58.49ms  min=47ms     med=59ms     max=70ms     p(90)=65ms     p(99)=68.02ms



$ k6 run --vus 10 --iterations 100 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js

  █ TOTAL RESULTS

    checks_total.......: 100     40.657206/s
    checks_succeeded...: 100.00% 100 out of 100
    checks_failed......: 0.00%   0 out of 100

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=55.27ms  min=41ms     med=54.5ms   max=83ms     p(90)=63ms     p(99)=81.02ms



$ k6 run --vus 10 --iterations 100 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js

  █ TOTAL RESULTS

    checks_total.......: 100     39.992817/s
    checks_succeeded...: 100.00% 100 out of 100
    checks_failed......: 0.00%   0 out of 100

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=55.96ms  min=41ms     med=56ms    max=73ms     p(90)=63.1ms   p(99)=71.02ms
```

```
$ k6 run --vus 25 --iterations 250 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js

  █ TOTAL RESULTS

    checks_total.......: 250     89.184426/s
    checks_succeeded...: 100.00% 250 out of 250
    checks_failed......: 0.00%   0 out of 250

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=58.04ms min=42ms     med=56.5ms   max=99ms     p(90)=66ms     p(99)=94.51ms
                                                                                                               
$ k6 run --vus 25 --iterations 250 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js

  █ TOTAL RESULTS

    checks_total.......: 250     94.979812/s
    checks_succeeded...: 100.00% 250 out of 250
    checks_failed......: 0.00%   0 out of 250

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=55.56ms  min=42ms    med=54ms     max=83ms     p(90)=65.09ms  p(99)=71.5ms



$ k6 run --vus 25 --iterations 250 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js

  █ TOTAL RESULTS

    checks_total.......: 250     94.799759/s
    checks_succeeded...: 100.00% 250 out of 250
    checks_failed......: 0.00%   0 out of 250

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=54.47ms  min=41ms     med=54ms     max=70ms     p(90)=63ms     p(99)=69ms
```

```
$ k6 run --vus 50 --iterations 500 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js

  █ TOTAL RESULTS

    checks_total.......: 500     130.393853/s
    checks_succeeded...: 100.00% 500 out of 500
    checks_failed......: 0.00%   0 out of 500

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=66.47ms  min=42ms     med=59ms     max=175ms    p(90)=101.1ms  p(99)=162.02ms



$ k6 run --vus 50 --iterations 500 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js

  █ TOTAL RESULTS

    checks_total.......: 500     125.090541/s
    checks_succeeded...: 100.00% 500 out of 500
    checks_failed......: 0.00%   0 out of 500

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=73.04ms  min=41ms     med=66ms    max=211ms    p(90)=104.1ms  p(99)=156.11ms


$ k6 run --vus 50 --iterations 500 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js

  █ TOTAL RESULTS

    checks_total.......: 500     131.535956/s
    checks_succeeded...: 100.00% 500 out of 500
    checks_failed......: 0.00%   0 out of 500

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=73.64ms  min=42ms     med=62ms     max=230ms    p(90)=112ms    p(99)=164.13ms
```

```
$ k6 run --vus 100 --iterations 1000 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js

  █ TOTAL RESULTS

    checks_total.......: 1000    147.266476/s
    checks_succeeded...: 100.00% 1000 out of 1000
    checks_failed......: 0.00%   0 out of 1000

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=171.42ms min=41ms     med=155ms    max=577ms    p(90)=284ms    p(99)=431.01ms



$ k6 run --vus 100 --iterations 1000 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js

  █ TOTAL RESULTS

    checks_total.......: 1000    149.710908/s
    checks_succeeded...: 100.00% 1000 out of 1000
    checks_failed......: 0.00%   0 out of 1000

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=175.3ms  min=46ms     med=164ms    max=437ms    p(90)=272ms    p(99)=360.02ms



$ k6 run --vus 100 --iterations 1000 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js

  █ TOTAL RESULTS

    checks_total.......: 1000    149.278982/s
    checks_succeeded...: 100.00% 1000 out of 1000
    checks_failed......: 0.00%   0 out of 1000

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=166.4ms  min=44ms     med=150ms    max=504ms    p(90)=279ms   p(99)=409.04ms
```