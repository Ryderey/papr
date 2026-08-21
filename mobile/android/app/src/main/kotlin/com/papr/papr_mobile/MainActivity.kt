package com.papr.papr_mobile

import android.app.Activity
import android.content.Intent
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.embedding.android.FlutterActivity
import io.flutter.plugin.common.MethodChannel
import java.io.IOException

class MainActivity : FlutterActivity() {
    private var channel: MethodChannel? = null
    private var pendingResult: MethodChannel.Result? = null
    private var pendingExportText: String? = null

    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
        channel = MethodChannel(
            flutterEngine.dartExecutor.binaryMessenger,
            PLATFORM_CHANNEL,
        ).also { methodChannel ->
            methodChannel.setMethodCallHandler { call, result ->
                when (call.method) {
                    "getInitialDeepLink" -> result.success(intent?.dataString)
                    "openOpmlDocument" -> openOpmlDocument(result)
                    "saveOpmlDocument" -> {
                        val text = call.argument<String>("text")
                        if (text == null) {
                            result.error("invalidOpml", "Missing OPML text", null)
                        } else {
                            saveOpmlDocument(text, result)
                        }
                    }
                    else -> result.notImplemented()
                }
            }
        }
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        intent.dataString?.let { channel?.invokeMethod("deepLink", it) }
    }

    @Deprecated("Deprecated in Android")
    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
        val result = pendingResult ?: return
        when (requestCode) {
            OPEN_OPML_REQUEST -> {
                pendingResult = null
                if (resultCode != Activity.RESULT_OK || data?.data == null) {
                    result.success(null)
                    return
                }
                try {
                    val stream = contentResolver.openInputStream(data.data!!)
                        ?: throw IOException("Unable to open selected document")
                    val text = stream.bufferedReader().use { it.readText() }
                    result.success(text)
                } catch (error: IOException) {
                    result.error("documentReadFailed", error.message, null)
                }
            }
            SAVE_OPML_REQUEST -> {
                pendingResult = null
                val text = pendingExportText
                pendingExportText = null
                if (resultCode != Activity.RESULT_OK || data?.data == null) {
                    result.success(false)
                    return
                }
                try {
                    val stream = contentResolver.openOutputStream(data.data!!)
                        ?: throw IOException("Unable to open created document")
                    stream.bufferedWriter().use {
                        it.write(text.orEmpty())
                    }
                    result.success(true)
                } catch (error: IOException) {
                    result.error("documentWriteFailed", error.message, null)
                }
            }
        }
    }

    private fun openOpmlDocument(result: MethodChannel.Result) {
        if (!beginDocumentRequest(result)) return
        val intent = Intent(Intent.ACTION_OPEN_DOCUMENT).apply {
            addCategory(Intent.CATEGORY_OPENABLE)
            type = "*/*"
            putExtra(
                Intent.EXTRA_MIME_TYPES,
                arrayOf("text/xml", "application/xml", "text/x-opml", "application/octet-stream"),
            )
        }
        startActivityForResult(intent, OPEN_OPML_REQUEST)
    }

    private fun saveOpmlDocument(text: String, result: MethodChannel.Result) {
        if (!beginDocumentRequest(result)) return
        pendingExportText = text
        val intent = Intent(Intent.ACTION_CREATE_DOCUMENT).apply {
            addCategory(Intent.CATEGORY_OPENABLE)
            type = "text/x-opml"
            putExtra(Intent.EXTRA_TITLE, "papr-subscriptions.opml")
        }
        startActivityForResult(intent, SAVE_OPML_REQUEST)
    }

    private fun beginDocumentRequest(result: MethodChannel.Result): Boolean {
        if (pendingResult != null) {
            result.error("documentPickerBusy", "Another document request is active", null)
            return false
        }
        pendingResult = result
        return true
    }

    companion object {
        private const val PLATFORM_CHANNEL = "com.papr.papr_mobile/platform"
        private const val OPEN_OPML_REQUEST = 4101
        private const val SAVE_OPML_REQUEST = 4102
    }
}
