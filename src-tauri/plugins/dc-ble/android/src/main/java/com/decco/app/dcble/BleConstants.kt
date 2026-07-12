// AI-generated (Claude)
package com.decco.app.dcble

import java.util.UUID

/** Ported from `src-tauri/src/dc/transport/ble.rs`'s `SERIAL_SERVICES`/`UPGRADE_SERVICES` —
 * keep both lists in sync. Source: Subsurface's `qt-ble.cpp`. */
object BleConstants {
    val SERIAL_SERVICES: List<String> = listOf(
        "0000fefb-0000-1000-8000-00805f9b34fb", // Heinrichs-Weikamp (Telit/Stollmann)
        "2456e1b9-26e2-8f83-e744-f34f01e9d701", // Heinrichs-Weikamp (U-Blox)
        "544e326b-5b72-c6b0-1c46-41c1bc448118", // Mares BlueLink Pro
        "98ae7120-e62e-11e3-badd-0002a5d5c51b", // Suunto (EON Steel/Core, G5)
        "cb3c4555-d670-4670-bc20-b61dbc851e9a", // Pelagic (i770R, i200C, Pro Plus X, Geo 4.0)
        "ca7b0001-f785-4c38-b599-c7c5fbadb034", // Pelagic (i330R, DSX)
        "fdcdeaaa-295d-470e-bf15-04217b7aa0a0", // ScubaPro (G2, G3)
        "fe25c237-0ece-443c-b0aa-e02033e7029d", // Shearwater (Perdix/Teric/Peregrine/Tern)
        "0000fcef-0000-1000-8000-00805f9b34fb", // Divesoft
        "6e400001-b5a3-f393-e0a9-e50e24dc10b8", // Cressi
        "6e400001-b5a3-f393-e0a9-e50e24dcca9e", // Nordic Semi UART
        "00000001-8c3b-4f2c-a59e-8c08224f3253", // Halcyon Symbios
        "84968ffe-d26d-478a-b953-5010bcf58bca", // Seac
    )

    val UPGRADE_SERVICES: Set<String> = setOf(
        "00001530-1212-efde-1523-785feabcd123",
        "9e5d1e47-5c13-43a0-8635-82ad38a1386f",
        "a86abc2d-d44c-442e-99f7-80059a873e36",
    )

    val CCCD_UUID: UUID = UUID.fromString("00002902-0000-1000-8000-00805f9b34fb")
}
