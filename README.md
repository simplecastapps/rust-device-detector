# Introduction
This is a straight port from https://github.com/matomo-org/device-detector.git.

All tests are ported straight from that library and must all pass. It is intended that
this will be in lock step with upstream, though it is possible some user agents will
be exist here that are not accepted upstream, this should be rare.

The latest update commit from parent is 6d427fe171b80fd6f632d3320f06bd14d29c53cb (Tue Jun 27 11:12:10 2023 +0200)

If you wish to contribute non code fixes, it is recommended that you contribute
your patches and tests upstream, then request updates here to bring these projects into parity.


# Build
> cargo build --release

# Test
> cargo test

# Usage
```shell
> rust-device-detector --help
```

A single user agent

```shell
> rust-device-detector 'Googlebot'
{"category":"Search bot","name":"Googlebot","producer":{"name":"Google Inc.","url":"http://www.google.com"},"url":"http://www.google.com/bot.html"}

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

> rust-device-detector -s -p 8080&
> curl --data-binary 'Spotify/8.6.72 iOS/13.5.1 (iPhone9,2)' 'localhost:8080'
# get a result.

And of course it is perfectly usable as a library by adding to your Cargo.toml
```
rust-device-detector = { git = "https://github.com/simplecastapps/rust-device-detector.git", branch = "master" }
```

This will likely be added to crates.io once it has been proven in production and the API has fully settled.

# RoadMap

Need ability to submit headers via commandline.
