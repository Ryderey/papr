package com.papr.papr_mobile

import android.annotation.TargetApi
import android.content.Context
import android.os.Build
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Base64
import java.io.IOException
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

/** Stores encrypted AI credentials using an app-private Android Keystore key. */
internal class AiCredentialStore(context: Context) {
    private val preferences = context.applicationContext.getSharedPreferences(
        PREFERENCES_NAME,
        Context.MODE_PRIVATE,
    )

    fun set(credentialRef: String, secret: String) {
        requireSupported()
        setSupported(credentialRef, secret)
    }

    fun get(credentialRef: String): String? {
        requireSupported()
        return getSupported(credentialRef)
    }

    fun delete(credentialRef: String) {
        requireSupported()
        deleteSupported(credentialRef)
    }

    @TargetApi(Build.VERSION_CODES.M)
    private fun setSupported(credentialRef: String, secret: String) {
        val key = getOrCreateKey(credentialRef)
        val cipher = Cipher.getInstance(TRANSFORMATION).apply {
            init(Cipher.ENCRYPT_MODE, key)
        }
        val encrypted = cipher.doFinal(secret.toByteArray(Charsets.UTF_8))
        val value = listOf(cipher.iv, encrypted)
            .joinToString(SEPARATOR) { Base64.encodeToString(it, Base64.NO_WRAP) }
        if (!preferences.edit().putString(credentialRef, value).commit()) {
            throw IOException("Unable to persist encrypted credential")
        }
    }

    @TargetApi(Build.VERSION_CODES.M)
    private fun getSupported(credentialRef: String): String? {
        val value = preferences.getString(credentialRef, null) ?: return null
        val parts = value.split(SEPARATOR, limit = 2)
        if (parts.size != 2) throw IOException("Invalid encrypted credential")

        val keyStore = loadKeyStore()
        val key = keyStore.getKey(credentialRef, null) as? SecretKey
            ?: throw IOException("Credential key is unavailable")
        val cipher = Cipher.getInstance(TRANSFORMATION).apply {
            init(
                Cipher.DECRYPT_MODE,
                key,
                GCMParameterSpec(GCM_TAG_BITS, Base64.decode(parts[0], Base64.NO_WRAP)),
            )
        }
        val decrypted = cipher.doFinal(Base64.decode(parts[1], Base64.NO_WRAP))
        return decrypted.toString(Charsets.UTF_8)
    }

    @TargetApi(Build.VERSION_CODES.M)
    private fun deleteSupported(credentialRef: String) {
        if (!preferences.edit().remove(credentialRef).commit()) {
            throw IOException("Unable to remove encrypted credential")
        }
        val keyStore = loadKeyStore()
        if (keyStore.containsAlias(credentialRef)) {
            keyStore.deleteEntry(credentialRef)
        }
    }

    @TargetApi(Build.VERSION_CODES.M)
    private fun getOrCreateKey(credentialRef: String): SecretKey {
        val keyStore = loadKeyStore()
        (keyStore.getKey(credentialRef, null) as? SecretKey)?.let { return it }

        return KeyGenerator.getInstance(
            KeyProperties.KEY_ALGORITHM_AES,
            ANDROID_KEY_STORE,
        ).run {
            init(
                KeyGenParameterSpec.Builder(
                    credentialRef,
                    KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT,
                )
                    .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
                    .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
                    .build(),
            )
            generateKey()
        }
    }

    private fun loadKeyStore(): KeyStore = KeyStore.getInstance(ANDROID_KEY_STORE).apply {
        load(null)
    }

    private fun requireSupported() {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.M) {
            throw UnsupportedOperationException("Android Keystore AES requires API 23")
        }
    }

    companion object {
        private const val ANDROID_KEY_STORE = "AndroidKeyStore"
        private const val PREFERENCES_NAME = "papr_ai_credentials"
        private const val TRANSFORMATION = "AES/GCM/NoPadding"
        private const val GCM_TAG_BITS = 128
        private const val SEPARATOR = "."
        private val REFERENCE_PATTERN = Regex("^papr\\.ai\\.[A-Za-z0-9_-]{1,80}$")

        fun isValidReference(value: String?): Boolean =
            value != null && REFERENCE_PATTERN.matches(value)

        fun isValidSecret(value: String?): Boolean =
            value != null && value.isNotBlank() && value.length <= 8_192
    }
}
