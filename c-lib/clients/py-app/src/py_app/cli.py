import hellow
import sys

def main() -> int:
    ctx = hellow.Hellow_new()

    if len(sys.argv) > 1:
        hellow.Hellow_set_prefix(ctx, sys.argv[1])
    
    for arg in sys.argv[2:]:
        hellow.Hellow_announce(ctx, arg)

    

if __name__ == "__main__":
    main()