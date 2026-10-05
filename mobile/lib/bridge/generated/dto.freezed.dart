// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'dto.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

/// @nodoc
mixin _$AiStreamEvent {
  String get requestId => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String requestId, String text) delta,
    required TResult Function(String requestId, int completed, int total)
        progress,
    required TResult Function(String requestId) completed,
    required TResult Function(String requestId, String code) error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String requestId, String text)? delta,
    TResult? Function(String requestId, int completed, int total)? progress,
    TResult? Function(String requestId)? completed,
    TResult? Function(String requestId, String code)? error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String requestId, String text)? delta,
    TResult Function(String requestId, int completed, int total)? progress,
    TResult Function(String requestId)? completed,
    TResult Function(String requestId, String code)? error,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(AiStreamEvent_Delta value) delta,
    required TResult Function(AiStreamEvent_Progress value) progress,
    required TResult Function(AiStreamEvent_Completed value) completed,
    required TResult Function(AiStreamEvent_Error value) error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(AiStreamEvent_Delta value)? delta,
    TResult? Function(AiStreamEvent_Progress value)? progress,
    TResult? Function(AiStreamEvent_Completed value)? completed,
    TResult? Function(AiStreamEvent_Error value)? error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(AiStreamEvent_Delta value)? delta,
    TResult Function(AiStreamEvent_Progress value)? progress,
    TResult Function(AiStreamEvent_Completed value)? completed,
    TResult Function(AiStreamEvent_Error value)? error,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;

  /// Create a copy of AiStreamEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $AiStreamEventCopyWith<AiStreamEvent> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $AiStreamEventCopyWith<$Res> {
  factory $AiStreamEventCopyWith(
          AiStreamEvent value, $Res Function(AiStreamEvent) then) =
      _$AiStreamEventCopyWithImpl<$Res, AiStreamEvent>;
  @useResult
  $Res call({String requestId});
}

/// @nodoc
class _$AiStreamEventCopyWithImpl<$Res, $Val extends AiStreamEvent>
    implements $AiStreamEventCopyWith<$Res> {
  _$AiStreamEventCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of AiStreamEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? requestId = null,
  }) {
    return _then(_value.copyWith(
      requestId: null == requestId
          ? _value.requestId
          : requestId // ignore: cast_nullable_to_non_nullable
              as String,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$AiStreamEvent_DeltaImplCopyWith<$Res>
    implements $AiStreamEventCopyWith<$Res> {
  factory _$$AiStreamEvent_DeltaImplCopyWith(_$AiStreamEvent_DeltaImpl value,
          $Res Function(_$AiStreamEvent_DeltaImpl) then) =
      __$$AiStreamEvent_DeltaImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String requestId, String text});
}

/// @nodoc
class __$$AiStreamEvent_DeltaImplCopyWithImpl<$Res>
    extends _$AiStreamEventCopyWithImpl<$Res, _$AiStreamEvent_DeltaImpl>
    implements _$$AiStreamEvent_DeltaImplCopyWith<$Res> {
  __$$AiStreamEvent_DeltaImplCopyWithImpl(_$AiStreamEvent_DeltaImpl _value,
      $Res Function(_$AiStreamEvent_DeltaImpl) _then)
      : super(_value, _then);

  /// Create a copy of AiStreamEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? requestId = null,
    Object? text = null,
  }) {
    return _then(_$AiStreamEvent_DeltaImpl(
      requestId: null == requestId
          ? _value.requestId
          : requestId // ignore: cast_nullable_to_non_nullable
              as String,
      text: null == text
          ? _value.text
          : text // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$AiStreamEvent_DeltaImpl extends AiStreamEvent_Delta {
  const _$AiStreamEvent_DeltaImpl({required this.requestId, required this.text})
      : super._();

  @override
  final String requestId;
  @override
  final String text;

  @override
  String toString() {
    return 'AiStreamEvent.delta(requestId: $requestId, text: $text)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$AiStreamEvent_DeltaImpl &&
            (identical(other.requestId, requestId) ||
                other.requestId == requestId) &&
            (identical(other.text, text) || other.text == text));
  }

  @override
  int get hashCode => Object.hash(runtimeType, requestId, text);

  /// Create a copy of AiStreamEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$AiStreamEvent_DeltaImplCopyWith<_$AiStreamEvent_DeltaImpl> get copyWith =>
      __$$AiStreamEvent_DeltaImplCopyWithImpl<_$AiStreamEvent_DeltaImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String requestId, String text) delta,
    required TResult Function(String requestId, int completed, int total)
        progress,
    required TResult Function(String requestId) completed,
    required TResult Function(String requestId, String code) error,
  }) {
    return delta(requestId, text);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String requestId, String text)? delta,
    TResult? Function(String requestId, int completed, int total)? progress,
    TResult? Function(String requestId)? completed,
    TResult? Function(String requestId, String code)? error,
  }) {
    return delta?.call(requestId, text);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String requestId, String text)? delta,
    TResult Function(String requestId, int completed, int total)? progress,
    TResult Function(String requestId)? completed,
    TResult Function(String requestId, String code)? error,
    required TResult orElse(),
  }) {
    if (delta != null) {
      return delta(requestId, text);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(AiStreamEvent_Delta value) delta,
    required TResult Function(AiStreamEvent_Progress value) progress,
    required TResult Function(AiStreamEvent_Completed value) completed,
    required TResult Function(AiStreamEvent_Error value) error,
  }) {
    return delta(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(AiStreamEvent_Delta value)? delta,
    TResult? Function(AiStreamEvent_Progress value)? progress,
    TResult? Function(AiStreamEvent_Completed value)? completed,
    TResult? Function(AiStreamEvent_Error value)? error,
  }) {
    return delta?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(AiStreamEvent_Delta value)? delta,
    TResult Function(AiStreamEvent_Progress value)? progress,
    TResult Function(AiStreamEvent_Completed value)? completed,
    TResult Function(AiStreamEvent_Error value)? error,
    required TResult orElse(),
  }) {
    if (delta != null) {
      return delta(this);
    }
    return orElse();
  }
}

abstract class AiStreamEvent_Delta extends AiStreamEvent {
  const factory AiStreamEvent_Delta(
      {required final String requestId,
      required final String text}) = _$AiStreamEvent_DeltaImpl;
  const AiStreamEvent_Delta._() : super._();

  @override
  String get requestId;
  String get text;

