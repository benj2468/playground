from adk import *

def main() -> int:
    scenario = scenario_new()
    surrogate = entity_new()

    lla = lla_new()
    lla_with_lat(lla, 33.687607)
    lla_with_lon(lla, -117.782648)
    lla_with_alt(lla, 500.0)
    entity_with_position(surrogate, lla)

    for i in range(5):
        entity_with_name(surrogate, f"surrogate-{i}")
        scenario_with_entity(scenario, surrogate)

    scenario_debug(scenario)    
    

if __name__ == "__main__":
    main()