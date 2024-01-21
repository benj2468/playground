#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct HellowContext HellowContext;

struct HellowContext *Hellow_new(void);

/**
 * # Safety
 *
 * `name` must be a null terminated string that has at most isize::MAX bytes
 */
intptr_t Hellow_set_name(struct HellowContext *ctx, const char *name);

void Hellow_say_hi(const struct HellowContext *ctx);
