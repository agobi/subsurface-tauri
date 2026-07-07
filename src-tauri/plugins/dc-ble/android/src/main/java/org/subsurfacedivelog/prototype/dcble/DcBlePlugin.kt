// AI-generated (Claude)
package org.subsurfacedivelog.prototype.dcble

import android.Manifest
import android.annotation.SuppressLint
import android.app.Activity
import android.bluetooth.BluetoothManager
import android.bluetooth.le.ScanCallback
import android.bluetooth.le.ScanResult
import android.content.Intent
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Build
import android.provider.Settings
import androidx.core.content.ContextCompat
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.Permission
import app.tauri.annotation.PermissionCallback
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Channel
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

private const val PERMISSION_ALIAS_BLE = "ble"

private fun requiredPermissions(): Array<String> =
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
        arrayOf(Manifest.permission.BLUETOOTH_SCAN, Manifest.permission.BLUETOOTH_CONNECT)
    } else {
        arrayOf(Manifest.permission.ACCESS_FINE_LOCATION)
    }

@InvokeArg
class ConnectArgs {
    lateinit var address: String
    lateinit var channel: Channel
}

@InvokeArg
class WriteArgs {
    var bytes: List<Int> = emptyList()
}

@InvokeArg
class ScanArgs {
    lateinit var vendor: String
    lateinit var model: String
    lateinit var channel: Channel
}

@TauriPlugin(
    permissions = [
        Permission(
            strings = [
                Manifest.permission.BLUETOOTH_SCAN,
                Manifest.permission.BLUETOOTH_CONNECT,
                Manifest.permission.ACCESS_FINE_LOCATION,
            ],
            alias = PERMISSION_ALIAS_BLE,
        )
    ]
)
class DcBlePlugin(private val activity: Activity) : Plugin(activity) {
    private val scope = CoroutineScope(Dispatchers.IO)
    private var client: BleGattClient? = null

    private fun hasBlePermissions(): Boolean = requiredPermissions().all {
        ContextCompat.checkSelfPermission(activity, it) == PackageManager.PERMISSION_GRANTED
    }

    @Command
    fun connect(invoke: Invoke) {
        val args = invoke.parseArgs(ConnectArgs::class.java)
        val notifyChannel = args.channel
        if (!hasBlePermissions()) {
            invoke.reject("PermissionDenied")
            return
        }
        scope.launch {
            try {
                val c = BleGattClient(
                    activity,
                    onNotification = { bytes ->
                        // Jackson serializes ByteArray as base64 by default, which
                        // wouldn't deserialize into Rust's Vec<u8> (a JSON number
                        // array) — send a plain Int list instead so both sides agree.
                        notifyChannel.sendObject(mapOf(
                            "type" to "data",
                            "bytes" to bytes.map { b -> b.toInt() and 0xFF },
                        ))
                    },
                    onDisconnected = { notifyChannel.sendObject(mapOf("type" to "disconnected")) },
                )
                c.connect(args.address)
                client = c
                invoke.resolve(JSObject())
            } catch (e: Exception) {
                invoke.reject(e.message ?: "BLE connect failed")
            }
        }
    }

    @Command
    fun write(invoke: Invoke) {
        val args = invoke.parseArgs(WriteArgs::class.java)
        val c = client
        if (c == null) {
            invoke.reject("not connected")
            return
        }
        val bytes = ByteArray(args.bytes.size) { i -> args.bytes[i].toByte() }
        scope.launch {
            try {
                c.write(bytes)
                invoke.resolve(JSObject())
            } catch (e: Exception) {
                invoke.reject(e.message ?: "BLE write failed")
            }
        }
    }

    @Command
    fun disconnect(invoke: Invoke) {
        client?.disconnect()
        client = null
        invoke.resolve(JSObject())
    }

    @Command
    fun ensurePermissions(invoke: Invoke) {
        if (hasBlePermissions()) {
            val ret = JSObject()
            ret.put("granted", true)
            invoke.resolve(ret)
            return
        }
        requestPermissionForAlias(PERMISSION_ALIAS_BLE, invoke, "blePermissionCallback")
    }

    @PermissionCallback
    fun blePermissionCallback(invoke: Invoke) {
        val ret = JSObject()
        ret.put("granted", hasBlePermissions())
        invoke.resolve(ret)
    }

    @Command
    fun openAppSettings(invoke: Invoke) {
        val intent = Intent(
            Settings.ACTION_APPLICATION_DETAILS_SETTINGS,
            Uri.fromParts("package", activity.packageName, null),
        )
        activity.startActivity(intent)
        invoke.resolve(JSObject())
    }

    @SuppressLint("MissingPermission")
    @Command
    fun scan(invoke: Invoke) {
        val args = invoke.parseArgs(ScanArgs::class.java)
        val resultsChannel = args.channel
        if (!hasBlePermissions()) {
            invoke.reject("PermissionDenied")
            return
        }
        val manager = activity.getSystemService(BluetoothManager::class.java)
        val scanner = manager?.adapter?.bluetoothLeScanner
        if (scanner == null) {
            invoke.reject("no BLE adapter available")
            return
        }
        val seen = mutableSetOf<String>()
        val callback = object : ScanCallback() {
            override fun onScanResult(callbackType: Int, result: ScanResult) {
                val name = result.device.name ?: result.scanRecord?.deviceName ?: return
                val address = result.device.address
                if (seen.add(address)) {
                    resultsChannel.sendObject(mapOf("name" to name, "address" to address))
                }
            }
        }
        scanner.startScan(callback)
        invoke.resolve(JSObject())
        // Matches desktop's fixed 10s scan window (dc/commands.rs's 20 x 500ms poll loop).
        scope.launch {
            delay(10_000)
            scanner.stopScan(callback)
        }
    }
}
