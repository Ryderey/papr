package com.papr.papr_mobile

import io.flutter.embedding.engine.plugins.FlutterPlugin
import io.flutter.plugin.common.MethodCall
import io.flutter.plugin.common.MethodChannel

/** Registers Keystore access in both the UI engine and Workmanager's engine. */
class SyncCredentialPlugin : FlutterPlugin, MethodChannel.MethodCallHandler {
    private var channel: MethodChannel? = null
    private var store: AiCredentialStore? = null

    override fun onAttachedToEngine(binding: FlutterPlugin.FlutterPluginBinding) {
        store = AiCredentialStore(binding.applicationContext)
        channel = MethodChannel(
            binding.binaryMessenger,
            "com.papr.papr_mobile/sync_credentials",
        ).also { it.setMethodCallHandler(this) }
    }

    override fun onDetachedFromEngine(binding: FlutterPlugin.FlutterPluginBinding) {
        channel?.setMethodCallHandler(null)
        channel = null
        store = null
    }

    override fun onMethodCall(call: MethodCall, result: MethodChannel.Result) {
        val credentialRef = call.argument<String>("credentialRef")
        if (!AiCredentialStore.isValidSyncReference(credentialRef)) {
            result.error("invalidSyncCredential", null, null)
            return
        }
        try {
            when (call.method) {
                "setSyncCredential" -> {
                    val secret = call.argument<String>("secret")
                    if (!AiCredentialStore.isValidSecret(secret)) {
                        result.error("invalidSyncCredential", null, null)
                        return
                    }
                    store!!.set(credentialRef!!, secret!!)
                    result.success(true)
                }
                "getSyncCredential" -> result.success(store!!.get(credentialRef!!))
                "deleteSyncCredential" -> {
                    store!!.delete(credentialRef!!)
                    result.success(true)
                }
                else -> result.notImplemented()
            }
        } catch (_: Exception) {
            val code = when (call.method) {
                "setSyncCredential" -> "credentialWriteFailed"
                "getSyncCredential" -> "credentialReadFailed"
                else -> "credentialDeleteFailed"
            }
            result.error(code, null, null)
        }
    }
}
