cmake_minimum_required(VERSION 3.20)
project(autocore_starter_cpp LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

include_directories(
    ${CMAKE_CURRENT_SOURCE_DIR}/../../crates/autocore-sdk/include
)

add_executable(mission mission.cpp)

# Gerçek zamanlı pthread ve sistem kütüphaneleri
find_package(Threads REQUIRED)
target_link_libraries(mission PRIVATE Threads::Threads)
