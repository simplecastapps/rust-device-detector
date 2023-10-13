#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

struct Bot;

struct Client;

struct Device;

struct OS;

template<typename T = void>
struct Option;

struct RDDDetection;

struct RDDDeviceDetector;

struct RDDClient {
  const Option<Client> *client;
};

struct RDDDevice {
  const Option<Device> *device;
};

struct RDDOS {
  const Option<OS> *os;
};

struct RDDBot {
  const Bot *bot;
};

extern "C" {

RDDDeviceDetector *rdd_device_detector_new(uint64_t cache_size);

RDDDetection *rdd_lookup(const RDDDeviceDetector *rdd, const char *ua);

const RDDClient *rdd_client(const RDDDetection *rdd);

char *rdd_client_name(const RDDClient *client);

char *rdd_client_type(const RDDClient *client);

char *rdd_client_version(const RDDClient *client);

char *rdd_client_browser_engine(const RDDClient *client);

char *rdd_client_browser_version(const RDDClient *client);

const RDDDevice *rdd_device(const RDDDetection *rdd);

char *rdd_device_brand(const RDDDevice *device);

char *rdd_device_model(const RDDDevice *device);

char *rdd_device_type(const RDDDevice *device);

const RDDOS *rdd_os(const RDDDetection *rdd);

char *rdd_os_name(const RDDOS *os);

char *rdd_os_version(const RDDOS *os);

char *rdd_os_platform(const RDDOS *os);

char *rdd_os_family(const RDDOS *os);

const RDDBot *rdd_bot(const RDDDetection *rdd);

char *rdd_bot_name(const RDDBot *bot);

char *rdd_bot_category(const RDDBot *bot);

char *rdd_bot_url(const RDDBot *bot);

char *rdd_bot_producer_name(const RDDBot *bot);

char *rdd_bot_producer_url(const RDDBot *bot);

bool rdd_is_bot(const RDDDetection *rdd);

bool rdd_is_mobile(const RDDDetection *rdd);

bool rdd_is_touch_enabled(const RDDDetection *rdd);

bool rdd_is_pim(const RDDDetection *rdd);

bool rdd_is_feed_reader(const RDDDetection *rdd);

bool rdd_is_mobile_app(const RDDDetection *rdd);

bool rdd_is_media_player(const RDDDetection *rdd);

bool rdd_is_browser(const RDDDetection *rdd);

bool rdd_is_library(const RDDDetection *rdd);

bool rdd_is_desktop(const RDDDetection *rdd);

bool rdd_is_console(const RDDDetection *rdd);

bool rdd_is_car_browser(const RDDDetection *rdd);

bool rdd_is_camera(const RDDDetection *rdd);

bool rdd_is_portable_media_player(const RDDDetection *rdd);

bool rdd_is_notebook(const RDDDetection *rdd);

bool rdd_is_television(const RDDDetection *rdd);

bool rdd_is_smart_display(const RDDDetection *rdd);

bool rdd_is_feature_phone(const RDDDetection *rdd);

bool rdd_is_smart_phone(const RDDDetection *rdd);

bool rdd_is_tablet(const RDDDetection *rdd);

bool rdd_is_phablet(const RDDDetection *rdd);

bool rdd_is_smart_speaker(const RDDDetection *rdd);

bool rdd_is_peripheral(const RDDDetection *rdd);

bool rdd_is_wearable(const RDDDetection *rdd);

void rdd_free_device_detector(RDDDeviceDetector *rdd);

void rdd_free_detection(RDDDetection *rdd);

void rdd_free_string(char *rdd);

} // extern "C"