  /// Create a copy of AiStreamEvent
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$AiStreamEvent_DeltaImplCopyWith<_$AiStreamEvent_DeltaImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$AiStreamEvent_ProgressImplCopyWith<$Res>
    implements $AiStreamEventCopyWith<$Res> {
  factory _$$AiStreamEvent_ProgressImplCopyWith(
          _$AiStreamEvent_ProgressImpl value,
          $Res Function(_$AiStreamEvent_ProgressImpl) then) =
      __$$AiStreamEvent_ProgressImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String requestId, int completed, int total});
}

/// @nodoc
class __$$AiStreamEvent_ProgressImplCopyWithImpl<$Res>
    extends _$AiStreamEventCopyWithImpl<$Res, _$AiStreamEvent_ProgressImpl>
    implements _$$AiStreamEvent_ProgressImplCopyWith<$Res> {
  __$$AiStreamEvent_ProgressImplCopyWithImpl(
      _$AiStreamEvent_ProgressImpl _value,
      $Res Function(_$AiStreamEvent_ProgressImpl) _then)
      : super(_value, _then);

  /// Create a copy of AiStreamEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? requestId = null,
    Object? completed = null,
    Object? total = null,
  }) {
    return _then(_$AiStreamEvent_ProgressImpl(
      requestId: null == requestId
          ? _value.requestId
          : requestId // ignore: cast_nullable_to_non_nullable
              as String,
      completed: null == completed
          ? _value.completed
          : completed // ignore: cast_nullable_to_non_nullable
              as int,
      total: null == total
          ? _value.total
          : total // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc

class _$AiStreamEvent_ProgressImpl extends AiStreamEvent_Progress {
  const _$AiStreamEvent_ProgressImpl(
      {required this.requestId, required this.completed, required this.total})
      : super._();

  @override
  final String requestId;
  @override
  final int completed;
  @override
  final int total;

  @override
  String toString() {
    return 'AiStreamEvent.progress(requestId: $requestId, completed: $completed, total: $total)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$AiStreamEvent_ProgressImpl &&
            (identical(other.requestId, requestId) ||
                other.requestId == requestId) &&
            (identical(other.completed, completed) ||
                other.completed == completed) &&
            (identical(other.total, total) || other.total == total));
  }

  @override
  int get hashCode => Object.hash(runtimeType, requestId, completed, total);

  /// Create a copy of AiStreamEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$AiStreamEvent_ProgressImplCopyWith<_$AiStreamEvent_ProgressImpl>
      get copyWith => __$$AiStreamEvent_ProgressImplCopyWithImpl<
          _$AiStreamEvent_ProgressImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String requestId, String text) delta,
    required TResult Function(String requestId, int completed, int total)
        progress,
    required TResult Function(String requestId) completed,
    required TResult Function(String requestId, String code) error,
  }) {
    return progress(requestId, this.completed, total);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String requestId, String text)? delta,
    TResult? Function(String requestId, int completed, int total)? progress,
    TResult? Function(String requestId)? completed,
    TResult? Function(String requestId, String code)? error,
  }) {
    return progress?.call(requestId, this.completed, total);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String requestId, String text)? delta,
    TResult Function(String requestId, int completed, int total)? progress,
    TResult Function(String requestId)? completed,
    TResult Function(String requestId, String code)? error,
    required TResult orElse(),
  }) {
    if (progress != null) {
      return progress(requestId, this.completed, total);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(AiStreamEvent_Delta value) delta,
    required TResult Function(AiStreamEvent_Progress value) progress,
    required TResult Function(AiStreamEvent_Completed value) completed,
    required TResult Function(AiStreamEvent_Error value) error,
  }) {
    return progress(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(AiStreamEvent_Delta value)? delta,
    TResult? Function(AiStreamEvent_Progress value)? progress,
    TResult? Function(AiStreamEvent_Completed value)? completed,
    TResult? Function(AiStreamEvent_Error value)? error,
  }) {
    return progress?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(AiStreamEvent_Delta value)? delta,
    TResult Function(AiStreamEvent_Progress value)? progress,
    TResult Function(AiStreamEvent_Completed value)? completed,
    TResult Function(AiStreamEvent_Error value)? error,
    required TResult orElse(),
  }) {
    if (progress != null) {
      return progress(this);
    }
    return orElse();
  }
}

abstract class AiStreamEvent_Progress extends AiStreamEvent {
  const factory AiStreamEvent_Progress(
      {required final String requestId,
      required final int completed,
      required final int total}) = _$AiStreamEvent_ProgressImpl;
  const AiStreamEvent_Progress._() : super._();

  @override
  String get requestId;
  int get completed;
  int get total;

  /// Create a copy of AiStreamEvent
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$AiStreamEvent_ProgressImplCopyWith<_$AiStreamEvent_ProgressImpl>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$AiStreamEvent_CompletedImplCopyWith<$Res>
    implements $AiStreamEventCopyWith<$Res> {
  factory _$$AiStreamEvent_CompletedImplCopyWith(
          _$AiStreamEvent_CompletedImpl value,
          $Res Function(_$AiStreamEvent_CompletedImpl) then) =
      __$$AiStreamEvent_CompletedImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String requestId});
}

