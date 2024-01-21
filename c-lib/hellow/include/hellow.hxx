#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

namespace hellow {

template<typename T = void>
struct Box;

struct HellowContext;

extern "C" {

Box<HellowContext> Hellow_new();

/// # Safety
///
/// `name` must be a null terminated string that has at most isize::MAX bytes
intptr_t Hellow_set_name(HellowContext *ctx, const char *name);

void Hellow_say_hi(const HellowContext *ctx);

} // extern "C"

} // namespace hellow
