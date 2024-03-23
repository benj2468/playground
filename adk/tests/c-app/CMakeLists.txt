cmake_minimum_required(VERSION 3.10)

project(c-app)

add_executable(c-app main.c)

target_link_libraries(c-app adk)

install(TARGETS c-app)