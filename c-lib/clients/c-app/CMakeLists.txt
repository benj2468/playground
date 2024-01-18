cmake_minimum_required(VERSION 3.10)

project(hellow-c)

add_executable(hellow-c main.c)

target_link_libraries(hellow-c hellow)

install(TARGETS hellow-c)