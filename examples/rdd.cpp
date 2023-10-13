using namespace std;
#include <iostream>
#include "rdd.h"

void lookup(const RDDDeviceDetector *dd, const char *ua);

int main() {
  RDDDeviceDetector *dd = rdd_device_detector_new(20000);

  lookup(dd, "googlebot");
  lookup(dd, "Spotify/7.6.84.1240 Android/23 (Lenovo A7020a48)");
  lookup(dd, "AppleCoreMedia/1.0.0.12B466 (Apple TV; U; CPU OS 8_1_3 like Mac OS X; en_us)");
  lookup(dd, "Mozilla/5.0 (Linux; Android 12; SM-A037U1 Build/SP1A.210812.016; wv) AppleWebKit/537.36 (KHTML, like Gecko) Version/4.0 Chrome/110.0.5481.153 Mobile Safari/537.36 [FB_IAB/FB4A;FBAV/404.0.0.35.70;]");
  lookup(dd, "Mozilla/5.0 (Linux; Android 10; R2021W2 Build/QP1A.190711.020; wv) AppleWebKit/537.36 (KHTML, like Gecko) Version/4.0 Chrome/74.0.3729.186 Mobile Safari/537.36");
  lookup(dd, "Mozilla/5.0 (Linux; Android 10; R2021W2 Build/QP1A.190711.020; wv) AppleWebKit/537.36 (KHTML, like Gecko) Version/4.0 Chrome/74.0.3729.186 Mobile Safari/537.36");
  lookup(dd, "Mozilla/5.0 (Linux; Android 11; SAMSUNG SM-R925N) AppleWebKit/537.36 (KHTML, like Gecko) SamsungBrowser/1.2. Chrome/90.0.4430.210 Mobile Safari/537.36");
  lookup(dd, "פודקאסטים/1420.35 CFNetwork/1120 Darwin/19.0.0");

  rdd_free_device_detector(dd);
}

