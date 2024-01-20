#include <stdlib.h>
#include <stdio.h>
#include "hellow.h"

int main(int argc, char *argv[])
{
    HellowContext *ctx = Hellow_new();

    Hellow_set_name(ctx, argv[1]);

    Hellow_say_hi(ctx);
}