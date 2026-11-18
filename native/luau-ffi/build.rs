use std::env;

fn main() {
    let target = build_target::target_triple().unwrap();
    if target == "wasm32-unknown-emscripten" {
        if let Ok(em_dir) = env::var("EM_DIR") {
            fn exec_path(path: &str) -> String {
                if cfg!(windows) {
                    format!("{path}.bat")
                } else {
                    path.to_string()
                }
            }
            env::set_var(
                "EMCMAKE",
                exec_path(&format!("{em_dir}/emscripten/emcmake")),
            );
            env::set_var("EMMAKE", exec_path(&format!("{em_dir}/emscripten/emmake")));
            env::set_var("EM_CONFIG", format!("{em_dir}/.emscripten"));
        }
    }

    let dst = new_cmake_config().build_target("Luau.Compiler").build();
    println!(
        "cargo:warning=CMake configure (Luau.Compiler) completed: {}",
        dst.display()
    );

    let dst = new_cmake_config().build_target("Luau.Require").build();
    println!(
        "cargo:warning=CMake configure (Luau.Require) completed: {}",
        dst.display()
    );

    // San Aurie fork: native code generation (luau_codegen_*)
    let dst = new_cmake_config().build_target("Luau.CodeGen").build();
    println!(
        "cargo:warning=CMake configure (Luau.CodeGen) completed: {}",
        dst.display()
    );

    let target = build_target::target_triple().unwrap();
    let android = target.ends_with("-linux-android");
    if target == "x86_64-pc-windows-msvc" {
        println!("cargo:rustc-link-search=native={}/build/Release", dst.display());
    } else {
        println!("cargo:rustc-link-search=native={}/build", dst.display());
    }

    // Dependents before their dependencies (GNU ld resolves static libraries in order). Luau 0.7xx merged
    // RequireNavigator into Require and split Bytecode and Common out of the compiler.
    println!("cargo:rustc-link-lib=static=Luau.Require");
    println!("cargo:rustc-link-lib=static=Luau.Config");
    println!("cargo:rustc-link-lib=static=Luau.Compiler");
    println!("cargo:rustc-link-lib=static=Luau.CodeGen");
    println!("cargo:rustc-link-lib=static=Luau.VM");
    println!("cargo:rustc-link-lib=static=Luau.Ast");
    println!("cargo:rustc-link-lib=static=Luau.Bytecode");
    println!("cargo:rustc-link-lib=static=Luau.Common");

    // The C++ runtime last, after everything that needs it (rustc links with --as-needed, which drops a shared
    // library named before its users)
    if target == "aarch64-unknown-linux-gnu" || target == "x86_64-unknown-linux-gnu" {
        // Luau is compiled with g++ here
        println!("cargo:rustc-link-lib=dylib=stdc++");
    } else if android {
        // libc++ linked statically (CMAKE_ANDROID_STL_TYPE c++_static), so the .so doesn't need libc++_shared.so,
        // which Unity's Android player doesn't ship
        // Only these two archives are copied to a private folder: the NDK folder that holds them also has the static
        // libc.a, which must never end up in a shared library
        let ndk_home = env::var("ANDROID_NDK_HOME").unwrap();
        let arch = if target.starts_with("aarch64") { "aarch64" } else { "x86_64" };
        let ndk_lib = format!(
            "{}/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/{}-linux-android",
            ndk_home, arch
        );
        let cxx_dir = format!("{}/cxx", env::var("OUT_DIR").unwrap());
        std::fs::create_dir_all(&cxx_dir).unwrap();
        for lib in ["libc++_static.a", "libc++abi.a"] {
            std::fs::copy(format!("{}/{}", ndk_lib, lib), format!("{}/{}", cxx_dir, lib)).unwrap();
        }
        println!("cargo:rustc-link-search=native={}", cxx_dir);
        println!("cargo:rustc-link-lib=static=c++_static");
        println!("cargo:rustc-link-lib=static=c++abi");
    } else if target != "x86_64-pc-windows-msvc" {
        println!("cargo:rustc-link-lib=dylib=c++");
    }

    // bindgen parses the headers for the target; Android needs the NDK's sysroot instead of the host's headers
    let mut clang_args = vec![format!("--target={}", target)];
    if android {
        clang_args.push(format!(
            "--sysroot={}/toolchains/llvm/prebuilt/linux-x86_64/sysroot",
            env::var("ANDROID_NDK_HOME").unwrap()
        ));
    }

    bindgen::Builder::default()
        .headers([
            "../../luau/VM/include/lua.h",
            "../../luau/VM/include/lualib.h",
            "../../luau/Compiler/include/luacode.h",
        ])
        .clang_args(&clang_args)
        .clang_arg("-fvisibility=default")
        .layout_tests(false)
        .generate()
        .unwrap()
        .write_to_file("src/luau.rs")
        .unwrap();

    bindgen::Builder::default()
        .headers(["../../luau/Require/include/Luau/Require.h"])
        .clang_arg("-x")
        .clang_arg("c++")
        .clang_arg("-std=c++17")
        .clang_arg("-I../../luau/Compiler/include")
        .clang_arg("-I../../luau/VM/include")
        .clang_arg("-I../../luau/Require/include")
        .clang_arg("-DLUA_API=extern\"C\"")
        .allowlist_function("luarequire_.*")
        .allowlist_function("luaopen_require")
        .allowlist_type("luarequire_.*")
        .blocklist_type("lua_State")
        .raw_line("use super::luau::*;")
        .clang_args(&clang_args)
        .clang_arg("-fvisibility=default")
        .layout_tests(false)
        .generate()
        .unwrap()
        .write_to_file("src/luau_require.rs")
        .unwrap();

    // Code generation: bindings only. The exports (src/luau_codegen_ffi.rs) and the C# side
    // (NativeMethods.CodeGen.cs) are written by hand, so iOS can report "not supported".
    bindgen::Builder::default()
        .headers(["../../luau/CodeGen/include/luacodegen.h"])
        .allowlist_function("luau_codegen_.*")
        .blocklist_type("lua_State")
        .raw_line("use super::luau::*;")
        .clang_args(&clang_args)
        .clang_arg("-fvisibility=default")
        .layout_tests(false)
        .generate()
        .unwrap()
        .write_to_file("src/luau_codegen.rs")
        .unwrap();

    let cs = new_csbindgen_builder("src/luau.rs")
        .method_filter(keep_method)
        .rust_file_header("use super::luau::*;")
        .csharp_file_header(
            "
using lua_newstate_f_delegate = Luau.Native.lua_Alloc;
using lua_newstate_allocator_delegate = Luau.Native.lua_Alloc;
using lua_pushcclosurek_fn__delegate = Luau.Native.lua_CFunction;
using lua_tocfunction_return_delegate = Luau.Native.lua_CFunction;
using lua_pushcclosurek_cont_delegate = Luau.Native.lua_Continuation;
using lua_setuserdatadtor_dtor_delegate = Luau.Native.lua_Destructor;
using lua_getuserdatadtor_return_delegate = Luau.Native.lua_Destructor;
using lua_getallocf_return_delegate = Luau.Native.lua_Alloc;
using lua_getcoverage_callback_delegate = Luau.Native.lua_Coverage;
",
        );

    cs.generate_to_file(
        "src/luau_ffi.rs",
        "../../src/Luau.Native/NativeMethods.g.cs",
    )
    .unwrap();

    cs.csharp_dll_name_if("(UNITY_IOS || UNITY_WEBGL) && !UNITY_EDITOR", "__Internal")
        .generate_csharp_file("../../src/Luau.Unity/Assets/Luau.Unity/Native/NativeMethods.g.cs")
        .unwrap();
    keep_csharp_api("../../src/Luau.Native/NativeMethods.g.cs");
    keep_csharp_api("../../src/Luau.Unity/Assets/Luau.Unity/Native/NativeMethods.g.cs");

    let cs2 = new_csbindgen_builder("src/luau_require.rs")
        .rust_file_header(
            "
use super::luau::*;
use super::luau_require::*;
",
        )
        .csharp_disable_emit_dll_name(true)
        .csharp_file_header(
            "
using luarequire_pushrequire_config_init_delegate = Luau.Native.luarequire_Configuration_init;     
using luaopen_require_config_init_delegate = Luau.Native.luarequire_Configuration_init;    
using luarequire_pushproxyrequire_config_init_delegate = Luau.Native.luarequire_Configuration_init;   
",
        );

    cs2.generate_to_file(
        "src/luau_require_ffi.rs",
        "../../src/Luau.Native/NativeMethods.Require.g.cs",
    )
    .unwrap();

    cs2.generate_csharp_file(
        "../../src/Luau.Unity/Assets/Luau.Unity/Native/NativeMethods.Require.g.cs",
    )
    .unwrap();
}

