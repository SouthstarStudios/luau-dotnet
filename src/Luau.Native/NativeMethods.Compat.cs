// NativeMethods.Compat
// Members of the package's C# API (0.1.6, Luau 0.678) that the Luau update would otherwise change, kept by the
// San Aurie fork (see FORK.md) so Luau.dll and existing callers keep working:
//   luaL_callyieldable became a macro for lua_callyieldable, so it binds to that export
//   lua_newuserdatadtor's destructor delegate kept its old nested type (the header now names it lua_Destructor)
// build.rs keeps the rest in the generated file (lua_unref returning void, lua_pushvfstring's va_list as byte*).

using System.Runtime.InteropServices;

namespace Luau.Native { public static unsafe partial class NativeMethods {

[DllImport(__DllName, EntryPoint = "ffi_lua_callyieldable", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
public static extern int luaL_callyieldable(lua_State* L, int nargs, int nresults);

[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
public delegate void lua_newuserdatadtor_dtor_delegate(void* arg1);

}}
