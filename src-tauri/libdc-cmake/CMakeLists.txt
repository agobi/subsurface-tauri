# AI-generated (Claude)
# CMakeLists.txt for libdivecomputer (Subsurface-DS9 branch).
# The upstream source uses autotools; this file provides cmake support
# for Rust build.rs integration via the `cmake` crate.
# Source files live in the sibling `libdc/` git submodule; this file is
# tracked in the parent repo so it survives fresh clones.

cmake_minimum_required(VERSION 3.15)

project(divecomputer VERSION 0.10.0 LANGUAGES C)

# Generate version.h from version.h.in
set(DC_VERSION "0.10.0-devel-Subsurface-DS9")
set(DC_VERSION_MAJOR 0)
set(DC_VERSION_MINOR 10)
set(DC_VERSION_MICRO 0)

configure_file(
    "${CMAKE_CURRENT_SOURCE_DIR}/../libdc/include/libdivecomputer/version.h.in"
    "${CMAKE_CURRENT_BINARY_DIR}/include/libdivecomputer/version.h"
    @ONLY
)

# All source files (derived from src/Makefile.am in ../libdc/)
set(SOURCES
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/version.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/descriptor.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/iostream.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/iterator.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/common.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/context.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/device.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/parser.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/datetime.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/timer.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/suunto_common.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/suunto_common2.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/suunto_solution.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/suunto_solution_parser.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/suunto_eon.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/suunto_eon_parser.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/suunto_vyper.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/suunto_vyper_parser.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/suunto_vyper2.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/suunto_d9.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/suunto_d9_parser.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/suunto_eonsteel.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/suunto_eonsteel_parser.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/reefnet_sensus.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/reefnet_sensus_parser.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/reefnet_sensuspro.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/reefnet_sensuspro_parser.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/reefnet_sensusultra.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/reefnet_sensusultra_parser.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/uwatec_aladin.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/uwatec_memomouse.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/uwatec_memomouse_parser.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/uwatec_smart.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/uwatec_smart_parser.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/oceanic_common.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/oceanic_atom2.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/oceanic_atom2_parser.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/oceanic_veo250.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/oceanic_veo250_parser.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/oceanic_vtpro.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/oceanic_vtpro_parser.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/pelagic_i330r.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/mares_common.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/mares_nemo.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/mares_nemo_parser.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/mares_puck.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/mares_darwin.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/mares_darwin_parser.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/mares_iconhd.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/mares_iconhd_parser.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/ihex.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/hw_ostc.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/hw_ostc_parser.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/hw_frog.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/hw_ostc3.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/aes.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/cressi_edy.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/cressi_edy_parser.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/cressi_leonardo.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/cressi_leonardo_parser.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/cressi_goa.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/cressi_goa_parser.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/zeagle_n2ition3.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/atomics_cobalt.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/atomics_cobalt_parser.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/shearwater_common.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/shearwater_predator.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/shearwater_predator_parser.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/shearwater_petrel.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/diverite_nitekq.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/diverite_nitekq_parser.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/citizen_aqualand.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/citizen_aqualand_parser.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/divesystem_idive.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/divesystem_idive_parser.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/platform.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/ringbuffer.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/rbstream.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/checksum.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/array.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/buffer.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/cochran_commander.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/cochran_commander_parser.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/tecdiving_divecomputereu.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/tecdiving_divecomputereu_parser.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/mclean_extreme.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/mclean_extreme_parser.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/liquivision_lynx.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/liquivision_lynx_parser.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/sporasub_sp2.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/sporasub_sp2_parser.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/deepsix_excursion.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/deepsix_excursion_parser.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/seac_screen_common.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/seac_screen.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/seac_screen_parser.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/deepblu_cosmiq.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/deepblu_cosmiq_parser.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/oceans_s1_common.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/oceans_s1.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/oceans_s1_parser.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/divesoft_freedom.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/divesoft_freedom_parser.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/halcyon_symbios.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/halcyon_symbios_parser.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/hdlc.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/packet.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/socket.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/irda.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/usb.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/usbhid.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/ble.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/bluetooth.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/custom.c
    # Subsurface-specific additions not yet in upstream
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/usb_storage.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/field-cache.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/garmin.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/garmin_parser.c
)

# Platform-specific serial implementation
if(WIN32)
    list(APPEND SOURCES ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/serial_win32.c)
else()
    list(APPEND SOURCES ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src/serial_posix.c)
endif()

add_library(divecomputer STATIC ${SOURCES})

# Include paths
target_include_directories(divecomputer PUBLIC
    "${CMAKE_CURRENT_SOURCE_DIR}/../libdc/include"
    "${CMAKE_CURRENT_BINARY_DIR}/include"
)
target_include_directories(divecomputer PRIVATE
    "${CMAKE_CURRENT_SOURCE_DIR}/../libdc/src"
)

# Compile definitions (replaces autotools config.h)
target_compile_definitions(divecomputer PRIVATE
    ENABLE_LOGGING=1
)

# pthread is available on macOS and Linux
if(NOT WIN32)
    find_package(Threads REQUIRED)
    target_link_libraries(divecomputer PUBLIC Threads::Threads)
    target_compile_definitions(divecomputer PRIVATE HAVE_PTHREAD_H=1)
endif()

# Suppress warnings from the library sources
target_compile_options(divecomputer PRIVATE
    $<$<C_COMPILER_ID:GNU,Clang,AppleClang>:-Wno-unused-parameter -Wno-sign-compare -Wno-implicit-function-declaration>
)

# Platform-specific linking
if(APPLE)
    target_link_libraries(divecomputer PUBLIC "-framework IOKit" "-framework CoreFoundation")
elseif(WIN32)
    target_link_libraries(divecomputer PUBLIC ws2_32 setupapi hid)
elseif(UNIX)
    target_link_libraries(divecomputer PUBLIC m)
endif()

# Install static lib so the cmake crate can find it
install(TARGETS divecomputer ARCHIVE DESTINATION lib)
install(DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/../libdc/include/ DESTINATION include)
install(FILES "${CMAKE_CURRENT_BINARY_DIR}/include/libdivecomputer/version.h"
    DESTINATION "include/libdivecomputer"
)
