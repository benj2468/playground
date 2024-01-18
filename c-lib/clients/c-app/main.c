#include <stdlib.h>
#include <stdio.h>
#include "hellow.h"

int main(void)
{
    HellowContext *ctx = NULL;

    ctx = new_hellow();

    say_hi(ctx);
}