/// @nodoc
class __$$AiStreamEvent_CompletedImplCopyWithImpl<$Res>
    extends _$AiStreamEventCopyWithImpl<$Res, _$AiStreamEvent_CompletedImpl>
    implements _$$AiStreamEvent_CompletedImplCopyWith<$Res> {
  __$$AiStreamEvent_CompletedImplCopyWithImpl(
      _$AiStreamEvent_CompletedImpl _value,
      $Res Function(_$AiStreamEvent_CompletedImpl) _then)
      : super(_value, _then);

  /// Create a copy of AiStreamEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? requestId = null,
  }) {
    return _then(_$AiStreamEvent_CompletedImpl(
      requestId: null == requestId
          ? _value.requestId
          : requestId // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$AiStreamEvent_CompletedImpl extends AiStreamEvent_Completed {
  const _$AiStreamEvent_CompletedImpl({required this.requestId}) : super._();

  @override
  final String requestId;

  @override
  String toString() {
    return 'AiStreamEvent.completed(requestId: $requestId)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$AiStreamEvent_CompletedImpl &&
            (identical(other.requestId, requestId) ||
                other.requestId == requestId));
  }

  @override
  int get hashCode => Object.hash(runtimeType, requestId);

  /// Create a copy of AiStreamEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$AiStreamEvent_CompletedImplCopyWith<_$AiStreamEvent_CompletedImpl>
      get copyWith => __$$AiStreamEvent_CompletedImplCopyWithImpl<
          _$AiStreamEvent_CompletedImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String requestId, String text) delta,
    required TResult Function(String requestId, int completed, int total)
        progress,
    required TResult Function(String requestId) completed,
    required TResult Function(String requestId, String code) error,
  }) {
    return completed(requestId);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String requestId, String text)? delta,
    TResult? Function(String requestId, int completed, int total)? progress,
    TResult? Function(String requestId)? completed,
    TResult? Function(String requestId, String code)? error,
  }) {
    return completed?.call(requestId);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String requestId, String text)? delta,
    TResult Function(String requestId, int completed, int total)? progress,
    TResult Function(String requestId)? completed,
    TResult Function(String requestId, String code)? error,
    required TResult orElse(),
  }) {
    if (completed != null) {
      return completed(requestId);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(AiStreamEvent_Delta value) delta,
    required TResult Function(AiStreamEvent_Progress value) progress,
    required TResult Function(AiStreamEvent_Completed value) completed,
    required TResult Function(AiStreamEvent_Error value) error,
  }) {
    return completed(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(AiStreamEvent_Delta value)? delta,
    TResult? Function(AiStreamEvent_Progress value)? progress,
    TResult? Function(AiStreamEvent_Completed value)? completed,
    TResult? Function(AiStreamEvent_Error value)? error,
  }) {
    return completed?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(AiStreamEvent_Delta value)? delta,
    TResult Function(AiStreamEvent_Progress value)? progress,
    TResult Function(AiStreamEvent_Completed value)? completed,
    TResult Function(AiStreamEvent_Error value)? error,
    required TResult orElse(),
  }) {
    if (completed != null) {
      return completed(this);
    }
    return orElse();
  }
}

abstract class AiStreamEvent_Completed extends AiStreamEvent {
  const factory AiStreamEvent_Completed({required final String requestId}) =
      _$AiStreamEvent_CompletedImpl;
  const AiStreamEvent_Completed._() : super._();

  @override
  String get requestId;

  /// Create a copy of AiStreamEvent
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$AiStreamEvent_CompletedImplCopyWith<_$AiStreamEvent_CompletedImpl>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$AiStreamEvent_ErrorImplCopyWith<$Res>
    implements $AiStreamEventCopyWith<$Res> {
  factory _$$AiStreamEvent_ErrorImplCopyWith(_$AiStreamEvent_ErrorImpl value,
          $Res Function(_$AiStreamEvent_ErrorImpl) then) =
      __$$AiStreamEvent_ErrorImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String requestId, String code});
}

/// @nodoc
class __$$AiStreamEvent_ErrorImplCopyWithImpl<$Res>
    extends _$AiStreamEventCopyWithImpl<$Res, _$AiStreamEvent_ErrorImpl>
    implements _$$AiStreamEvent_ErrorImplCopyWith<$Res> {
  __$$AiStreamEvent_ErrorImplCopyWithImpl(_$AiStreamEvent_ErrorImpl _value,
      $Res Function(_$AiStreamEvent_ErrorImpl) _then)
      : super(_value, _then);

  /// Create a copy of AiStreamEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? requestId = null,
    Object? code = null,
  }) {
    return _then(_$AiStreamEvent_ErrorImpl(
      requestId: null == requestId
          ? _value.requestId
          : requestId // ignore: cast_nullable_to_non_nullable
              as String,
      code: null == code
          ? _value.code
          : code // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$AiStreamEvent_ErrorImpl extends AiStreamEvent_Error {
  const _$AiStreamEvent_ErrorImpl({required this.requestId, required this.code})
      : super._();

  @override
  final String requestId;
  @override
  final String code;

  @override
  String toString() {
    return 'AiStreamEvent.error(requestId: $requestId, code: $code)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$AiStreamEvent_ErrorImpl &&
            (identical(other.requestId, requestId) ||
                other.requestId == requestId) &&
            (identical(other.code, code) || other.code == code));
  }

  @override
  int get hashCode => Object.hash(runtimeType, requestId, code);

  /// Create a copy of AiStreamEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$AiStreamEvent_ErrorImplCopyWith<_$AiStreamEvent_ErrorImpl> get copyWith =>
      __$$AiStreamEvent_ErrorImplCopyWithImpl<_$AiStreamEvent_ErrorImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String requestId, String text) delta,
    required TResult Function(String requestId, int completed, int total)
        progress,
    required TResult Function(String requestId) completed,
    required TResult Function(String requestId, String code) error,
  }) {
    return error(requestId, code);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String requestId, String text)? delta,
    TResult? Function(String requestId, int completed, int total)? progress,
    TResult? Function(String requestId)? completed,
    TResult? Function(String requestId, String code)? error,
  }) {
    return error?.call(requestId, code);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String requestId, String text)? delta,
    TResult Function(String requestId, int completed, int total)? progress,
    TResult Function(String requestId)? completed,
    TResult Function(String requestId, String code)? error,
    required TResult orElse(),
  }) {
    if (error != null) {
      return error(requestId, code);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(AiStreamEvent_Delta value) delta,
    required TResult Function(AiStreamEvent_Progress value) progress,
    required TResult Function(AiStreamEvent_Completed value) completed,
    required TResult Function(AiStreamEvent_Error value) error,
  }) {
    return error(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(AiStreamEvent_Delta value)? delta,
    TResult? Function(AiStreamEvent_Progress value)? progress,
    TResult? Function(AiStreamEvent_Completed value)? completed,
    TResult? Function(AiStreamEvent_Error value)? error,
  }) {
    return error?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(AiStreamEvent_Delta value)? delta,
    TResult Function(AiStreamEvent_Progress value)? progress,
    TResult Function(AiStreamEvent_Completed value)? completed,
    TResult Function(AiStreamEvent_Error value)? error,
    required TResult orElse(),
  }) {
    if (error != null) {
      return error(this);
    }
    return orElse();
  }
}

abstract class AiStreamEvent_Error extends AiStreamEvent {
  const factory AiStreamEvent_Error(
      {required final String requestId,
      required final String code}) = _$AiStreamEvent_ErrorImpl;
  const AiStreamEvent_Error._() : super._();

  @override
  String get requestId;
  String get code;

