package com.papr.papr_mobile

import android.app.Service
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.Handler
import android.os.Looper
import androidx.media3.common.AudioAttributes
import androidx.media3.common.C
import androidx.media3.common.MediaItem
import androidx.media3.common.MediaMetadata
import androidx.media3.common.PlaybackException
import androidx.media3.common.Player
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.session.MediaSession
import androidx.media3.session.MediaSessionService
import java.util.concurrent.CopyOnWriteArraySet

class PlaybackService : MediaSessionService() {
    private lateinit var player: ExoPlayer
    private var mediaSession: MediaSession? = null
    private val mainHandler = Handler(Looper.getMainLooper())
    private var lastError: String? = null
    private val positionTicker = object : Runnable {
        override fun run() {
            if (!player.isPlaying) return
            publishState()
            mainHandler.postDelayed(this, POSITION_UPDATE_INTERVAL_MS)
        }
    }

    override fun onCreate() {
        super.onCreate()
        player = ExoPlayer.Builder(this)
            .setSeekBackIncrementMs(SEEK_BACK_MS)
            .setSeekForwardIncrementMs(SEEK_FORWARD_MS)
            .setAudioAttributes(
                AudioAttributes.Builder()
                    .setUsage(C.USAGE_MEDIA)
                    .setContentType(C.AUDIO_CONTENT_TYPE_SPEECH)
                    .build(),
                true,
            )
            .setHandleAudioBecomingNoisy(true)
            .build()
        player.addListener(object : Player.Listener {
            override fun onEvents(player: Player, events: Player.Events) {
                publishState()
                updatePositionTicker()
            }

            override fun onIsPlayingChanged(isPlaying: Boolean) = updatePositionTicker()

            override fun onPlayerError(error: PlaybackException) {
                lastError = errorCode(error)
                publishState()
            }
        })
        mediaSession = MediaSession.Builder(this, player).build()
        publishState()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            ACTION_START -> startPlayback(intent)
            ACTION_PLAY -> player.play()
            ACTION_PAUSE -> player.pause()
            ACTION_SEEK -> player.seekTo(intent.getLongExtra(EXTRA_POSITION_MS, 0).coerceAtLeast(0))
            ACTION_SKIP_BACK -> player.seekTo((player.currentPosition - SEEK_BACK_MS).coerceAtLeast(0))
            ACTION_SKIP_FORWARD -> {
                val target = player.currentPosition + SEEK_FORWARD_MS
                player.seekTo(if (player.duration > 0) target.coerceAtMost(player.duration) else target)
            }
            ACTION_SPEED -> intent.getFloatExtra(EXTRA_SPEED, 1f).takeIf { it in MIN_SPEED..MAX_SPEED }
                ?.let { player.setPlaybackSpeed(it) }
            ACTION_STOP -> {
                player.stop()
                player.clearMediaItems()
                lastError = null
                publishState()
                stopSelf()
            }
        }
        return Service.START_NOT_STICKY
    }

    override fun onGetSession(controllerInfo: MediaSession.ControllerInfo): MediaSession? = mediaSession

    override fun onDestroy() {
        mainHandler.removeCallbacks(positionTicker)
        mediaSession?.run {
            player.release()
            release()
        }
        mediaSession = null
        super.onDestroy()
    }

    private fun startPlayback(intent: Intent) {
        val url = intent.getStringExtra(EXTRA_URL)
        val uri = url?.let(Uri::parse)
        if (uri == null || uri.scheme !in setOf("http", "https") || uri.host.isNullOrBlank()) {
            lastError = "invalidPlaybackUrl"
            publishState()
            stopSelf()
            return
        }
        val title = intent.getStringExtra(EXTRA_TITLE).orEmpty().ifBlank { "Podcast" }
        val source = intent.getStringExtra(EXTRA_SOURCE).orEmpty()
        lastError = null
        val mediaItem = MediaItem.Builder()
            .setMediaId(intent.getStringExtra(EXTRA_MEDIA_ID).orEmpty())
            .setUri(uri)
            .setMediaMetadata(
                MediaMetadata.Builder()
                    .setTitle(title)
                    .setArtist(source)
                    .build(),
            )
            .build()
        player.setMediaItem(mediaItem)
        player.prepare()
        player.play()
    }

    private fun updatePositionTicker() {
        mainHandler.removeCallbacks(positionTicker)
        if (player.isPlaying) {
            mainHandler.post(positionTicker)
        }
    }

    private fun publishState() {
        state = PlaybackState(
            mediaId = player.currentMediaItem?.mediaId.orEmpty(),
            title = player.mediaMetadata.title?.toString().orEmpty(),
            source = player.mediaMetadata.artist?.toString().orEmpty(),
            durationMs = player.duration.coerceAtLeast(0),
            positionMs = player.currentPosition.coerceAtLeast(0),
            speed = player.playbackParameters.speed,
            playing = player.isPlaying,
            buffering = player.playbackState == Player.STATE_BUFFERING,
            error = lastError,
        )
        listeners.forEach { it(state) }
    }

    companion object {
        private const val ACTION_START = "com.papr.papr_mobile.playback.START"
        private const val ACTION_PLAY = "com.papr.papr_mobile.playback.PLAY"
        private const val ACTION_PAUSE = "com.papr.papr_mobile.playback.PAUSE"
        private const val ACTION_SEEK = "com.papr.papr_mobile.playback.SEEK"
        private const val ACTION_SKIP_BACK = "com.papr.papr_mobile.playback.SKIP_BACK"
        private const val ACTION_SKIP_FORWARD = "com.papr.papr_mobile.playback.SKIP_FORWARD"
        private const val ACTION_SPEED = "com.papr.papr_mobile.playback.SPEED"
        private const val ACTION_STOP = "com.papr.papr_mobile.playback.STOP"
        private const val EXTRA_MEDIA_ID = "mediaId"
        private const val EXTRA_URL = "url"
        private const val EXTRA_TITLE = "title"
        private const val EXTRA_SOURCE = "source"
        private const val EXTRA_POSITION_MS = "positionMs"
        private const val EXTRA_SPEED = "speed"
        private const val SEEK_BACK_MS = 15_000L
        private const val SEEK_FORWARD_MS = 30_000L
        private const val MIN_SPEED = 0.75f
        private const val MAX_SPEED = 2f
        private const val POSITION_UPDATE_INTERVAL_MS = 500L
        private val listeners = CopyOnWriteArraySet<(PlaybackState) -> Unit>()

        @Volatile
        private var state = PlaybackState.empty()

        fun addStateListener(listener: (PlaybackState) -> Unit): () -> Unit {
            listeners += listener
            listener(state)
            return { listeners -= listener }
        }

        fun currentState(): Map<String, Any?> = state.toMap()

        fun start(context: Context, mediaId: String, url: String, title: String, source: String) {
            command(context, ACTION_START, Intent().apply {
                putExtra(EXTRA_MEDIA_ID, mediaId)
                putExtra(EXTRA_URL, url)
                putExtra(EXTRA_TITLE, title)
                putExtra(EXTRA_SOURCE, source)
            }, foreground = true)
        }

        fun play(context: Context) = command(context, ACTION_PLAY)
        fun pause(context: Context) = command(context, ACTION_PAUSE)
        fun skipBack(context: Context) = command(context, ACTION_SKIP_BACK)
        fun skipForward(context: Context) = command(context, ACTION_SKIP_FORWARD)
        fun stop(context: Context) = command(context, ACTION_STOP)
        fun seekTo(context: Context, positionMs: Long) =
            command(context, ACTION_SEEK, seekArguments(positionMs))
        fun setSpeed(context: Context, speed: Float) =
            command(context, ACTION_SPEED, speedArguments(speed))

        fun command(context: Context, command: String, arguments: Intent = Intent()) {
            command(context, command, arguments, foreground = false)
        }

        private fun command(context: Context, action: String, arguments: Intent, foreground: Boolean) {
            val intent = Intent(context, PlaybackService::class.java).apply {
                this.action = action
                putExtras(arguments)
            }
            if (foreground && Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                context.startForegroundService(intent)
            } else {
                context.startService(intent)
            }
        }

        fun seekArguments(positionMs: Long) = Intent().putExtra(EXTRA_POSITION_MS, positionMs)
        fun speedArguments(speed: Float) = Intent().putExtra(EXTRA_SPEED, speed)

        private fun errorCode(error: PlaybackException): String = when (error.errorCode) {
            PlaybackException.ERROR_CODE_IO_NETWORK_CONNECTION_FAILED,
            PlaybackException.ERROR_CODE_IO_NETWORK_CONNECTION_TIMEOUT,
            PlaybackException.ERROR_CODE_IO_BAD_HTTP_STATUS,
            PlaybackException.ERROR_CODE_IO_FILE_NOT_FOUND,
            -> "playbackNetwork"
            else -> "playbackFailed"
        }
    }
}

data class PlaybackState(
    val mediaId: String,
    val title: String,
    val source: String,
    val durationMs: Long,
    val positionMs: Long,
    val speed: Float,
    val playing: Boolean,
    val buffering: Boolean,
    val error: String?,
) {
    fun toMap(): Map<String, Any?> = mapOf(
        "mediaId" to mediaId,
        "title" to title,
        "source" to source,
        "durationMs" to durationMs,
        "positionMs" to positionMs,
        "speed" to speed.toDouble(),
        "playing" to playing,
        "buffering" to buffering,
        "error" to error,
    )

    companion object {
        fun empty() = PlaybackState("", "", "", 0, 0, 1f, false, false, null)
    }
}
