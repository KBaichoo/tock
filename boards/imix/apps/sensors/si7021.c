#include "si7021.h"

// enable TMP006, take a single reading, disable TMP006, callback with value
int si7021_subscribe(subscribe_cb callback, void* callback_args) {
    // subscribe to a single temp value callback
    //  also enables the temperature sensor for the duration of one sample
    return subscribe(5, 0, callback, callback_args);
}

int si7021_sample() {
  return command(5, 0, 0);
}