  /// Create a copy of AiStreamEvent
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$AiStreamEvent_ErrorImplCopyWith<_$AiStreamEvent_ErrorImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
mixin _$ArticleFilterKind {
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() all,
    required TResult Function() unread,
    required TResult Function() starred,
    required TResult Function() readLater,
    required TResult Function(int feedId) feed,
    required TResult Function(int folderId) folder,
    required TResult Function(int tagId) tag,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? all,
    TResult? Function()? unread,
    TResult? Function()? starred,
    TResult? Function()? readLater,
    TResult? Function(int feedId)? feed,
    TResult? Function(int folderId)? folder,
    TResult? Function(int tagId)? tag,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? all,
    TResult Function()? unread,
    TResult Function()? starred,
    TResult Function()? readLater,
    TResult Function(int feedId)? feed,
    TResult Function(int folderId)? folder,
    TResult Function(int tagId)? tag,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ArticleFilterKind_All value) all,
    required TResult Function(ArticleFilterKind_Unread value) unread,
    required TResult Function(ArticleFilterKind_Starred value) starred,
    required TResult Function(ArticleFilterKind_ReadLater value) readLater,
    required TResult Function(ArticleFilterKind_Feed value) feed,
    required TResult Function(ArticleFilterKind_Folder value) folder,
    required TResult Function(ArticleFilterKind_Tag value) tag,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ArticleFilterKind_All value)? all,
    TResult? Function(ArticleFilterKind_Unread value)? unread,
    TResult? Function(ArticleFilterKind_Starred value)? starred,
    TResult? Function(ArticleFilterKind_ReadLater value)? readLater,
    TResult? Function(ArticleFilterKind_Feed value)? feed,
    TResult? Function(ArticleFilterKind_Folder value)? folder,
    TResult? Function(ArticleFilterKind_Tag value)? tag,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ArticleFilterKind_All value)? all,
    TResult Function(ArticleFilterKind_Unread value)? unread,
    TResult Function(ArticleFilterKind_Starred value)? starred,
    TResult Function(ArticleFilterKind_ReadLater value)? readLater,
    TResult Function(ArticleFilterKind_Feed value)? feed,
    TResult Function(ArticleFilterKind_Folder value)? folder,
    TResult Function(ArticleFilterKind_Tag value)? tag,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $ArticleFilterKindCopyWith<$Res> {
  factory $ArticleFilterKindCopyWith(
          ArticleFilterKind value, $Res Function(ArticleFilterKind) then) =
      _$ArticleFilterKindCopyWithImpl<$Res, ArticleFilterKind>;
}

/// @nodoc
class _$ArticleFilterKindCopyWithImpl<$Res, $Val extends ArticleFilterKind>
    implements $ArticleFilterKindCopyWith<$Res> {
  _$ArticleFilterKindCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of ArticleFilterKind
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc
abstract class _$$ArticleFilterKind_AllImplCopyWith<$Res> {
  factory _$$ArticleFilterKind_AllImplCopyWith(
          _$ArticleFilterKind_AllImpl value,
          $Res Function(_$ArticleFilterKind_AllImpl) then) =
      __$$ArticleFilterKind_AllImplCopyWithImpl<$Res>;
}

/// @nodoc
class __$$ArticleFilterKind_AllImplCopyWithImpl<$Res>
    extends _$ArticleFilterKindCopyWithImpl<$Res, _$ArticleFilterKind_AllImpl>
    implements _$$ArticleFilterKind_AllImplCopyWith<$Res> {
  __$$ArticleFilterKind_AllImplCopyWithImpl(_$ArticleFilterKind_AllImpl _value,
      $Res Function(_$ArticleFilterKind_AllImpl) _then)
      : super(_value, _then);

  /// Create a copy of ArticleFilterKind
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc

class _$ArticleFilterKind_AllImpl extends ArticleFilterKind_All {
  const _$ArticleFilterKind_AllImpl() : super._();

  @override
  String toString() {
    return 'ArticleFilterKind.all()';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ArticleFilterKind_AllImpl);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() all,
    required TResult Function() unread,
    required TResult Function() starred,
    required TResult Function() readLater,
    required TResult Function(int feedId) feed,
    required TResult Function(int folderId) folder,
    required TResult Function(int tagId) tag,
  }) {
    return all();
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? all,
    TResult? Function()? unread,
    TResult? Function()? starred,
    TResult? Function()? readLater,
    TResult? Function(int feedId)? feed,
    TResult? Function(int folderId)? folder,
    TResult? Function(int tagId)? tag,
  }) {
    return all?.call();
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? all,
    TResult Function()? unread,
    TResult Function()? starred,
    TResult Function()? readLater,
    TResult Function(int feedId)? feed,
    TResult Function(int folderId)? folder,
    TResult Function(int tagId)? tag,
    required TResult orElse(),
  }) {
    if (all != null) {
      return all();
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ArticleFilterKind_All value) all,
    required TResult Function(ArticleFilterKind_Unread value) unread,
    required TResult Function(ArticleFilterKind_Starred value) starred,
    required TResult Function(ArticleFilterKind_ReadLater value) readLater,
    required TResult Function(ArticleFilterKind_Feed value) feed,
    required TResult Function(ArticleFilterKind_Folder value) folder,
    required TResult Function(ArticleFilterKind_Tag value) tag,
  }) {
    return all(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ArticleFilterKind_All value)? all,
    TResult? Function(ArticleFilterKind_Unread value)? unread,
    TResult? Function(ArticleFilterKind_Starred value)? starred,
    TResult? Function(ArticleFilterKind_ReadLater value)? readLater,
    TResult? Function(ArticleFilterKind_Feed value)? feed,
    TResult? Function(ArticleFilterKind_Folder value)? folder,
    TResult? Function(ArticleFilterKind_Tag value)? tag,
  }) {
    return all?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ArticleFilterKind_All value)? all,
    TResult Function(ArticleFilterKind_Unread value)? unread,
    TResult Function(ArticleFilterKind_Starred value)? starred,
    TResult Function(ArticleFilterKind_ReadLater value)? readLater,
    TResult Function(ArticleFilterKind_Feed value)? feed,
    TResult Function(ArticleFilterKind_Folder value)? folder,
    TResult Function(ArticleFilterKind_Tag value)? tag,
    required TResult orElse(),
  }) {
    if (all != null) {
      return all(this);
    }
    return orElse();
  }
}

abstract class ArticleFilterKind_All extends ArticleFilterKind {
  const factory ArticleFilterKind_All() = _$ArticleFilterKind_AllImpl;
  const ArticleFilterKind_All._() : super._();
}

/// @nodoc
abstract class _$$ArticleFilterKind_UnreadImplCopyWith<$Res> {
  factory _$$ArticleFilterKind_UnreadImplCopyWith(
          _$ArticleFilterKind_UnreadImpl value,
          $Res Function(_$ArticleFilterKind_UnreadImpl) then) =
      __$$ArticleFilterKind_UnreadImplCopyWithImpl<$Res>;
}

