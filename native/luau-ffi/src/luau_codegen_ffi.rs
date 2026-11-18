// Exports for Luau's native code generation (luacodegen.h), next to the generated ffi_* ones.
// Written by hand rather than by csbindgen so iOS can always report "not supported": the platform doesn't
// allow generated machine code, and the same scripts run on the interpreter there.

use super::luau::*;
use super::luau_codegen::*;

#[no_mangle]
pub unsafe extern "C" fn ffi_luau_codegen_supported() -> i32 {
    if cfg!(target_os = "ios") {
        return 0;
    }
    luau_codegen_supported()
}

#[no_mangle]
pub unsafe extern "C" fn ffi_luau_codegen_create(L: *mut lua_State) {
    if cfg!(target_os = "ios") {
        return;
    }
    luau_codegen_create(L)
}

#[no_mangle]
pub unsafe extern "C" fn ffi_luau_codegen_compile(L: *mut lua_State, idx: i32) {
    if cfg!(target_os = "ios") {
        return;
    }
    luau_codegen_compile(L, idx)
}
