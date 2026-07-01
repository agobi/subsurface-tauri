// AI-generated (Claude)
use std::ffi::CStr;
use std::ptr;
use std::sync::OnceLock;
use crate::dc::ffi::*;
use super::context::DcContext;

pub struct DescriptorInfo {
    pub vendor: String,
    pub product: String,
    pub transports: Vec<String>,
}

pub struct DcModel {
    pub product: String,
    pub transports: Vec<String>,
}

static DESCRIPTORS: OnceLock<Vec<DescriptorInfo>> = OnceLock::new();

fn transport_names(mask: u32) -> Vec<String> {
    let mut names = Vec::new();
    if mask & dc_transport_t_DC_TRANSPORT_USBHID != 0 { names.push("USBHID".to_string()); }
    if mask & dc_transport_t_DC_TRANSPORT_SERIAL != 0 { names.push("Serial".to_string()); }
    if mask & dc_transport_t_DC_TRANSPORT_USB != 0 { names.push("USB".to_string()); }
    if mask & dc_transport_t_DC_TRANSPORT_BLUETOOTH != 0 { names.push("Bluetooth".to_string()); }
    if mask & dc_transport_t_DC_TRANSPORT_BLE != 0 { names.push("BLE".to_string()); }
    names
}

fn load_descriptors() -> Vec<DescriptorInfo> {
    let ctx = match DcContext::new() {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let mut infos = Vec::new();
    unsafe {
        let mut iterator = ptr::null_mut();
        if dc_descriptor_iterator_new(&mut iterator, ctx.as_ptr()) != dc_status_t_DC_STATUS_SUCCESS {
            return infos;
        }
        loop {
            let mut desc: *mut dc_descriptor_t = ptr::null_mut();
            let rc = dc_iterator_next(
                iterator,
                &mut desc as *mut *mut dc_descriptor_t as *mut _,
            );
            if rc != dc_status_t_DC_STATUS_SUCCESS {
                break;
            }
            let vendor = CStr::from_ptr(dc_descriptor_get_vendor(desc))
                .to_string_lossy()
                .into_owned();
            let product = CStr::from_ptr(dc_descriptor_get_product(desc))
                .to_string_lossy()
                .into_owned();
            let transports = transport_names(dc_descriptor_get_transports(desc));
            infos.push(DescriptorInfo { vendor, product, transports });
            dc_descriptor_free(desc);
        }
        dc_iterator_free(iterator);
    }
    infos
}

pub fn all_descriptors() -> &'static Vec<DescriptorInfo> {
    DESCRIPTORS.get_or_init(load_descriptors)
}

