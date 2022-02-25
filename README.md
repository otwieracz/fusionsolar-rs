# fusionsolar-rs
[![Build Status](https://app.travis-ci.com/otwieracz/fusionsolar-rs.svg?branch=master)](https://app.travis-ci.com/otwieracz/fusionsolar-rs)

`Prometheus` exporter for Huawei PV inverters.

### Exported metrics
* `day_power`: total amount of power generated in current day (in kWh)
* `device_active_power`: active power reported by device
* `device_temperature`: actual temperature reported by device

### Device support
Currently, the only device with implemented KPI fetching is String Inverter:
* `StringInverter` (device type `1`)

To create feature request for any specific device, create an issue with device dump:
```shell
$ curl http://127.0.0.1:8000/dump-devices
```


### Notes
* FusionSolar API has very restrictive API throttling in place. With data being updated not more
often than every 5 minutes, it's highly recommended to set the `FS_INTERVAL` to at least 120 seconds.

### Usage
_requires `cross`[^1] for `musl` cross-compilation_

Set `FS_USERNAME`, `FS_PASSWORD` and `FS_INTERVAL` variables in `.env` file, then:
```shell
$ make
cross build --release --target x86_64-unknown-linux-musl
   Compiling fusionsolar-rs v0.1.0 (/project)
    Finished release [optimized] target(s) in 1m 45s
$ docker compose build
[+] Building 6.2s (7/7) FINISHED
 (...)
 => => exporting layers
 => => writing image sha256:809116c361369209673dc927a119e2235d59c4cbe2bc2f7f26b4755622dcdf73
 => => naming to docker.io/library/fusionsolar-rs_fusionsolar-rs
$ docker compose up
(...)
fusionsolar-rs-fusionsolar-rs-1  | Rocket has launched from http://0.0.0.0:8000
$ curl http://127.0.0.1:8000/metrics
# HELP day_power total amount of power generated in current day (in kWh)
# TYPE day_power gauge
day_power{station_code="sta_code"} 0
# HELP device_active_power active power production reported by inverter
# TYPE device_active_power gauge
device_active_power{device_id="1000000011111111",device_type_id="1",station_code="sta_code"} 0
# HELP device_temperature device reported temperature
# TYPE device_temperature gauge
device_temperature{device_id="1000000011111111",device_type_id="1",station_code="sta_code"} 0

```

[^1]: https://github.com/rust-embedded/cross