/// @nodoc
class __$$ArticleFilterKind_UnreadImplCopyWithImpl<$Res>
    extends _$ArticleFilterKindCopyWithImpl<$Res,
        _$ArticleFilterKind_UnreadImpl>
    implements _$$ArticleFilterKind_UnreadImplCopyWith<$Res> {
  __$$ArticleFilterKind_UnreadImplCopyWithImpl(
      _$ArticleFilterKind_UnreadImpl _value,
      $Res Function(_$ArticleFilterKind_UnreadImpl) _then)
      : super(_value, _then);

  /// Create a copy of ArticleFilterKind
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc

class _$ArticleFilterKind_UnreadImpl extends ArticleFilterKind_Unread {
  const _$ArticleFilterKind_UnreadImpl() : super._();

  @override
  String toString() {
    return 'ArticleFilterKind.unread()';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ArticleFilterKind_UnreadImpl);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() all,
    required TResult Function() unread,
    required TResult Function() starred,
    required TResult Function() readLater,
    required TResult Function(int feedId) feed,
    required TResult Function(int folderId) folder,
    required TResult Function(int tagId) tag,
  }) {
    return unread();
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? all,
    TResult? Function()? unread,
    TResult? Function()? starred,
    TResult? Function()? readLater,
    TResult? Function(int feedId)? feed,
    TResult? Function(int folderId)? folder,
    TResult? Function(int tagId)? tag,
  }) {
    return unread?.call();
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? all,
    TResult Function()? unread,
    TResult Function()? starred,
    TResult Function()? readLater,
    TResult Function(int feedId)? feed,
    TResult Function(int folderId)? folder,
    TResult Function(int tagId)? tag,
    required TResult orElse(),
  }) {
    if (unread != null) {
      return unread();
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ArticleFilterKind_All value) all,
    required TResult Function(ArticleFilterKind_Unread value) unread,
    required TResult Function(ArticleFilterKind_Starred value) starred,
    required TResult Function(ArticleFilterKind_ReadLater value) readLater,
    required TResult Function(ArticleFilterKind_Feed value) feed,
    required TResult Function(ArticleFilterKind_Folder value) folder,
    required TResult Function(ArticleFilterKind_Tag value) tag,
  }) {
    return unread(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ArticleFilterKind_All value)? all,
    TResult? Function(ArticleFilterKind_Unread value)? unread,
    TResult? Function(ArticleFilterKind_Starred value)? starred,
    TResult? Function(ArticleFilterKind_ReadLater value)? readLater,
    TResult? Function(ArticleFilterKind_Feed value)? feed,
    TResult? Function(ArticleFilterKind_Folder value)? folder,
    TResult? Function(ArticleFilterKind_Tag value)? tag,
  }) {
    return unread?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ArticleFilterKind_All value)? all,
    TResult Function(ArticleFilterKind_Unread value)? unread,
    TResult Function(ArticleFilterKind_Starred value)? starred,
    TResult Function(ArticleFilterKind_ReadLater value)? readLater,
    TResult Function(ArticleFilterKind_Feed value)? feed,
    TResult Function(ArticleFilterKind_Folder value)? folder,
    TResult Function(ArticleFilterKind_Tag value)? tag,
    required TResult orElse(),
  }) {
    if (unread != null) {
      return unread(this);
    }
    return orElse();
  }
}

abstract class ArticleFilterKind_Unread extends ArticleFilterKind {
  const factory ArticleFilterKind_Unread() = _$ArticleFilterKind_UnreadImpl;
  const ArticleFilterKind_Unread._() : super._();
}

/// @nodoc
abstract class _$$ArticleFilterKind_StarredImplCopyWith<$Res> {
  factory _$$ArticleFilterKind_StarredImplCopyWith(
          _$ArticleFilterKind_StarredImpl value,
          $Res Function(_$ArticleFilterKind_StarredImpl) then) =
      __$$ArticleFilterKind_StarredImplCopyWithImpl<$Res>;
}

/// @nodoc
class __$$ArticleFilterKind_StarredImplCopyWithImpl<$Res>
    extends _$ArticleFilterKindCopyWithImpl<$Res,
        _$ArticleFilterKind_StarredImpl>
    implements _$$ArticleFilterKind_StarredImplCopyWith<$Res> {
  __$$ArticleFilterKind_StarredImplCopyWithImpl(
      _$ArticleFilterKind_StarredImpl _value,
      $Res Function(_$ArticleFilterKind_StarredImpl) _then)
      : super(_value, _then);

  /// Create a copy of ArticleFilterKind
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc

class _$ArticleFilterKind_StarredImpl extends ArticleFilterKind_Starred {
  const _$ArticleFilterKind_StarredImpl() : super._();

  @override
  String toString() {
    return 'ArticleFilterKind.starred()';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ArticleFilterKind_StarredImpl);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() all,
    required TResult Function() unread,
    required TResult Function() starred,
    required TResult Function() readLater,
    required TResult Function(int feedId) feed,
    required TResult Function(int folderId) folder,
    required TResult Function(int tagId) tag,
  }) {
    return starred();
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? all,
    TResult? Function()? unread,
    TResult? Function()? starred,
    TResult? Function()? readLater,
    TResult? Function(int feedId)? feed,
    TResult? Function(int folderId)? folder,
    TResult? Function(int tagId)? tag,
  }) {
    return starred?.call();
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? all,
    TResult Function()? unread,
    TResult Function()? starred,
    TResult Function()? readLater,
    TResult Function(int feedId)? feed,
    TResult Function(int folderId)? folder,
    TResult Function(int tagId)? tag,
    required TResult orElse(),
  }) {
    if (starred != null) {
      return starred();
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ArticleFilterKind_All value) all,
    required TResult Function(ArticleFilterKind_Unread value) unread,
    required TResult Function(ArticleFilterKind_Starred value) starred,
    required TResult Function(ArticleFilterKind_ReadLater value) readLater,
    required TResult Function(ArticleFilterKind_Feed value) feed,
    required TResult Function(ArticleFilterKind_Folder value) folder,
    required TResult Function(ArticleFilterKind_Tag value) tag,
  }) {
    return starred(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ArticleFilterKind_All value)? all,
    TResult? Function(ArticleFilterKind_Unread value)? unread,
    TResult? Function(ArticleFilterKind_Starred value)? starred,
    TResult? Function(ArticleFilterKind_ReadLater value)? readLater,
    TResult? Function(ArticleFilterKind_Feed value)? feed,
    TResult? Function(ArticleFilterKind_Folder value)? folder,
    TResult? Function(ArticleFilterKind_Tag value)? tag,
  }) {
    return starred?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ArticleFilterKind_All value)? all,
    TResult Function(ArticleFilterKind_Unread value)? unread,
    TResult Function(ArticleFilterKind_Starred value)? starred,
    TResult Function(ArticleFilterKind_ReadLater value)? readLater,
    TResult Function(ArticleFilterKind_Feed value)? feed,
    TResult Function(ArticleFilterKind_Folder value)? folder,
    TResult Function(ArticleFilterKind_Tag value)? tag,
    required TResult orElse(),
  }) {
    if (starred != null) {
      return starred(this);
    }
    return orElse();
  }
}

