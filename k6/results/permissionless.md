# Run stats

## Get device

```
$ k6 run --vus 5 --iterations 50 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 150     1.255924/s
    checks_succeeded...: 100.00% 150 out of 150
    checks_failed......: 0.00%   0 out of 150

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=67.2ms min=61ms    med=66ms    max=77ms   p(90)=72ms   p(99)=75.03ms



$ k6 run --vus 5 --iterations 50 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 150     1.333819/s
    checks_succeeded...: 100.00% 150 out of 150
    checks_failed......: 0.00%   0 out of 150

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=64.02ms min=53ms    med=63ms    max=76ms   p(90)=71.2ms p(99)=75.5ms



$ k6 run --vus 5 --iterations 50 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 150     1.334307/s
    checks_succeeded...: 100.00% 150 out of 150
    checks_failed......: 0.00%   0 out of 150

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=65.42ms min=52ms    med=65.5ms  max=75ms   p(90)=72ms   p(99)=73.53ms
```

```
$ k6 run --vus 10 --iterations 100 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 300     2.667052/s
    checks_succeeded...: 100.00% 300 out of 300
    checks_failed......: 0.00%   0 out of 300

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=64.44ms min=53ms   med=65ms    max=74ms   p(90)=69ms  p(99)=74ms



$ k6 run --vus 10 --iterations 100 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 300     2.525029/s
    checks_succeeded...: 100.00% 300 out of 300
    checks_failed......: 0.00%   0 out of 300

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=62.99ms min=53ms    med=64ms    max=71ms   p(90)=68ms   p(99)=70.01ms



$ k6 run --vus 10 --iterations 100 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 300     2.524271/s
    checks_succeeded...: 100.00% 300 out of 300
    checks_failed......: 0.00%   0 out of 300

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=65.75ms min=57ms    med=66ms    max=74ms   p(90)=72ms  p(99)=73.01ms
```

```
$ k6 run --vus 25 --iterations 250 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 750     6.310881/s
    checks_succeeded...: 100.00% 750 out of 750
    checks_failed......: 0.00%   0 out of 750

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=65.29ms min=53ms    med=66ms     max=77ms   p(90)=72ms   p(99)=74ms



$ k6 run --vus 25 --iterations 250 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 750     6.290206/s
    checks_succeeded...: 100.00% 750 out of 750
    checks_failed......: 0.00%   0 out of 750

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=64.41ms min=50ms    med=64ms     max=129ms  p(90)=70ms   p(99)=73ms



$ k6 run --vus 25 --iterations 250 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 750     5.97923/s
    checks_succeeded...: 100.00% 750 out of 750
    checks_failed......: 0.00%   0 out of 750

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=65.2ms min=51ms    med=65ms     max=76ms   p(90)=70.09ms p(99)=73.5ms
```


```
$ k6 run --vus 50 --iterations 500 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 1500    12.627293/s
    checks_succeeded...: 100.00% 1500 out of 1500
    checks_failed......: 0.00%   0 out of 1500

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=65.35ms min=49ms    med=66ms     max=87ms   p(90)=73ms   p(99)=78ms



$ k6 run --vus 50 --iterations 500 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 1500    12.624061/s
    checks_succeeded...: 100.00% 1500 out of 1500
    checks_failed......: 0.00%   0 out of 1500

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=64.57ms min=48ms    med=64ms     max=96ms   p(90)=72ms   p(99)=84ms



$ k6 run --vus 50 --iterations 500 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 1500    12.620676/s
    checks_succeeded...: 100.00% 1500 out of 1500
    checks_failed......: 0.00%   0 out of 1500

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=64.79ms min=48ms    med=66ms     max=90ms   p(90)=73ms   p(99)=78ms
```

