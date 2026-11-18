// NativeMethods.CodeGen
// Luau's native code generation (luacodegen.h), added by the San Aurie fork of this package (see FORK.md). The exports
// are written by hand in native/luau-ffi/src/luau_codegen_ffi.rs, not generated, so csbindgen doesn't emit these.
// Call luau_codegen_create once per VM, and only where luau_codegen_supported returns 1 (always 0 on iOS, which doesn't
// allow generated machine code). luau_codegen_compile compiles the function at idx and every function inside it.

using System.Runtime.InteropServices;

namespace Luau.Native { public static unsafe partial class NativeMethods {

[DllImport(__DllName, EntryPoint = "ffi_luau_codegen_supported", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
public static extern int luau_codegen_supported();

[DllImport(__DllName, EntryPoint = "ffi_luau_codegen_create", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
public static extern void luau_codegen_create(lua_State* L);

[DllImport(__DllName, EntryPoint = "ffi_luau_codegen_compile", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
public static extern void luau_codegen_compile(lua_State* L, int idx);

}}