abstract class ArticleFilterKind_Starred extends ArticleFilterKind {
  const factory ArticleFilterKind_Starred() = _$ArticleFilterKind_StarredImpl;
  const ArticleFilterKind_Starred._() : super._();
}

/// @nodoc
abstract class _$$ArticleFilterKind_ReadLaterImplCopyWith<$Res> {
  factory _$$ArticleFilterKind_ReadLaterImplCopyWith(
          _$ArticleFilterKind_ReadLaterImpl value,
          $Res Function(_$ArticleFilterKind_ReadLaterImpl) then) =
      __$$ArticleFilterKind_ReadLaterImplCopyWithImpl<$Res>;
}

/// @nodoc
class __$$ArticleFilterKind_ReadLaterImplCopyWithImpl<$Res>
    extends _$ArticleFilterKindCopyWithImpl<$Res,
        _$ArticleFilterKind_ReadLaterImpl>
    implements _$$ArticleFilterKind_ReadLaterImplCopyWith<$Res> {
  __$$ArticleFilterKind_ReadLaterImplCopyWithImpl(
      _$ArticleFilterKind_ReadLaterImpl _value,
      $Res Function(_$ArticleFilterKind_ReadLaterImpl) _then)
      : super(_value, _then);

  /// Create a copy of ArticleFilterKind
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc

class _$ArticleFilterKind_ReadLaterImpl extends ArticleFilterKind_ReadLater {
  const _$ArticleFilterKind_ReadLaterImpl() : super._();

  @override
  String toString() {
    return 'ArticleFilterKind.readLater()';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ArticleFilterKind_ReadLaterImpl);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() all,
    required TResult Function() unread,
    required TResult Function() starred,
    required TResult Function() readLater,
    required TResult Function(int feedId) feed,
    required TResult Function(int folderId) folder,
    required TResult Function(int tagId) tag,
  }) {
    return readLater();
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? all,
    TResult? Function()? unread,
    TResult? Function()? starred,
    TResult? Function()? readLater,
    TResult? Function(int feedId)? feed,
    TResult? Function(int folderId)? folder,
    TResult? Function(int tagId)? tag,
  }) {
    return readLater?.call();
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? all,
    TResult Function()? unread,
    TResult Function()? starred,
    TResult Function()? readLater,
    TResult Function(int feedId)? feed,
    TResult Function(int folderId)? folder,
    TResult Function(int tagId)? tag,
    required TResult orElse(),
  }) {
    if (readLater != null) {
      return readLater();
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ArticleFilterKind_All value) all,
    required TResult Function(ArticleFilterKind_Unread value) unread,
    required TResult Function(ArticleFilterKind_Starred value) starred,
    required TResult Function(ArticleFilterKind_ReadLater value) readLater,
    required TResult Function(ArticleFilterKind_Feed value) feed,
    required TResult Function(ArticleFilterKind_Folder value) folder,
    required TResult Function(ArticleFilterKind_Tag value) tag,
  }) {
    return readLater(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ArticleFilterKind_All value)? all,
    TResult? Function(ArticleFilterKind_Unread value)? unread,
    TResult? Function(ArticleFilterKind_Starred value)? starred,
    TResult? Function(ArticleFilterKind_ReadLater value)? readLater,
    TResult? Function(ArticleFilterKind_Feed value)? feed,
    TResult? Function(ArticleFilterKind_Folder value)? folder,
    TResult? Function(ArticleFilterKind_Tag value)? tag,
  }) {
    return readLater?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ArticleFilterKind_All value)? all,
    TResult Function(ArticleFilterKind_Unread value)? unread,
    TResult Function(ArticleFilterKind_Starred value)? starred,
    TResult Function(ArticleFilterKind_ReadLater value)? readLater,
    TResult Function(ArticleFilterKind_Feed value)? feed,
    TResult Function(ArticleFilterKind_Folder value)? folder,
    TResult Function(ArticleFilterKind_Tag value)? tag,
    required TResult orElse(),
  }) {
    if (readLater != null) {
      return readLater(this);
    }
    return orElse();
  }
}

abstract class ArticleFilterKind_ReadLater extends ArticleFilterKind {
  const factory ArticleFilterKind_ReadLater() =
      _$ArticleFilterKind_ReadLaterImpl;
  const ArticleFilterKind_ReadLater._() : super._();
}

/// @nodoc
abstract class _$$ArticleFilterKind_FeedImplCopyWith<$Res> {
  factory _$$ArticleFilterKind_FeedImplCopyWith(
          _$ArticleFilterKind_FeedImpl value,
          $Res Function(_$ArticleFilterKind_FeedImpl) then) =
      __$$ArticleFilterKind_FeedImplCopyWithImpl<$Res>;
  @useResult
  $Res call({int feedId});
}