// San Aurie fork: functions new in Luau 0.7xx whose callback parameters are typedefs csbindgen can't name in C#.
// They are left out of the bindings (and the ffi_ exports) until something needs them.
const UNBOUND: &[&str] = &[
    "lua_cpcall",
    "lua_memorydump",
    "lua_setuserdatamark",
    "lua_setembeddergc",
    "lua_registeruserdatadirectaccess",
    "lua_registeruserdatadirectfieldget",
    "lua_callhook",
    "lua_getcounters",
    "lua_setmemorycage",
];

fn keep_method(name: String) -> bool {
    !UNBOUND.contains(&name.as_str())
}

// San Aurie fork: the Luau update changed a few signatures that Luau.dll and other callers were compiled against.
// The generated C# keeps the old ones where the native call is the same (NativeMethods.Compat.cs has the rest):
//   lua_unref now returns int (the C# side keeps void; an ignored return value is safe)
//   lua_pushvfstring's va_list is a pointer on every target (bindgen spells it per platform)
fn keep_csharp_api(path: &str) {
    let text = std::fs::read_to_string(path).unwrap();
    let text = text
        .replace(
            "public static extern int lua_unref(lua_State* L, int ref_);",
            "public static extern void lua_unref(lua_State* L, int ref_);",
        )
        .replace("__va_list_tag* argp", "byte* argp");
    std::fs::write(path, text).unwrap();
}

