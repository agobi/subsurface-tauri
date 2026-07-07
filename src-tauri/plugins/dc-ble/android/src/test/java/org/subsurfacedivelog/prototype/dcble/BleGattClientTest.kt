// AI-generated (Claude)
package org.subsurfacedivelog.prototype.dcble

import android.bluetooth.BluetoothGattCharacteristic
import android.bluetooth.BluetoothGattDescriptor
import android.bluetooth.BluetoothGattService
import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import java.util.UUID

@RunWith(RobolectricTestRunner::class)
class BleGattClientTest {
    private fun serviceWith(uuid: String, hasWrite: Boolean, hasNotify: Boolean): BluetoothGattService {
        val svc = BluetoothGattService(UUID.fromString(uuid), BluetoothGattService.SERVICE_TYPE_PRIMARY)
        if (hasWrite) {
            svc.addCharacteristic(
                BluetoothGattCharacteristic(
                    UUID.randomUUID(),
                    BluetoothGattCharacteristic.PROPERTY_WRITE,
                    BluetoothGattCharacteristic.PERMISSION_WRITE,
                )
            )
        }
        if (hasNotify) {
            svc.addCharacteristic(
                BluetoothGattCharacteristic(
                    UUID.randomUUID(),
                    BluetoothGattCharacteristic.PROPERTY_NOTIFY,
                    BluetoothGattCharacteristic.PERMISSION_READ,
                )
            )
        }
        return svc
    }

    @Test
    fun picksKnownSerialServiceOverHeuristicFallback() {
        val shearwater = serviceWith("fe25c237-0ece-443c-b0aa-e02033e7029d", hasWrite = true, hasNotify = true)
        val other = serviceWith("11111111-1111-1111-1111-111111111111", hasWrite = true, hasNotify = true)
        val picked = findPreferredService(listOf(other, shearwater))
        assertEquals(shearwater.uuid, picked?.uuid)
    }

    @Test
    fun skipsUpgradeServiceInHeuristicFallback() {
        val upgrade = serviceWith("00001530-1212-efde-1523-785feabcd123", hasWrite = true, hasNotify = true)
        val normal = serviceWith("22222222-2222-2222-2222-222222222222", hasWrite = true, hasNotify = true)
        val picked = findPreferredService(listOf(upgrade, normal))
        assertEquals(normal.uuid, picked?.uuid)
    }

    @Test
    fun returnsNullWhenNoServiceHasBothWriteAndNotify() {
        val writeOnly = serviceWith("33333333-3333-3333-3333-333333333333", hasWrite = true, hasNotify = false)
        assertNull(findPreferredService(listOf(writeOnly)))
    }

    @Test
    fun cccdValueForNotifyOnlyUsesEnableNotification() {
        assertArrayEquals(
            BluetoothGattDescriptor.ENABLE_NOTIFICATION_VALUE,
            cccdValueFor(BluetoothGattCharacteristic.PROPERTY_NOTIFY),
        )
    }

    @Test
    fun cccdValueForIndicateOnlyUsesEnableIndication() {
        assertArrayEquals(
            BluetoothGattDescriptor.ENABLE_INDICATION_VALUE,
            cccdValueFor(BluetoothGattCharacteristic.PROPERTY_INDICATE),
        )
    }

    @Test
    fun cccdValueForBothNotifyAndIndicatePrefersNotification() {
        assertArrayEquals(
            BluetoothGattDescriptor.ENABLE_NOTIFICATION_VALUE,
            cccdValueFor(BluetoothGattCharacteristic.PROPERTY_NOTIFY or BluetoothGattCharacteristic.PROPERTY_INDICATE),
        )
    }
}