```
$ k6 run --vus 100 --iterations 1000 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 3000    23.997896/s
    checks_succeeded...: 100.00% 3000 out of 3000
    checks_failed......: 0.00%   0 out of 3000

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=72.84ms min=48ms    med=72ms     max=423ms  p(90)=84ms   p(99)=92ms



$ k6 run --vus 100 --iterations 1000 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 3000    26.594609/s
    checks_succeeded...: 100.00% 3000 out of 3000
    checks_failed......: 0.00%   0 out of 3000

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=73.74ms min=48ms    med=72ms     max=541ms  p(90)=84ms   p(99)=110.04ms



$ k6 run --vus 100 --iterations 1000 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device.js

  █ TOTAL RESULTS

    checks_total.......: 3000    25.252583/s
    checks_succeeded...: 100.00% 3000 out of 3000
    checks_failed......: 0.00%   0 out of 3000

    ✓ get device 200
    ✓ has x25519
    ✓ has ed25519

    CUSTOM
    get_device_ms..................: avg=74.35ms min=53ms    med=72ms     max=390ms  p(90)=90ms   p(99)=113.01ms
```

## Set device

```
$ k6 run --vus 5 --iterations 50 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js

  █ TOTAL RESULTS

    checks_total.......: 50      0.418897/s
    checks_succeeded...: 100.00% 50 out of 50
    checks_failed......: 0.00%   0 out of 50

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=11.73s min=6.79s   med=13.79s   max=13.83s p(90)=13.82s p(99)=13.83s



$ k6 run --vus 5 --iterations 50 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js

  █ TOTAL RESULTS

    checks_total.......: 50      0.420674/s
    checks_succeeded...: 100.00% 50 out of 50
    checks_failed......: 0.00%   0 out of 50

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=11.68s min=6.52s   med=13.8s   max=13.83s p(90)=13.82s p(99)=13.83s



$ k6 run --vus 5 --iterations 50 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js

  █ TOTAL RESULTS

    checks_total.......: 50      0.420686/s
    checks_succeeded...: 100.00% 50 out of 50
    checks_failed......: 0.00%   0 out of 50

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=11.68s min=6.54s   med=13.8s    max=13.83s p(90)=13.81s p(99)=13.83s
```

```
$ k6 run --vus 10 --iterations 100 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js
  █ TOTAL RESULTS

    checks_total.......: 100     0.842705/s
    checks_succeeded...: 100.00% 100 out of 100
    checks_failed......: 0.00%   0 out of 100

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=11.62s min=5.06s   med=13.78s   max=13.81s p(90)=13.79s p(99)=13.8s



$ k6 run --vus 10 --iterations 100 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js
  █ TOTAL RESULTS

    checks_total.......: 100     0.84144/s
    checks_succeeded...: 100.00% 100 out of 100
    checks_failed......: 0.00%   0 out of 100

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=11.66s min=6.67s   med=13.78s  max=13.83s p(90)=13.81s p(99)=13.83s



$ k6 run --vus 10 --iterations 100 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js
  █ TOTAL RESULTS

    checks_total.......: 100     0.889267/s
    checks_succeeded...: 100.00% 100 out of 100
    checks_failed......: 0.00%   0 out of 100

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=11.02s min=6.75s   med=13.77s   max=13.82s p(90)=13.81s p(99)=13.82s
```

```
$ k6 run --vus 25 --iterations 250 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js
  █ TOTAL RESULTS

    checks_total.......: 250     1.986841/s
    checks_succeeded...: 100.00% 250 out of 250
    checks_failed......: 0.00%   0 out of 250

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=12.31s min=6.71s   med=13.73s   max=13.8s  p(90)=13.76s p(99)=13.8s



$ k6 run --vus 25 --iterations 250 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js
  █ TOTAL RESULTS

    checks_total.......: 250     2.103639/s
    checks_succeeded...: 100.00% 250 out of 250
    checks_failed......: 0.00%   0 out of 250

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=11.61s min=6.45s   med=13.72s   max=13.81s p(90)=13.76s p(99)=13.8s
```

```
$ k6 run --vus 50 --iterations 500 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js
  █ TOTAL RESULTS

    checks_total.......: 500     4.209114/s
    checks_succeeded...: 100.00% 500 out of 500
    checks_failed......: 0.00%   0 out of 500

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=11.52s min=6.58s   med=13.57s   max=13.77s p(90)=13.67s p(99)=13.71s



$ k6 run --vus 50 --iterations 500 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js
  █ TOTAL RESULTS

    checks_total.......: 500     4.206971/s
    checks_succeeded...: 100.00% 500 out of 500
    checks_failed......: 0.00%   0 out of 500

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=11.52s min=6.56s   med=13.56s   max=13.75s p(90)=13.68s p(99)=13.72s



$ k6 run --vus 50 --iterations 500 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js
  █ TOTAL RESULTS

    checks_total.......: 500     4.207479/s
    checks_succeeded...: 100.00% 500 out of 500
    checks_failed......: 0.00%   0 out of 500

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=11.52s min=6.54s   med=13.33s   max=13.73s p(90)=13.66s p(99)=13.69s
```

