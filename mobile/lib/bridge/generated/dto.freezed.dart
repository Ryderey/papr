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
