#include <stdio.h>

#include "adk.h"

int main() {
    Scenario *scenario = scenario_new();
    Entity *surrogate = entity_new();

    Lla *lla = lla_new();
    lla_with_lat(lla, 33.687607);
    lla_with_lon(lla, -117.782648);
    lla_with_alt(lla, 500.0);
    entity_with_position(surrogate, lla);

    char *str[11];
    for (size_t i = 0; i < 5; i++)
    {
        sprintf(str, "surrogate-%d", i);
        entity_with_name(surrogate, str);
        scenario_with_entity(scenario, surrogate);
    }

    scenario_debug(scenario);
}