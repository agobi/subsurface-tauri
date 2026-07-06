// AI-generated (Claude)
package org.subsurfacedivelog.prototype.dcble

import android.annotation.SuppressLint
import android.bluetooth.BluetoothDevice
import android.bluetooth.BluetoothGatt
import android.bluetooth.BluetoothGattCallback
import android.bluetooth.BluetoothGattCharacteristic
import android.bluetooth.BluetoothGattDescriptor
import android.bluetooth.BluetoothGattService
import android.bluetooth.BluetoothManager
import android.bluetooth.BluetoothProfile
import android.content.Context
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.withTimeout

/** Selects the preferred GATT service: known serial-service UUIDs first (priority
 * order, ported from `dc/transport/ble.rs`'s `find_preferred_service`), then a
 * heuristic fallback (first non-upgrade service exposing both write and
 * notify/read characteristics). Pure function — no Android framework calls beyond
 * plain data access, so it's directly unit-testable. */
fun findPreferredService(services: List<BluetoothGattService>): BluetoothGattService? {
    for (known in BleConstants.SERIAL_SERVICES) {
        val svc = services.find { it.uuid.toString().lowercase() == known && hasReadAndWrite(it) }
        if (svc != null) return svc
    }
    return services.find { svc ->
        val u = svc.uuid.toString().lowercase()
        u !in BleConstants.UPGRADE_SERVICES && hasReadAndWrite(svc)
    }
}

fun hasReadAndWrite(svc: BluetoothGattService): Boolean {
    val hasWrite = svc.characteristics.any {
        it.properties and (BluetoothGattCharacteristic.PROPERTY_WRITE or BluetoothGattCharacteristic.PROPERTY_WRITE_NO_RESPONSE) != 0
    }
    val hasNotify = svc.characteristics.any {
        it.properties and (BluetoothGattCharacteristic.PROPERTY_NOTIFY or BluetoothGattCharacteristic.PROPERTY_INDICATE or BluetoothGattCharacteristic.PROPERTY_READ) != 0
    }
    return hasWrite && hasNotify
}

class BleConnectException(message: String) : Exception(message)

/** Bridges one BLE dive-computer connection's async `BluetoothGatt` callbacks to a
 * suspend-function API for [DcBlePlugin]. One instance per connection; not reused
 * across reconnects. */
class BleGattClient(
    private val context: Context,
    private val onNotification: (ByteArray) -> Unit,
    private val onDisconnected: () -> Unit,
) {
    private var gatt: BluetoothGatt? = null
    private var writeChar: BluetoothGattCharacteristic? = null
    private var writeWithResponse = true
    private var connectDeferred: CompletableDeferred<Unit>? = null
    private var writeDeferred: CompletableDeferred<Unit>? = null

    private val callback = object : BluetoothGattCallback() {
        override fun onConnectionStateChange(g: BluetoothGatt, status: Int, newState: Int) {
            if (newState == BluetoothProfile.STATE_CONNECTED) {
                g.discoverServices()
            } else if (newState == BluetoothProfile.STATE_DISCONNECTED) {
                connectDeferred?.completeExceptionally(BleConnectException("disconnected during setup"))
                onDisconnected()
            }
        }

        override fun onServicesDiscovered(g: BluetoothGatt, status: Int) {
            if (status != BluetoothGatt.GATT_SUCCESS) {
                connectDeferred?.completeExceptionally(BleConnectException("service discovery failed: status=$status"))
                return
            }
            val service = findPreferredService(g.services)
            if (service == null) {
                connectDeferred?.completeExceptionally(BleConnectException("no suitable BLE serial service found"))
                return
            }
            val write = service.characteristics.find {
                it.properties and (BluetoothGattCharacteristic.PROPERTY_WRITE or BluetoothGattCharacteristic.PROPERTY_WRITE_NO_RESPONSE) != 0
            }
            val notify = service.characteristics.find {
                it.properties and (BluetoothGattCharacteristic.PROPERTY_NOTIFY or BluetoothGattCharacteristic.PROPERTY_INDICATE) != 0
            }
            if (write == null || notify == null) {
                connectDeferred?.completeExceptionally(BleConnectException("service missing write or notify characteristic"))
                return
            }
            writeChar = write
            writeWithResponse = write.properties and BluetoothGattCharacteristic.PROPERTY_WRITE_NO_RESPONSE == 0
            g.setCharacteristicNotification(notify, true)
            val cccd = notify.getDescriptor(BleConstants.CCCD_UUID)
            if (cccd == null) {
                connectDeferred?.completeExceptionally(BleConnectException("notify characteristic has no CCCD descriptor"))
                return
            }
            @Suppress("DEPRECATION")
            cccd.value = BluetoothGattDescriptor.ENABLE_NOTIFICATION_VALUE
            @Suppress("DEPRECATION")
            g.writeDescriptor(cccd)
        }

        override fun onDescriptorWrite(g: BluetoothGatt, descriptor: BluetoothGattDescriptor, status: Int) {
            if (status == BluetoothGatt.GATT_SUCCESS) {
                connectDeferred?.complete(Unit)
            } else {
                connectDeferred?.completeExceptionally(BleConnectException("CCCD write failed: status=$status"))
            }
        }

        @Suppress("DEPRECATION")
        override fun onCharacteristicChanged(g: BluetoothGatt, characteristic: BluetoothGattCharacteristic) {
            onNotification(characteristic.value ?: ByteArray(0))
        }

        override fun onCharacteristicWrite(g: BluetoothGatt, characteristic: BluetoothGattCharacteristic, status: Int) {
            if (status == BluetoothGatt.GATT_SUCCESS) {
                writeDeferred?.complete(Unit)
            } else {
                writeDeferred?.completeExceptionally(BleConnectException("characteristic write failed: status=$status"))
            }
        }
    }

    @SuppressLint("MissingPermission")
    suspend fun connect(address: String) {
        val adapter = (context.getSystemService(Context.BLUETOOTH_SERVICE) as BluetoothManager).adapter
            ?: throw BleConnectException("no Bluetooth adapter")
        val device: BluetoothDevice = adapter.getRemoteDevice(address)
        val deferred = CompletableDeferred<Unit>()
        connectDeferred = deferred
        gatt = device.connectGatt(context, false, callback, BluetoothDevice.TRANSPORT_LE)
        withTimeout(15_000) { deferred.await() }
    }

    @SuppressLint("MissingPermission")
    suspend fun write(bytes: ByteArray) {
        val g = gatt ?: throw BleConnectException("not connected")
        val char = writeChar ?: throw BleConnectException("not connected")
        val deferred = CompletableDeferred<Unit>()
        writeDeferred = deferred
        @Suppress("DEPRECATION")
        char.value = bytes
        @Suppress("DEPRECATION")
        char.writeType = if (writeWithResponse) {
            BluetoothGattCharacteristic.WRITE_TYPE_DEFAULT
        } else {
            BluetoothGattCharacteristic.WRITE_TYPE_NO_RESPONSE
        }
        @Suppress("DEPRECATION")
        if (!g.writeCharacteristic(char)) throw BleConnectException("writeCharacteristic returned false")
        withTimeout(12_000) { deferred.await() }
    }

    @SuppressLint("MissingPermission")
    fun disconnect() {
        gatt?.disconnect()
        gatt?.close()
        gatt = null
    }
}
