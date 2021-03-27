cmake_minimum_required(VERSION 3.17)
project("Embedded Rust Wrapper")


add_custom_target(
        Build
        COMMAND cargo build
        BYPRODUCTS ${CMAKE_CURRENT_SOURCE_DIR}/target/thumbv7em-none-eabihf/debug/scheduler_demo
)

add_executable(Executable IMPORTED GLOBAL)
set_target_properties(
        Executable PROPERTIES
        IMPORTED_LOCATION ${CMAKE_CURRENT_SOURCE_DIR}/target/thumbv7em-none-eabihf/debug/scheduler_demo
)
add_dependencies(Executable Build)