# Introduction
This is a straight port from https://github.com/matomo-org/device-detector.git.

All tests are ported directly from that library and must all pass. It is intended that
this will be in lock step with upstream, though it is possible some user agents will
be exist here that are not accepted upstream, this should be rare.

The latest update commit from parent php project was (Wed Jun 26 20:30:46 2024 +0200)
https://github.com/matomo-org/device-detector/commit/75d88bbefb0182f9207c9f48dc39b1bc8c7cc43f

If you wish to contribute non code fixes, it is recommended that you contribute
your patches and tests upstream, then request updates here to bring these projects into parity.

Any contributions to fix reproducible inaccuracies or improve performance are greatly appreciated.

# Build
```shell
> cargo build --release
```

# Test
```shell
> cargo test
```

# Usage
```shell
> rust-device-detector --help
```

A single user agent. Bots are indicated by a top level "bot" key. Otherwise, it is at least a valid user agent, though all three fields (device, client, os) may be null.

```shell
> rust-device-detector 'Googlebot'
{"bot":{"category":"Search bot","name":"Googlebot","producer":{"name":"Google Inc.","url":"http://www.google.com"},"url":"http://www.google.com/bot.html"}}

> rust-device-detector 'Spotify/8.6.72 iOS/13.5.1 (iPhone9,2)'
{"client":{"engine":null,"engine_version":null,"name":"Spotify","type":"mobile app","version":"8.6.72"},"device":{"brand":"Apple","device_type":"phablet","model":"iPhone 7 Plus"},"is":{"browser":false,"camera":false,"car_browser":false,"console":false,"desktop":false,"feature_phone":false,"feed_reader":false,"library":false,"media_player":false,"mobile":true,"mobile_app":true,"peripheral":false,"pim":false,"portable_media_player":false,"robot":false,"smart_display":false,"smart_phone":false,"smart_speaker":false,"tablet":false,"television":false,"touch_enabled":false},"os":{"family":"iOS","name":"iOS","platform":null,"version":"13.5.1"}}
```

It takes a long time to compile all the some 30k+ regular expressions so calling on a single user agent at a time is not recommended.

Call on many user agents

```
> cat user_agents | head -n 2 | rust-device-detector -i
# each line is a result.
```

Or you may call as a webserver, in which will allow for concurrency.

```shell
> rust-device-detector -s -p 8080&
> curl --data-binary 'Spotify/8.6.72 iOS/13.5.1 (iPhone9,2)' 'localhost:8080'
# get a result.
```

In docker
```shell
> docker build . -t detector
> docker run --name=detector --rm -it -p 8080:8080 detector
```

And of course it is perfectly usable as a library by adding to your Cargo.toml
```
rust-device-detector = { git = "https://github.com/simplecastapps/rust-device-detector.git", branch = "main" }
```

This will likely be added to crates.io once it has been proven in production and the API has fully settled.

# RoadMap

These changes are required before this could be considered for a 1.0 or be submitted to crates.io

* Ability to submit raw headers for commandline and http server mode, presumably with a user agent header within them.
* Need ability to just send entire set of headers including user agent to standard in as well.