```
$ k6 run --vus 100 --iterations 1000 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js
  █ TOTAL RESULTS

    checks_total.......: 1000    8.416558/s
    checks_succeeded...: 100.00% 1000 out of 1000
    checks_failed......: 0.00%   0 out of 1000

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=11.29s min=6.29s   med=13.36s   max=13.84s p(90)=13.49s p(99)=13.68s



$ k6 run --vus 100 --iterations 1000 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js
  █ TOTAL RESULTS

    checks_total.......: 1000    8.415311/s
    checks_succeeded...: 100.00% 1000 out of 1000
    checks_failed......: 0.00%   0 out of 1000

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=11.28s min=6.25s   med=13.34s   max=13.81s p(90)=13.45s p(99)=13.75s



$ k6 run --vus 100 --iterations 1000 --summary-trend-stats="avg,min,med,max,p(90),p(99)" set_device.js
  █ TOTAL RESULTS

    checks_total.......: 1000    8.415622/s
    checks_succeeded...: 100.00% 1000 out of 1000
    checks_failed......: 0.00%   0 out of 1000

    ✓ put device 200

    CUSTOM
    upload_device_ms...............: avg=11.3s  min=6.31s   med=13.1s    max=13.84s p(90)=13.44s p(99)=13.73s
```

## Device history

```
$ k6 run --vus 50 --iterations 500 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device_history.js  

  █ TOTAL RESULTS

    checks_total.......: 1000    8.363399/s
    checks_succeeded...: 100.00% 1000 out of 1000
    checks_failed......: 0.00%   0 out of 1000

    ✓ get history 200
    ✓ has entries

    CUSTOM
    get_device_history_ms..........: avg=63.34ms min=48ms   med=63ms     max=78ms   p(90)=70ms  p(99)=75.01ms



$ k6 run --vus 50 --iterations 500 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device_history.js

  █ TOTAL RESULTS

    checks_total.......: 1000    8.420446/s
    checks_succeeded...: 100.00% 1000 out of 1000
    checks_failed......: 0.00%   0 out of 1000

    ✓ get history 200
    ✓ has entries

    CUSTOM
    get_device_history_ms..........: avg=64.41ms min=47ms    med=64ms     max=84ms   p(90)=75ms   p(99)=79.03ms



$ k6 run --vus 50 --iterations 500 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device_history.js

  █ TOTAL RESULTS

    checks_total.......: 1000    8.417449/s
    checks_succeeded...: 100.00% 1000 out of 1000
    checks_failed......: 0.00%   0 out of 1000

    ✓ get history 200
    ✓ has entries

    CUSTOM
    get_device_history_ms..........: avg=63.11ms min=47ms    med=63ms    max=79ms   p(90)=70ms   p(99)=76ms



$ k6 run --vus 50 --iterations 500 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device_history.js 

  █ TOTAL RESULTS

    checks_total.......: 1000    8.413997/s
    checks_succeeded...: 100.00% 1000 out of 1000
    checks_failed......: 0.00%   0 out of 1000

    ✓ get history 200
    ✓ has entries

    CUSTOM
    get_device_history_ms..........: avg=64.84ms min=47ms   med=66ms     max=106ms  p(90)=73ms   p(99)=78ms



$ k6 run --vus 50 --iterations 500 --summary-trend-stats="avg,min,med,max,p(90),p(99)" get_device_history.js

  █ TOTAL RESULTS

    checks_total.......: 1000    8.414473/s
    checks_succeeded...: 100.00% 1000 out of 1000
    checks_failed......: 0.00%   0 out of 1000

    ✓ get history 200
    ✓ has entries

    CUSTOM
    get_device_history_ms..........: avg=67.15ms min=47ms    med=67ms     max=85ms   p(90)=74.1ms p(99)=79ms
```