void lookup(const RDDDeviceDetector *dd, const char *ua) {
  cout << "looking up '" << ua << "'" << endl;
  RDDDetection *dt = rdd_lookup(dd, ua);

  cout << "IS" << endl;

  cout << "  bot: " << rdd_is_bot(dt) << endl;
  cout << "  mobile: " << rdd_is_mobile(dt) << endl;
  cout << "  touch_enabled: " << rdd_is_touch_enabled(dt) << endl;
  cout << "  pim: " << rdd_is_pim(dt) << endl;
  cout << "  feed_reader: " << rdd_is_feed_reader(dt) << endl;
  cout << "  mobile_app: " << rdd_is_mobile_app(dt) << endl;
  cout << "  media_player: " << rdd_is_media_player(dt) << endl;
  cout << "  browser: " << rdd_is_browser(dt) << endl;
  cout << "  library: " << rdd_is_library(dt) << endl;
  cout << "  desktop: " << rdd_is_desktop(dt) << endl;
  cout << "  console: " << rdd_is_console(dt) << endl;
  cout << "  car_browser: " << rdd_is_car_browser(dt) << endl;
  cout << "  camera: " << rdd_is_camera(dt) << endl;
  cout << "  portable_media_player: " << rdd_is_portable_media_player(dt) << endl;
  cout << "  notebook: " << rdd_is_notebook(dt) << endl;
  cout << "  television: " << rdd_is_television(dt) << endl;
  cout << "  smart_display: " << rdd_is_smart_display(dt) << endl;
  cout << "  feature_phone: " << rdd_is_feature_phone(dt) << endl;
  cout << "  smart_phone: " << rdd_is_smart_phone(dt) << endl;
  cout << "  tablet: " << rdd_is_tablet(dt) << endl;
  cout << "  phablet: " << rdd_is_phablet(dt) << endl;
  cout << "  smart_speaker: " << rdd_is_smart_speaker(dt) << endl;
  cout << "  peripheral: " << rdd_is_peripheral(dt) << endl;
  cout << "  wearable: " << rdd_is_wearable(dt) << endl;


  const RDDBot *bot = rdd_bot(dt);

  if (bot) {
    cout << "Bot:" << endl;

    char *bot_name = rdd_bot_name(bot);
    char *bot_category = rdd_bot_category(bot);
    char *bot_url = rdd_bot_url(bot);
    char *bot_producer_name = rdd_bot_producer_name(bot);
    char *bot_producer_url = rdd_bot_producer_url(bot);

    if (bot_name) {
      cout << "  bot_name: " << bot_name << endl;
      rdd_free_string(bot_name);
    }
    if (bot_category) {
      cout << "  bot_category: " << bot_category << endl;
      rdd_free_string(bot_category);
    }
    if (bot_url) {
      cout << "  bot_url: " << bot_url << endl;
      rdd_free_string(bot_url);
    }
    if (bot_producer_name) {
      cout << "  bot_producer_name: " << bot_producer_name << endl;
      rdd_free_string(bot_producer_name);
    }
    if (bot_producer_url) {
      cout << "  bot_producer_url: " << bot_producer_url << endl;
      rdd_free_string(bot_producer_url);
    }

  }

  const RDDClient *client = rdd_client(dt);

  if (client) {
    cout << "Client:" << endl;

    char *client_name = rdd_client_name(client);
    char *client_type = rdd_client_type(client);
    char *client_version = rdd_client_version(client);
    char *client_browser_engine = rdd_client_browser_engine(client);
    char *client_browser_version = rdd_client_browser_version(client);

    if (client_name) {
      cout << "  client_name: " << client_name << endl;
      rdd_free_string(client_name);
    }

    if (client_type) {
      cout << "  client_type: " << client_type << endl;
      rdd_free_string(client_type);
    }

    if (client_version) {
      cout << "  client_version: " << client_version << endl;
      rdd_free_string(client_version);
    }

    if (client_browser_engine) {
      cout << "  client_browser_engine: " << client_browser_engine << endl;
      rdd_free_string(client_browser_engine);
    }

    if (client_browser_version) {
      cout << "  client_browser_version: " << client_browser_version << endl;
      rdd_free_string(client_browser_version);
    }

  } else {
    cout << "Unrecognized client" << endl;
  }


  const RDDDevice *device = rdd_device(dt);
  if (device) {
    cout << "Device:" << endl;

    char *device_brand = rdd_device_brand(device);
    char *device_type = rdd_device_type(device);
    char *device_model = rdd_device_model(device);

    if (device_brand) {
      cout << "  device_brand: " << device_brand << endl;
      rdd_free_string(device_brand);
    }
 
    if (device_type) {
      cout << "  device_type: " << device_type << endl;
      rdd_free_string(device_type);
    }
   if (device_model) {
      cout << "  device_model: " << device_model << endl;
      rdd_free_string(device_model);
   }

  } else {
    cout << "Unrecognized device" << endl;
  }

  const RDDOS * os = rdd_os(dt);
  if (os) {
    cout << "OS:" << endl;

    char *os_name = rdd_os_name(os);
    char *os_version = rdd_os_version(os);
    char *os_platform = rdd_os_platform(os);
    char *os_family = rdd_os_family(os);

    if (os_name) {
      cout << "  os_name: " << os_name << endl;
      rdd_free_string(os_name);
    }

    if (os_version) {
      cout << "  os_version: " << os_version << endl;
      rdd_free_string(os_version);
    }

    if (os_platform) {
      cout << "  os_platform: " << os_platform << endl;
      rdd_free_string(os_platform);
    }

    if (os_family) {
      cout << "  os_family: " << os_family << endl;
      rdd_free_string(os_family);
    }
  }
  else {
    cout << "Unrecognized OS" << endl;
  }

  cout << endl;

  rdd_free_detection(dt);
}

