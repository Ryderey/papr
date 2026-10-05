//! Windows-only private sync credentials. No plaintext fallback.
use crate::error::{AppError, AppResult};

fn target(reference: &str) -> AppResult<Vec<u16>> {
    let suffix = reference
        .strip_prefix("papr.sync.")
        .ok_or_else(|| AppError::code("githubInvalidCredential"))?;
    if suffix.is_empty()
        || suffix.len() > 80
        || !suffix
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
    {
        return Err(AppError::code("githubInvalidCredential"));
    }
    Ok(format!("Papr/GitHubSync/{reference}\0")
        .encode_utf16()
        .collect())
}

#[cfg(windows)]
pub fn get(reference: &str) -> AppResult<Option<String>> {
    use windows_sys::Win32::{
        Foundation::{GetLastError, ERROR_NOT_FOUND},
        Security::Credentials::{CredFree, CredReadW, CREDENTIALW, CRED_TYPE_GENERIC},
    };
    let target = target(reference)?;
    let mut credential: *mut CREDENTIALW = std::ptr::null_mut();
    // CredRead allocates the complete record; copy the validated byte slice
    // before freeing it exactly once. The pointer never crosses an await.
    unsafe {
        if CredReadW(target.as_ptr(), CRED_TYPE_GENERIC, 0, &mut credential) == 0 {
            return if GetLastError() == ERROR_NOT_FOUND {
                Ok(None)
            } else {
                Err(AppError::code("credentialReadFailed"))
            };
        }
        if credential.is_null() {
            return Err(AppError::code("credentialReadFailed"));
        }
        let record = &*credential;
        let result = if record.CredentialBlobSize > 4096
            || (record.CredentialBlobSize > 0 && record.CredentialBlob.is_null())
        {
            Err(AppError::code("credentialReadFailed"))
        } else if record.CredentialBlobSize == 0 {
            Ok(String::new())
        } else {
            String::from_utf8(
                std::slice::from_raw_parts(
                    record.CredentialBlob,
                    record.CredentialBlobSize as usize,
                )
                .to_vec(),
            )
            .map_err(|_| AppError::code("credentialReadFailed"))
        };
        CredFree(credential.cast());
        result.map(Some)
    }
}
#[cfg(windows)]
pub fn set(reference: &str, value: &str) -> AppResult<()> {
    use windows_sys::Win32::Security::Credentials::{
        CredWriteW, CREDENTIALW, CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC,
    };
    let mut target = target(reference)?;
    if value.is_empty() || value.len() > 4096 {
        return Err(AppError::code("githubInvalidCredential"));
    }
    let mut bytes = value.as_bytes().to_vec();
    let mut record: CREDENTIALW = unsafe { std::mem::zeroed() };
    record.Type = CRED_TYPE_GENERIC;
    record.TargetName = target.as_mut_ptr();
    record.Persist = CRED_PERSIST_LOCAL_MACHINE;
    record.CredentialBlobSize = bytes.len() as u32;
    record.CredentialBlob = bytes.as_mut_ptr();
    let result = unsafe { CredWriteW(&record, 0) };
    bytes.fill(0);
    if result == 0 {
        Err(AppError::code("credentialWriteFailed"))
    } else {
        Ok(())
    }
}
#[cfg(windows)]
pub fn delete(reference: &str) -> AppResult<()> {
    use windows_sys::Win32::{
        Foundation::{GetLastError, ERROR_NOT_FOUND},
        Security::Credentials::{CredDeleteW, CRED_TYPE_GENERIC},
    };
    let target = target(reference)?;
    if unsafe { CredDeleteW(target.as_ptr(), CRED_TYPE_GENERIC, 0) } == 0
        && unsafe { GetLastError() } != ERROR_NOT_FOUND
    {
        Err(AppError::code("credentialDeleteFailed"))
    } else {
        Ok(())
    }
}
#[cfg(not(windows))]
pub fn get(reference: &str) -> AppResult<Option<String>> {
    target(reference)?;
    Err(AppError::code("githubSecureStorageUnavailable"))
}
#[cfg(not(windows))]
pub fn set(reference: &str, _value: &str) -> AppResult<()> {
    target(reference)?;
    Err(AppError::code("githubSecureStorageUnavailable"))
}
#[cfg(not(windows))]
pub fn delete(reference: &str) -> AppResult<()> {
    target(reference)?;
    Err(AppError::code("githubSecureStorageUnavailable"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn credential_namespace_is_bounded() {
        assert!(target("papr.sync.abc_123-x").is_ok());
        for value in ["other.abc", "papr.sync.", "papr.sync.a/b", "papr.sync.a\0b"] {
            assert!(target(value).is_err());
        }
    }
    #[cfg(windows)]
    #[test]
    fn windows_secure_storage_round_trip_uses_an_isolated_test_reference() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let reference = format!("papr.sync.test_{}_{unique}", std::process::id());
        assert!(get(&reference).unwrap().is_none());
        let outcome = set(&reference, "Papr test marker").and_then(|()| get(&reference));
        let cleanup = delete(&reference);
        cleanup.unwrap();
        assert_eq!(outcome.unwrap(), Some("Papr test marker".into()));
        assert!(get(&reference).unwrap().is_none());
    }
}
