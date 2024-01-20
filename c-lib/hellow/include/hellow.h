#ifndef HELLOW
#define HELLOW

typedef struct _rust_context HellowContext;

HellowContext *Hellow_new();

int Hellow_set_name(HellowContext *ctx, const char *name);

void Hellow_say_hi(HellowContext *ctx);

#endif