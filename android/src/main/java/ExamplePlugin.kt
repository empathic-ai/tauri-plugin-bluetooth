package com.plugin.bluetooth

import android.app.Activity
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke

import android.bluetooth.BluetoothAdapter
import android.bluetooth.BluetoothDevice
import android.content.Intent
import androidx.appcompat.app.AppCompatActivity
import android.os.Bundle

@InvokeArg
class PingArgs {
  var value: String? = null
}

@TauriPlugin
class ExamplePlugin(private val activity: Activity): Plugin(activity) {
    private val implementation = Example()

    private lateinit var bluetoothAdapter: BluetoothAdapter

    private val receiver = object : BroadcastReceiver() {
        override fun onReceive(context: Context, intent: Intent) {
            val action = intent.action
            if (BluetoothDevice.ACTION_FOUND == action) {
                val device: BluetoothDevice = intent.getParcelableExtra(BluetoothDevice.EXTRA_DEVICE)
                // Add the device to a list or display its name and address
            }
        }
    }

    @Command
    fun ping(invoke: Invoke) {
      
        bluetoothAdapter = BluetoothAdapter.getDefaultAdapter()

        if (bluetoothAdapter == null) {
            Toast.makeText(this, "Bluetooth not supported", Toast.LENGTH_SHORT).show()
            activity.finish()
        }

        if (!bluetoothAdapter.isEnabled) {
          val enableBluetoothIntent = Intent(BluetoothAdapter.ACTION_REQUEST_ENABLE)
          startActivityForResult(enableBluetoothIntent, REQUEST_ENABLE_BT)
        }

        companion object {
            private const val REQUEST_ENABLE_BT = 1
        }

        //val discoverDevicesIntent = IntentFilter(BluetoothDevice.ACTION_FOUND)
        //registerReceiver(receiver, discoverDevicesIntent)
        //bluetoothAdapter.startDiscovery()

        Set<BluetoothDevice> pairedDevices = bluetoothAdapter.getBondedDevices();
        if (pairedDevices.size() > 0) {
            for (BluetoothDevice d: pairedDevices) {
                String deviceName = d.getName();
                String macAddress = d.getAddress();
                Log.i(LOGTAG, "paired device: " + deviceName + " at " + macAddress);
                // do what you need/want this these list items
            }
        }

        val args = invoke.parseArgs(PingArgs::class.java)

        val ret = JSObject()
        ret.put("value", implementation.pong(args.value ?: "default value :("))
        invoke.resolve(ret)
    }
    
}