fn new_cmake_config() -> cmake::Config {
    let mut config = cmake::Config::new("../../luau");
    // extern "C" APIs with LUA_USE_LONGJMP=1 (errors longjmp instead of throwing C++ exceptions)
    config.define("LUAU_EXTERN_C", "ON");
    config.define("LUAU_BUILD_CLI", "OFF");
    config.define("LUAU_BUILD_TESTS", "OFF");

    let target = build_target::target_triple().unwrap();

    if target == "x86_64-pc-windows-msvc" {
        if let Ok(cc) = std::env::var("CC") {
            if !cc.is_empty() {
                config.define("CMAKE_C_COMPILER", cc);
            }
        }
        if let Ok(cxx) = std::env::var("CXX") {
            if !cxx.is_empty() {
                config.define("CMAKE_CXX_COMPILER", cxx);
            }
        }

        config.cxxflag("/EHsc");
    } else if target == "aarch64-unknown-linux-gnu" {
        config.define("CMAKE_SYSTEM_NAME", "Linux");
        config.define("CMAKE_SYSTEM_PROCESSOR", "aarch64");
        config.define("CMAKE_C_FLAGS", "-ffunction-sections -fdata-sections -fPIC");
        config.define(
            "CMAKE_CXX_FLAGS",
            "-ffunction-sections -fdata-sections -fPIC",
        );
        if let Ok(cc) = std::env::var("CC") {
            if !cc.is_empty() {
                config.define("CMAKE_C_COMPILER", cc);
            }
        }
        if let Ok(cxx) = std::env::var("CXX") {
            if !cxx.is_empty() {
                config.define("CMAKE_CXX_COMPILER", cxx);
            }
        }
    } else if target == "aarch64-apple-darwin" {
        config.define("CMAKE_OSX_ARCHITECTURES", "arm64");
        config.define("CMAKE_OSX_DEPLOYMENT_TARGET", "11.0");
    } else if target == "x86_64-apple-darwin" {
        config.define("CMAKE_OSX_ARCHITECTURES", "x86_64");
        config.define("CMAKE_OSX_DEPLOYMENT_TARGET", "10.15");
    } else if target == "x86_64-apple-ios" {
        config.define("CMAKE_SYSTEM_NAME", "iOS");
        config.define("CMAKE_SYSTEM_PROCESSOR", "x86_64");
        config.define("CMAKE_OSX_ARCHITECTURES", "x86_64");
        config.define("CMAKE_OSX_SYSROOT", "/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneSimulator.platform/Developer/SDKs/iPhoneSimulator.sdk");
        config.define(
            "CMAKE_C_FLAGS",
            "-fPIC -m64 --target=x86_64-apple-ios-simulator -mios-simulator-version-min=17.5",
        );
        config.define(
            "CMAKE_CXX_FLAGS",
            "-fPIC -m64 --target=x86_64-apple-ios-simulator -mios-simulator-version-min=17.5",
        );
    } else if target == "aarch64-apple-ios" {
        config.define("CMAKE_SYSTEM_NAME", "iOS");
        config.define("CMAKE_SYSTEM_PROCESSOR", "arm64");
        config.define("CMAKE_OSX_ARCHITECTURES", "arm64");
        config.define("CMAKE_OSX_SYSROOT", "/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS.sdk");
        config.define(
            "CMAKE_C_FLAGS",
            "-fPIC --target=arm64-apple-ios -miphoneos-version-min=17.5",
        );
        config.define(
            "CMAKE_CXX_FLAGS",
            "-fPIC --target=arm64-apple-ios -miphoneos-version-min=17.5",
        );
    } else if target.ends_with("-linux-android") {
        // The NDK's own toolchain file (the cmake crate leaves the compilers to it), libc++ static, API 26
        let ndk_home = std::env::var("ANDROID_NDK_HOME").unwrap();
        let abi = if target.starts_with("aarch64") { "arm64-v8a" } else { "x86_64" };
        config.define(
            "CMAKE_TOOLCHAIN_FILE",
            format!("{}/build/cmake/android.toolchain.cmake", ndk_home),
        );
        config.define("ANDROID_ABI", abi);
        config.define("ANDROID_PLATFORM", "android-26");
        config.define("ANDROID_STL", "c++_static");
    } else if target == "wasm32-unknown-emscripten" {
        if let Ok(em_dir) = env::var("EM_DIR") {
            // By default cmake crate overrides compiler paths with not qualified ones, causing missing compiler errors with no emscripten in PATH.
            config.define(
                "CMAKE_C_COMPILER",
                format!("{}/emscripten/emcc", em_dir),
            );
            config.define(
                "CMAKE_CXX_COMPILER",
                format!("{}/emscripten/em++", em_dir),
            );
        }
        config.define("CMAKE_C_FLAGS", "-ffunction-sections -fdata-sections -fPIC");
        config.define(
            "CMAKE_CXX_FLAGS",
            "-ffunction-sections -fdata-sections -fPIC",
        );
    }

    config
}

fn new_csbindgen_builder(src: &'static str) -> csbindgen::Builder {
    csbindgen::Builder::default()
        .input_bindgen_file(src)
        .rust_method_prefix("ffi_")
        .csharp_entry_point_prefix("ffi_")
        .csharp_method_prefix("")
        .csharp_namespace("Luau.Native")
        .csharp_dll_name("libluau")
        .csharp_class_accessibility("public")
        .csharp_generate_const_filter(|x| x.starts_with("LUA"))
        .csharp_use_function_pointer(false)
}
