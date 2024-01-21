import hellow
import sys

def main() -> int:
    ctx = hellow.Hellow_new()

    hellow.Hellow_set_name(ctx, sys.argv[1])

    hellow.Hellow_say_hi(ctx)

if __name__ == "__main__":
    main()