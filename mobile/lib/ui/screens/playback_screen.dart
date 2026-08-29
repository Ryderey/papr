import 'dart:async';

import 'package:flutter/material.dart';

import '../../l10n/l10n.dart';
import '../../services/platform_service.dart';

class PlaybackScreen extends StatefulWidget {
  final PlaybackState initialState;

  const PlaybackScreen({super.key, required this.initialState});

  @override
  State<PlaybackScreen> createState() => _PlaybackScreenState();
}

class _PlaybackScreenState extends State<PlaybackScreen> {
  late PlaybackState _state = widget.initialState;
  StreamSubscription<PlaybackState>? _subscription;
  double? _dragPosition;

  @override
  void initState() {
    super.initState();
    _subscription = platformService.playbackStates.listen((state) {
      if (mounted) setState(() => _state = state);
    });
    platformService.getPlaybackState().then((state) {
      if (mounted) setState(() => _state = state);
    });
  }

  @override
  void dispose() {
    _subscription?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final state = _state;
    final active = state.mediaId.isNotEmpty;
    final duration = state.durationMs > 0 ? state.durationMs : 1;
    final position =
        _dragPosition ?? state.positionMs.clamp(0, duration).toDouble();
    return Scaffold(
      appBar: AppBar(title: Text(context.l10n.nowPlaying)),
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: active
              ? Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    const Spacer(),
                    Icon(
                      Icons.graphic_eq,
                      size: 120,
                      color: Theme.of(context).colorScheme.primary,
                    ),
                    const SizedBox(height: 32),
                    Text(
                      state.title,
                      maxLines: 2,
                      overflow: TextOverflow.ellipsis,
                      textAlign: TextAlign.center,
                      style: Theme.of(context).textTheme.headlineSmall,
                    ),
                    if (state.source.isNotEmpty) ...[
                      const SizedBox(height: 8),
                      Text(
                        state.source,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        textAlign: TextAlign.center,
                        style: Theme.of(context).textTheme.bodyLarge,
                      ),
                    ],
                    if (state.error != null) ...[
                      const SizedBox(height: 16),
                      Text(
                        _errorText(context, state.error!),
                        textAlign: TextAlign.center,
                        style: TextStyle(
                          color: Theme.of(context).colorScheme.error,
                        ),
                      ),
                    ],
                    const SizedBox(height: 24),
                    Slider(
                      value: position,
                      max: duration.toDouble(),
                      onChanged: state.durationMs > 0
                          ? (value) => setState(() => _dragPosition = value)
                          : null,
                      onChangeEnd: state.durationMs > 0
                          ? (value) async {
                              setState(() => _dragPosition = null);
                              await platformService.playbackCommand(
                                'seekTo',
                                positionMs: value.round(),
                              );
                            }
                          : null,
                    ),
                    Row(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      children: [
                        Text(_formatDuration(state.positionMs)),
                        Text(_formatDuration(state.durationMs)),
                      ],
                    ),
                    const SizedBox(height: 12),
                    Row(
                      mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                      children: [
                        IconButton(
                          tooltip: context.l10n.rewind15,
                          iconSize: 32,
                          onPressed: () =>
                              platformService.playbackCommand('skipBack'),
                          icon: const Icon(Icons.replay_10),
                        ),
                        FilledButton.tonal(
                          onPressed: () => platformService.playbackCommand(
                            state.playing ? 'pause' : 'play',
                          ),
                          style: FilledButton.styleFrom(
                            shape: const CircleBorder(),
                            padding: const EdgeInsets.all(20),
                          ),
                          child: Icon(
                            state.playing ? Icons.pause : Icons.play_arrow,
                            size: 38,
                          ),
                        ),
                        IconButton(
                          tooltip: context.l10n.forward30,
                          iconSize: 32,
                          onPressed: () =>
                              platformService.playbackCommand('skipForward'),
                          icon: const Icon(Icons.forward_30),
                        ),
                      ],
                    ),
                    const SizedBox(height: 24),
                    Row(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      children: [
                        Text(context.l10n.playbackSpeed),
                        PopupMenuButton<double>(
                          onSelected: (speed) => platformService
                              .playbackCommand('setSpeed', speed: speed),
                          itemBuilder: (_) => [
                            for (final speed in const [
                              0.75,
                              1.0,
                              1.25,
                              1.5,
                              2.0
                            ])
                              PopupMenuItem(
                                value: speed,
                                child: Text('$speed×'),
                              ),
                          ],
                          child: Chip(label: Text('$state.speed×')),
                        ),
                      ],
                    ),
                    const Spacer(),
                    OutlinedButton.icon(
                      onPressed: () async {
                        await platformService.playbackCommand('stop');
                        if (context.mounted) Navigator.pop(context);
                      },
                      icon: const Icon(Icons.stop_circle_outlined),
                      label: Text(context.l10n.stopPlayback),
                    ),
                  ],
                )
              : Center(child: Text(context.l10n.noActivePlayback)),
        ),
      ),
    );
  }
}

String _formatDuration(int milliseconds) {
  final seconds = Duration(milliseconds: milliseconds).inSeconds;
  final minutes = seconds ~/ 60;
  return '$minutes:${(seconds % 60).toString().padLeft(2, '0')}';
}

String _errorText(BuildContext context, String error) => switch (error) {
      'invalidPlaybackUrl' => context.l10n.errorInvalidPlaybackUrl,
      'playbackNetwork' => context.l10n.errorPlaybackNetwork,
      'playbackUnavailable' => context.l10n.errorPlaybackUnavailable,
      _ => context.l10n.errorPlaybackFailed,
    };
