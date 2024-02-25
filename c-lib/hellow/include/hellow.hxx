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
intptr_t Hellow_set_prefix(HellowContext *ctx, const char *prefix);

intptr_t Hellow_announce(const HellowContext *ctx, const char *name);

} // extern "C"

} // namespace hellow