pub fn vendors() -> Vec<String> {
    let mut vendors: Vec<String> = all_descriptors()
        .iter()
        .map(|d| d.vendor.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    vendors.sort();
    vendors
}

pub fn models_for_vendor(vendor: &str) -> Vec<DcModel> {
    all_descriptors()
        .iter()
        .filter(|d| d.vendor == vendor)
        .map(|d| DcModel {
            product: d.product.clone(),
            transports: d.transports.clone(),
        })
        .collect()
}

/// Walks every libdc descriptor reachable from `ctx_ptr`, calling `matches`
/// on each one. Returns the first descriptor `matches` accepts (ownership
/// passes to the caller, who must free it with `dc_descriptor_free`); every
/// rejected descriptor is freed internally. Shared by `find_descriptor` and
/// `resolve_descriptor_for_model`, which differ only in their predicate.
unsafe fn find_owned_descriptor(
    ctx_ptr: *mut dc_context_t,
    mut matches: impl FnMut(*mut dc_descriptor_t) -> bool,
) -> Option<*mut dc_descriptor_t> {
    let mut iterator = ptr::null_mut();
    if dc_descriptor_iterator_new(&mut iterator, ctx_ptr) != dc_status_t_DC_STATUS_SUCCESS {
        return None;
    }
    loop {
        let mut desc: *mut dc_descriptor_t = ptr::null_mut();
        let rc = dc_iterator_next(
            iterator,
            &mut desc as *mut *mut dc_descriptor_t as *mut _,
        );
        if rc != dc_status_t_DC_STATUS_SUCCESS {
            dc_iterator_free(iterator);
            return None;
        }
        if matches(desc) {
            dc_iterator_free(iterator);
            return Some(desc); // caller must free
        }
        dc_descriptor_free(desc);
    }
}

/// Returns a fresh `dc_descriptor_t*` for (vendor, product). Caller must free with
/// `dc_descriptor_free`. Returns None if no match found.
pub fn find_descriptor(vendor: &str, product: &str) -> Option<*mut dc_descriptor_t> {
    let ctx = DcContext::new().ok()?;
    unsafe {
        find_owned_descriptor(ctx.as_ptr(), |desc| {
            let v = CStr::from_ptr(dc_descriptor_get_vendor(desc)).to_string_lossy();
            let p = CStr::from_ptr(dc_descriptor_get_product(desc)).to_string_lossy();
            v == vendor && p == product
        })
    }
}

/// Finds the descriptor for a given family + libdc-reported model id, using
/// the caller's already-open context (no fresh `DcContext` is allocated).
///
/// Mirrors Qt Subsurface's auto-correction in `DC_EVENT_DEVINFO` handling
/// (`core/libdivecomputer.cpp`): when the device's self-reported `model` id
/// doesn't match the descriptor used to open the connection, the *actual*
/// descriptor must be looked up by (family, model) and used instead —
/// otherwise sibling descriptors (e.g. "Perdix" vs "Perdix AI", same family,
/// different model id, same wire protocol) silently fragment the fingerprint
/// namespace depending only on which one the user happened to pick.
///
/// Returns the owned descriptor together with its vendor/product strings:
/// callers need both the strings (for the fingerprint key) and the
/// descriptor itself (to keep `dc_parser_new2`-based dive parsing in sync
/// with the corrected identity — a caller that only swaps the string and
/// keeps parsing with the stale descriptor reintroduces the same class of
/// bug for the data itself). Caller must free the descriptor with
/// `dc_descriptor_free`.
pub fn resolve_descriptor_for_model(
    ctx_ptr: *mut dc_context_t,
    family: dc_family_t,
    model: u32,
) -> Option<(*mut dc_descriptor_t, String, String)> {
    unsafe {
        let desc = find_owned_descriptor(ctx_ptr, |desc| {
            dc_descriptor_get_type(desc) == family && dc_descriptor_get_model(desc) == model
        })?;
        let vendor = CStr::from_ptr(dc_descriptor_get_vendor(desc)).to_string_lossy().into_owned();
        let product = CStr::from_ptr(dc_descriptor_get_product(desc)).to_string_lossy().into_owned();
        Some((desc, vendor, product))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn vendor_list_is_non_empty_and_contains_shearwater() {
        let vendors = super::vendors();
        assert!(!vendors.is_empty(), "vendor list must not be empty");
        assert!(vendors.contains(&"Shearwater".to_string()), "Shearwater must be present");
    }

    #[test]
    fn shearwater_perdix_has_ble_transport() {
        let models = super::models_for_vendor("Shearwater");
        let perdix = models.iter().find(|m| m.product == "Perdix");
        assert!(perdix.is_some(), "Perdix must exist");
        let perdix = perdix.unwrap();
        assert!(perdix.transports.contains(&"BLE".to_string()), "Perdix must have BLE");
    }

    #[test]
    fn resolve_descriptor_for_model_finds_exact_sibling() {
        use crate::dc::ffi::{dc_family_t_DC_FAMILY_SHEARWATER_PETREL, dc_descriptor_get_model, dc_descriptor_free};
        // Model id 5 = "Perdix", id 6 = "Perdix AI" (libdc/src/descriptor.c).
        // A DEVINFO event reporting model 6 while connected via the "Perdix"
        // descriptor must resolve to the *actual* connected product, "Perdix AI" —
        // this is what lets the fingerprint cutoff stay keyed to the real device
        // regardless of which sibling entry the user picked in the download UI.
        let ctx = super::DcContext::new().unwrap();
        let (desc, vendor, product) =
            super::resolve_descriptor_for_model(ctx.as_ptr(), dc_family_t_DC_FAMILY_SHEARWATER_PETREL, 6)
                .expect("model 6 must resolve");
        assert_eq!(vendor, "Shearwater");
        assert_eq!(product, "Perdix AI");
        unsafe {
            assert_eq!(dc_descriptor_get_model(desc), 6, "returned descriptor must actually be the corrected one, not the original");
            dc_descriptor_free(desc);
        }
    }

    #[test]
    fn resolve_descriptor_for_model_returns_none_when_unknown() {
        use crate::dc::ffi::dc_family_t_DC_FAMILY_SHEARWATER_PETREL;
        let ctx = super::DcContext::new().unwrap();
        let resolved = super::resolve_descriptor_for_model(ctx.as_ptr(), dc_family_t_DC_FAMILY_SHEARWATER_PETREL, 9999);
        assert!(resolved.is_none());
    }
}
