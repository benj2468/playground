#include "adk.h"

int main() {
    Scenario *scenario = scenario_new();

    Lla *lla = lla_new();
    lla_with_lat(lla, 33.687607);
    lla_with_lon(lla, -117.782648);
    lla_with_alt(lla, 500.0);

    Entity *surrogate = entity_new();
    entity_with_name(surrogate, "Surrogate 1");
    entity_with_position(surrogate, lla);

    scenario_with_entity(scenario, surrogate);

    lla_with_alt(lla, 400);
    entity_with_name(surrogate, "Surrogate 2");
    entity_with_position(surrogate, lla);
    scenario_with_entity(scenario, surrogate);

    scenario_debug(scenario);
}