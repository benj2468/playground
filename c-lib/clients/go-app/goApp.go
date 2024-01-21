package main

// #cgo LDFLAGS: -lhellow
// #include "hellow.h"
import "C"
import "os"

func main() {

	ctx := C.Hellow_new()

	C.Hellow_set_name(ctx, C.CString(os.Args[1]))

	C.Hellow_say_hi(ctx)
}
