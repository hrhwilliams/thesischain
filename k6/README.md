# K6 load tests

Onboarding test that times how long it takes for a user to register and upload
device keys and OTKs to the server, *i.e.*, get their account to a state where
it can accept chat sessions.

```
k6 run --vus 250 --duration 5m create_account.js
```

Test that times how long it takes just for a user to upload their device keys

```
k6 run --vus 250 --duration 5m set_device.js
```

Test that times how long it takes for a user to fetch another user's device
keys. The previous test is basically how long it takes to push device keys
to our key directory; this test is how long it takes to fetch device keys
from it.

```
k6 run --vus 250 --duration 5m get_device.js
```

```
$env:K6_PROMETHEUS_RW_FLUSH_PERIOD="5s"; k6 run --out prometheus-rw=http://localhost:9090/api/v1/write script.js
```