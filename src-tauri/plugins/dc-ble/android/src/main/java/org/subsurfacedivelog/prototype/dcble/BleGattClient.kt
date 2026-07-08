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
import android.util.Log
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.withTimeout

private const val TAG = "DcBleGatt"

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

/** Picks the CCCD value to enable delivery on a notify/indicate characteristic: an
 * indicate-only characteristic (PROPERTY_INDICATE without PROPERTY_NOTIFY) requires
 * ENABLE_INDICATION_VALUE — writing ENABLE_NOTIFICATION_VALUE to it silently gets no
 * data. Pure function — no Android framework calls beyond flag checks. */
fun cccdValueFor(properties: Int): ByteArray =
    if (properties and BluetoothGattCharacteristic.PROPERTY_NOTIFY != 0) {
        BluetoothGattDescriptor.ENABLE_NOTIFICATION_VALUE
    } else {
        BluetoothGattDescriptor.ENABLE_INDICATION_VALUE
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
                // Android defaults to a 23-byte ATT MTU (20 usable bytes per
                // notification/write) unless explicitly negotiated higher — unlike
                // macOS/CoreBluetooth, which negotiates a larger MTU automatically.
                // Without this, any response longer than 20 bytes gets truncated
                // mid-frame and the peer is left waiting for a continuation that
                // never arrives. Service discovery proceeds from onMtuChanged.
                Log.d(TAG, "connected, requesting MTU 247 and high-priority connection")
                g.requestMtu(247)
                // The download protocol is a strict one-block-request-at-a-time
                // exchange (libdivecomputer's Shearwater driver), so throughput is
                // bounded by round-trip latency, not payload size. Android's default
                // connection interval is conservative (~49ms here); requesting HIGH
                // priority asks for the shortest interval the peripheral supports.
                g.requestConnectionPriority(BluetoothGatt.CONNECTION_PRIORITY_HIGH)
            } else if (newState == BluetoothProfile.STATE_DISCONNECTED) {
                connectDeferred?.completeExceptionally(BleConnectException("disconnected during setup"))
                onDisconnected()
            }
        }

        override fun onMtuChanged(g: BluetoothGatt, mtu: Int, status: Int) {
            Log.d(TAG, "onMtuChanged: mtu=$mtu status=$status")
            g.discoverServices()
        }

        override fun onServicesDiscovered(g: BluetoothGatt, status: Int) {
            Log.d(TAG, "onServicesDiscovered: status=$status, services=${g.services.map { it.uuid }}")
            if (status != BluetoothGatt.GATT_SUCCESS) {
                connectDeferred?.completeExceptionally(BleConnectException("service discovery failed: status=$status"))
                return
            }
            val service = findPreferredService(g.services)
            Log.d(TAG, "findPreferredService picked: ${service?.uuid}")
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
            Log.d(TAG, "write char=${write?.uuid} props=${write?.properties}, notify char=${notify?.uuid} props=${notify?.properties}")
            if (write == null || notify == null) {
                connectDeferred?.completeExceptionally(BleConnectException("service missing write or notify characteristic"))
                return
            }
            writeChar = write
            writeWithResponse = write.properties and BluetoothGattCharacteristic.PROPERTY_WRITE_NO_RESPONSE == 0
            val notifSet = g.setCharacteristicNotification(notify, true)
            val cccd = notify.getDescriptor(BleConstants.CCCD_UUID)
            Log.d(TAG, "setCharacteristicNotification=$notifSet, cccd=${cccd?.uuid}")
            if (cccd == null) {
                connectDeferred?.completeExceptionally(BleConnectException("notify characteristic has no CCCD descriptor"))
                return
            }
            val cccdValue = cccdValueFor(notify.properties)
            Log.d(TAG, "writing CCCD value=${cccdValue.joinToString(",")}")
            @Suppress("DEPRECATION")
            cccd.value = cccdValue
            @Suppress("DEPRECATION")
            val started = g.writeDescriptor(cccd)
            Log.d(TAG, "writeDescriptor() call returned $started")
        }

        override fun onDescriptorWrite(g: BluetoothGatt, descriptor: BluetoothGattDescriptor, status: Int) {
            Log.d(TAG, "onDescriptorWrite: uuid=${descriptor.uuid} status=$status")
            if (status == BluetoothGatt.GATT_SUCCESS) {
                connectDeferred?.complete(Unit)
            } else {
                connectDeferred?.completeExceptionally(BleConnectException("CCCD write failed: status=$status"))
            }
        }

        @Suppress("DEPRECATION")
        override fun onCharacteristicChanged(g: BluetoothGatt, characteristic: BluetoothGattCharacteristic) {
            val bytes = characteristic.value ?: ByteArray(0)
            Log.d(TAG, "onCharacteristicChanged: uuid=${characteristic.uuid} bytes=${bytes.size} data=${bytes.joinToString(",")}")
            onNotification(bytes)
        }

        override fun onCharacteristicWrite(g: BluetoothGatt, characteristic: BluetoothGattCharacteristic, status: Int) {
            Log.d(TAG, "onCharacteristicWrite: uuid=${characteristic.uuid} status=$status")
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
        Log.d(TAG, "write() bytes=${bytes.size} data=${bytes.joinToString(",")} writeWithResponse=$writeWithResponse")
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
        val started = g.writeCharacteristic(char)
        Log.d(TAG, "writeCharacteristic() call returned $started")
        if (!started) throw BleConnectException("writeCharacteristic returned false")
        try {
            withTimeout(12_000) { deferred.await() }
            Log.d(TAG, "write() completed successfully")
        } catch (e: Exception) {
            Log.d(TAG, "write() failed/timed out: ${e.message}")
            throw e
        }
    }

    @SuppressLint("MissingPermission")
    fun disconnect() {
        gatt?.disconnect()
        gatt?.close()
        gatt = null
    }
}
