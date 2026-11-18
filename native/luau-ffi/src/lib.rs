#[allow(dead_code)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[allow(non_upper_case_globals)]
mod luau;

#[allow(dead_code)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[allow(non_upper_case_globals)]
mod luau_ffi;

#[allow(dead_code)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[allow(non_upper_case_globals)]
mod luau_require;

#[allow(dead_code)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[allow(non_upper_case_globals)]
mod luau_require_ffi;

#[allow(dead_code)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[allow(non_upper_case_globals)]
mod luau_codegen;

// San Aurie fork: hand-written exports for native code generation
mod luau_codegen_ffi;

#[cfg(test)]
mod test {
    use crate::luau::*;
    use crate::luau_codegen_ffi::*;

    #[test]
    pub fn sandbox() {
        unsafe {
            let l = luaL_newstate();
            lua_close(l);
        }
    }

    extern "C" {
        fn free(p: *mut ::std::os::raw::c_void);
    }

    // San Aurie fork: the Luau update's built-ins, and native code generation where the platform has it
    #[test]
    pub fn builtins_and_codegen() {
        unsafe {
            let l = luaL_newstate();
            luaL_openlibs(l);
            let source = b"local v = vector.lerp(vector.create(0, 0, 0), vector.create(2, 4, 6), 0.5)
local function sum(n) local t = 0 for i = 1, n do t = t + i end return t end
return math.isnan(0 / 0) and math.isinf(math.huge) and math.isfinite(1) and not math.isfinite(1 / 0)
    and v.x == 1 and v.y == 2 and v.z == 3 and sum(100) == 5050";
            let mut size: usize = 0;
            let bytecode = luau_compile(source.as_ptr() as *const _, source.len(), std::ptr::null_mut(), &mut size);
            let name = b"=test\0";
            assert_eq!(luau_load(l, name.as_ptr() as *const _, bytecode, size, 0), 0);
            free(bytecode as *mut _);
            let supported = ffi_luau_codegen_supported();
            println!("luau_codegen_supported = {}", supported);
            if supported != 0 {
                ffi_luau_codegen_create(l);
                ffi_luau_codegen_compile(l, -1);
            }
            assert_eq!(lua_pcall(l, 0, 1, 0), 0);
            assert_eq!(lua_toboolean(l, -1), 1);
            lua_close(l);
        }
    }
}