/// @nodoc
class __$$ArticleFilterKind_FeedImplCopyWithImpl<$Res>
    extends _$ArticleFilterKindCopyWithImpl<$Res, _$ArticleFilterKind_FeedImpl>
    implements _$$ArticleFilterKind_FeedImplCopyWith<$Res> {
  __$$ArticleFilterKind_FeedImplCopyWithImpl(
      _$ArticleFilterKind_FeedImpl _value,
      $Res Function(_$ArticleFilterKind_FeedImpl) _then)
      : super(_value, _then);

  /// Create a copy of ArticleFilterKind
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? feedId = null,
  }) {
    return _then(_$ArticleFilterKind_FeedImpl(
      feedId: null == feedId
          ? _value.feedId
          : feedId // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc

class _$ArticleFilterKind_FeedImpl extends ArticleFilterKind_Feed {
  const _$ArticleFilterKind_FeedImpl({required this.feedId}) : super._();

  @override
  final int feedId;

  @override
  String toString() {
    return 'ArticleFilterKind.feed(feedId: $feedId)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ArticleFilterKind_FeedImpl &&
            (identical(other.feedId, feedId) || other.feedId == feedId));
  }

  @override
  int get hashCode => Object.hash(runtimeType, feedId);

  /// Create a copy of ArticleFilterKind
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ArticleFilterKind_FeedImplCopyWith<_$ArticleFilterKind_FeedImpl>
      get copyWith => __$$ArticleFilterKind_FeedImplCopyWithImpl<
          _$ArticleFilterKind_FeedImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() all,
    required TResult Function() unread,
    required TResult Function() starred,
    required TResult Function() readLater,
    required TResult Function(int feedId) feed,
    required TResult Function(int folderId) folder,
    required TResult Function(int tagId) tag,
  }) {
    return feed(feedId);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? all,
    TResult? Function()? unread,
    TResult? Function()? starred,
    TResult? Function()? readLater,
    TResult? Function(int feedId)? feed,
    TResult? Function(int folderId)? folder,
    TResult? Function(int tagId)? tag,
  }) {
    return feed?.call(feedId);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? all,
    TResult Function()? unread,
    TResult Function()? starred,
    TResult Function()? readLater,
    TResult Function(int feedId)? feed,
    TResult Function(int folderId)? folder,
    TResult Function(int tagId)? tag,
    required TResult orElse(),
  }) {
    if (feed != null) {
      return feed(feedId);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ArticleFilterKind_All value) all,
    required TResult Function(ArticleFilterKind_Unread value) unread,
    required TResult Function(ArticleFilterKind_Starred value) starred,
    required TResult Function(ArticleFilterKind_ReadLater value) readLater,
    required TResult Function(ArticleFilterKind_Feed value) feed,
    required TResult Function(ArticleFilterKind_Folder value) folder,
    required TResult Function(ArticleFilterKind_Tag value) tag,
  }) {
    return feed(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ArticleFilterKind_All value)? all,
    TResult? Function(ArticleFilterKind_Unread value)? unread,
    TResult? Function(ArticleFilterKind_Starred value)? starred,
    TResult? Function(ArticleFilterKind_ReadLater value)? readLater,
    TResult? Function(ArticleFilterKind_Feed value)? feed,
    TResult? Function(ArticleFilterKind_Folder value)? folder,
    TResult? Function(ArticleFilterKind_Tag value)? tag,
  }) {
    return feed?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ArticleFilterKind_All value)? all,
    TResult Function(ArticleFilterKind_Unread value)? unread,
    TResult Function(ArticleFilterKind_Starred value)? starred,
    TResult Function(ArticleFilterKind_ReadLater value)? readLater,
    TResult Function(ArticleFilterKind_Feed value)? feed,
    TResult Function(ArticleFilterKind_Folder value)? folder,
    TResult Function(ArticleFilterKind_Tag value)? tag,
    required TResult orElse(),
  }) {
    if (feed != null) {
      return feed(this);
    }
    return orElse();
  }
}

abstract class ArticleFilterKind_Feed extends ArticleFilterKind {
  const factory ArticleFilterKind_Feed({required final int feedId}) =
      _$ArticleFilterKind_FeedImpl;
  const ArticleFilterKind_Feed._() : super._();

  int get feedId;

  /// Create a copy of ArticleFilterKind
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ArticleFilterKind_FeedImplCopyWith<_$ArticleFilterKind_FeedImpl>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$ArticleFilterKind_FolderImplCopyWith<$Res> {
  factory _$$ArticleFilterKind_FolderImplCopyWith(
          _$ArticleFilterKind_FolderImpl value,
          $Res Function(_$ArticleFilterKind_FolderImpl) then) =
      __$$ArticleFilterKind_FolderImplCopyWithImpl<$Res>;
  @useResult
  $Res call({int folderId});
}

/// @nodoc
class __$$ArticleFilterKind_FolderImplCopyWithImpl<$Res>
    extends _$ArticleFilterKindCopyWithImpl<$Res,
        _$ArticleFilterKind_FolderImpl>
    implements _$$ArticleFilterKind_FolderImplCopyWith<$Res> {
  __$$ArticleFilterKind_FolderImplCopyWithImpl(
      _$ArticleFilterKind_FolderImpl _value,
      $Res Function(_$ArticleFilterKind_FolderImpl) _then)
      : super(_value, _then);

  /// Create a copy of ArticleFilterKind
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? folderId = null,
  }) {
    return _then(_$ArticleFilterKind_FolderImpl(
      folderId: null == folderId
          ? _value.folderId
          : folderId // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc

class _$ArticleFilterKind_FolderImpl extends ArticleFilterKind_Folder {
  const _$ArticleFilterKind_FolderImpl({required this.folderId}) : super._();

  @override
  final int folderId;

  @override
  String toString() {
    return 'ArticleFilterKind.folder(folderId: $folderId)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ArticleFilterKind_FolderImpl &&
            (identical(other.folderId, folderId) ||
                other.folderId == folderId));
  }

  @override
  int get hashCode => Object.hash(runtimeType, folderId);

  /// Create a copy of ArticleFilterKind
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ArticleFilterKind_FolderImplCopyWith<_$ArticleFilterKind_FolderImpl>
      get copyWith => __$$ArticleFilterKind_FolderImplCopyWithImpl<
          _$ArticleFilterKind_FolderImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() all,
    required TResult Function() unread,
    required TResult Function() starred,
    required TResult Function() readLater,
    required TResult Function(int feedId) feed,
    required TResult Function(int folderId) folder,
    required TResult Function(int tagId) tag,
  }) {
    return folder(folderId);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? all,
    TResult? Function()? unread,
    TResult? Function()? starred,
    TResult? Function()? readLater,
    TResult? Function(int feedId)? feed,
    TResult? Function(int folderId)? folder,
    TResult? Function(int tagId)? tag,
  }) {
    return folder?.call(folderId);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? all,
    TResult Function()? unread,
    TResult Function()? starred,
    TResult Function()? readLater,
    TResult Function(int feedId)? feed,
    TResult Function(int folderId)? folder,
    TResult Function(int tagId)? tag,
    required TResult orElse(),
  }) {
    if (folder != null) {
      return folder(folderId);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ArticleFilterKind_All value) all,
    required TResult Function(ArticleFilterKind_Unread value) unread,
    required TResult Function(ArticleFilterKind_Starred value) starred,
    required TResult Function(ArticleFilterKind_ReadLater value) readLater,
    required TResult Function(ArticleFilterKind_Feed value) feed,
    required TResult Function(ArticleFilterKind_Folder value) folder,
    required TResult Function(ArticleFilterKind_Tag value) tag,
  }) {
    return folder(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ArticleFilterKind_All value)? all,
    TResult? Function(ArticleFilterKind_Unread value)? unread,
    TResult? Function(ArticleFilterKind_Starred value)? starred,
    TResult? Function(ArticleFilterKind_ReadLater value)? readLater,
    TResult? Function(ArticleFilterKind_Feed value)? feed,
    TResult? Function(ArticleFilterKind_Folder value)? folder,
    TResult? Function(ArticleFilterKind_Tag value)? tag,
  }) {
    return folder?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ArticleFilterKind_All value)? all,
    TResult Function(ArticleFilterKind_Unread value)? unread,
    TResult Function(ArticleFilterKind_Starred value)? starred,
    TResult Function(ArticleFilterKind_ReadLater value)? readLater,
    TResult Function(ArticleFilterKind_Feed value)? feed,
    TResult Function(ArticleFilterKind_Folder value)? folder,
    TResult Function(ArticleFilterKind_Tag value)? tag,
    required TResult orElse(),
  }) {
    if (folder != null) {
      return folder(this);
    }
    return orElse();
  }
}

