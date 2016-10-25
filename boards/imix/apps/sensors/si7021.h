#ifndef _SI7021_H
#define _SI7021_H

#include <tock.h>

#ifdef __cplusplus
extern "C" {
#endif

#define ERR_NONE 0

int si7021_subscribe(subscribe_cb callback, void* callback_args);
int si7021_sample();

#ifdef __cplusplus
}
#endif

#endif // _SI7021_H
