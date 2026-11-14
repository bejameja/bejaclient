//! accounts.json holds live MSA/Minecraft/Beja tokens, so on Windows we run it through DPAPI
//! before it touches disk — tied to the Windows user account, no key file or password to manage.
//! Non-Windows just writes it plain for now, no keychain wired up there yet.

#[cfg(windows)]
use std::ffi::c_void;
#[cfg(windows)]
use windows::core::PCWSTR;
#[cfg(windows)]
use windows::Win32::Foundation::{LocalFree, HLOCAL};
#[cfg(windows)]
use windows::Win32::Security::Cryptography::{
    CryptProtectData, CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
};

#[cfg(windows)]
unsafe fn take(blob: CRYPT_INTEGER_BLOB) -> Vec<u8> {
    let out = std::slice::from_raw_parts(blob.pbData, blob.cbData as usize).to_vec();
    // windows-rs's LocalFree binding reports an error here even when the free worked, so ignore it
    let _ = LocalFree(Some(HLOCAL(blob.pbData as *mut c_void)));
    out
}

#[cfg(windows)]
pub fn protect(data: &[u8]) -> Result<Vec<u8>, String> {
    unsafe {
        let input = CRYPT_INTEGER_BLOB { cbData: data.len() as u32, pbData: data.as_ptr() as *mut u8 };
        let mut out = CRYPT_INTEGER_BLOB { cbData: 0, pbData: std::ptr::null_mut() };
        CryptProtectData(&input, PCWSTR::null(), None, None, None, CRYPTPROTECT_UI_FORBIDDEN, &mut out).map_err(|e| e.to_string())?;
        Ok(take(out))
    }
}

#[cfg(windows)]
pub fn unprotect(data: &[u8]) -> Result<Vec<u8>, String> {
    unsafe {
        let input = CRYPT_INTEGER_BLOB { cbData: data.len() as u32, pbData: data.as_ptr() as *mut u8 };
        let mut out = CRYPT_INTEGER_BLOB { cbData: 0, pbData: std::ptr::null_mut() };
        CryptUnprotectData(&input, None, None, None, None, CRYPTPROTECT_UI_FORBIDDEN, &mut out).map_err(|e| e.to_string())?;
        Ok(take(out))
    }
}

#[cfg(not(windows))]
pub fn protect(data: &[u8]) -> Result<Vec<u8>, String> {
    Ok(data.to_vec())
}

#[cfg(not(windows))]
pub fn unprotect(data: &[u8]) -> Result<Vec<u8>, String> {
    Ok(data.to_vec())
}