abstract class ArticleFilterKind_Folder extends ArticleFilterKind {
  const factory ArticleFilterKind_Folder({required final int folderId}) =
      _$ArticleFilterKind_FolderImpl;
  const ArticleFilterKind_Folder._() : super._();

  int get folderId;

  /// Create a copy of ArticleFilterKind
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ArticleFilterKind_FolderImplCopyWith<_$ArticleFilterKind_FolderImpl>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$ArticleFilterKind_TagImplCopyWith<$Res> {
  factory _$$ArticleFilterKind_TagImplCopyWith(
          _$ArticleFilterKind_TagImpl value,
          $Res Function(_$ArticleFilterKind_TagImpl) then) =
      __$$ArticleFilterKind_TagImplCopyWithImpl<$Res>;
  @useResult
  $Res call({int tagId});
}

/// @nodoc
class __$$ArticleFilterKind_TagImplCopyWithImpl<$Res>
    extends _$ArticleFilterKindCopyWithImpl<$Res, _$ArticleFilterKind_TagImpl>
    implements _$$ArticleFilterKind_TagImplCopyWith<$Res> {
  __$$ArticleFilterKind_TagImplCopyWithImpl(_$ArticleFilterKind_TagImpl _value,
      $Res Function(_$ArticleFilterKind_TagImpl) _then)
      : super(_value, _then);

  /// Create a copy of ArticleFilterKind
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? tagId = null,
  }) {
    return _then(_$ArticleFilterKind_TagImpl(
      tagId: null == tagId
          ? _value.tagId
          : tagId // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc

class _$ArticleFilterKind_TagImpl extends ArticleFilterKind_Tag {
  const _$ArticleFilterKind_TagImpl({required this.tagId}) : super._();

  @override
  final int tagId;

  @override
  String toString() {
    return 'ArticleFilterKind.tag(tagId: $tagId)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ArticleFilterKind_TagImpl &&
            (identical(other.tagId, tagId) || other.tagId == tagId));
  }

  @override
  int get hashCode => Object.hash(runtimeType, tagId);

  /// Create a copy of ArticleFilterKind
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ArticleFilterKind_TagImplCopyWith<_$ArticleFilterKind_TagImpl>
      get copyWith => __$$ArticleFilterKind_TagImplCopyWithImpl<
          _$ArticleFilterKind_TagImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() all,
    required TResult Function() unread,
    required TResult Function() starred,
    required TResult Function() readLater,
    required TResult Function(int feedId) feed,
    required TResult Function(int folderId) folder,
    required TResult Function(int tagId) tag,
  }) {
    return tag(tagId);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? all,
    TResult? Function()? unread,
    TResult? Function()? starred,
    TResult? Function()? readLater,
    TResult? Function(int feedId)? feed,
    TResult? Function(int folderId)? folder,
    TResult? Function(int tagId)? tag,
  }) {
    return tag?.call(tagId);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? all,
    TResult Function()? unread,
    TResult Function()? starred,
    TResult Function()? readLater,
    TResult Function(int feedId)? feed,
    TResult Function(int folderId)? folder,
    TResult Function(int tagId)? tag,
    required TResult orElse(),
  }) {
    if (tag != null) {
      return tag(tagId);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ArticleFilterKind_All value) all,
    required TResult Function(ArticleFilterKind_Unread value) unread,
    required TResult Function(ArticleFilterKind_Starred value) starred,
    required TResult Function(ArticleFilterKind_ReadLater value) readLater,
    required TResult Function(ArticleFilterKind_Feed value) feed,
    required TResult Function(ArticleFilterKind_Folder value) folder,
    required TResult Function(ArticleFilterKind_Tag value) tag,
  }) {
    return tag(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ArticleFilterKind_All value)? all,
    TResult? Function(ArticleFilterKind_Unread value)? unread,
    TResult? Function(ArticleFilterKind_Starred value)? starred,
    TResult? Function(ArticleFilterKind_ReadLater value)? readLater,
    TResult? Function(ArticleFilterKind_Feed value)? feed,
    TResult? Function(ArticleFilterKind_Folder value)? folder,
    TResult? Function(ArticleFilterKind_Tag value)? tag,
  }) {
    return tag?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ArticleFilterKind_All value)? all,
    TResult Function(ArticleFilterKind_Unread value)? unread,
    TResult Function(ArticleFilterKind_Starred value)? starred,
    TResult Function(ArticleFilterKind_ReadLater value)? readLater,
    TResult Function(ArticleFilterKind_Feed value)? feed,
    TResult Function(ArticleFilterKind_Folder value)? folder,
    TResult Function(ArticleFilterKind_Tag value)? tag,
    required TResult orElse(),
  }) {
    if (tag != null) {
      return tag(this);
    }
    return orElse();
  }
}

abstract class ArticleFilterKind_Tag extends ArticleFilterKind {
  const factory ArticleFilterKind_Tag({required final int tagId}) =
      _$ArticleFilterKind_TagImpl;
  const ArticleFilterKind_Tag._() : super._();

  int get tagId;

  /// Create a copy of ArticleFilterKind
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ArticleFilterKind_TagImplCopyWith<_$ArticleFilterKind_TagImpl>
      get copyWith => throw _privateConstructorUsedError;
}
