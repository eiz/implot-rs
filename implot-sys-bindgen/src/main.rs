use bindgen::{Builder, CargoCallbacks};
use std::{env, io::Write, path::PathBuf};

// All this crate does is run bindgen on cimplot and store the result
// in the src folder of the implot-sys crate. We add those bindings
// to git so people don't have to install clang just to use implot-rs.

fn main() {
    let cwd = env::current_dir().expect("Could not read current directory");
    let sys_crate_path = cwd
        .join("..")
        .join("implot-sys")
        .canonicalize()
        .expect("Could not find sys crate directory");

    let cimgui_include_path = PathBuf::from(
        env::var_os("DEP_IMGUI_THIRD_PARTY").expect("DEP_IMGUI_THIRD_PARTY not defined"),
    );

    println!["{:?}", cimgui_include_path.join("cimgui.h")];
    let bindings = Builder::default()
        .header(
            cimgui_include_path
                .join("cimgui.h")
                .to_str()
                .expect("Could not convert cimgui.h path to string"),
        )
        .header(
            sys_crate_path
                .join("third-party")
                .join("cimplot")
                .join("cimplot.h")
                .to_str()
                .expect("Could not turn cimplot.h path into string"),
        )
        .parse_callbacks(Box::new(CargoCallbacks))
        .clang_arg(format!(
            "-I{}",
            cimgui_include_path.as_os_str().to_str().unwrap()
        ))
        .clang_arg("-DCIMGUI_DEFINE_ENUMS_AND_STRUCTS=1")
        // Reuse the imgui types that implot requires from imgui_sys so we don't define
        // our own new types.
        .raw_line("pub use imgui_sys::{ImRect, ImVec2, ImVec4, ImGuiCond, ImTextureID};")
        .raw_line("pub use imgui_sys::{ImGuiContext, ImGuiID, ImGuiKeyModFlags, ImGuiStorage, ImGuiTextBuffer, ImDrawList};")
        .raw_line("pub use imgui_sys::{ImGuiMouseButton, ImGuiDragDropFlags};")
        .raw_line("pub use imgui_sys::{ImPoolIdx, ImVector_ImGuiColorMod, ImVector_ImGuiStyleMod};")
        .raw_line("pub use libc::{time_t, tm};")
        .whitelist_recursively(false)
        .whitelist_function("ImPlot.*")
        .whitelist_type("ImPlot.*")
        // We do want to create bindings for the scalar typedefs
        .whitelist_type("Im[U|S][0-9]{1,2}")
        .whitelist_type("ImAxis.*")
        .whitelist_type("ImPool_ImPlot.*")
        .whitelist_type("ImVector_(bool|int|float|double|ImS8|ImU8|ImS16|ImU16|ImS32|ImU32|ImS64|ImU64)")
        .whitelist_type("ImVector_ImPlot.*")
        // Remove some functions that would take a variable-argument list
        .blacklist_function("ImPlot_AnnotateVVec4")
        .blacklist_function("ImPlot_AnnotateVStr")
        .blacklist_function("ImPlot_AnnotateClampedVVec4")
        .blacklist_function("ImPlot_AnnotateClampedVStr")
        .blacklist_function("ImPlot_AnnotationV")
        .blacklist_function("ImPlotAnnotationCollection_AppendV")
        .blacklist_function("ImPlotTagCollection_AppendV")
        .blacklist_function("ImPlot_TagXV")
        .blacklist_function("ImPlot_TagYV")
        .generate()
        .expect("Unable to generate bindings");

    // The above type re-export shenanigans make bindgen unable to derive Copy, Clone and Debug on
    // some types, but they would work - we hence manually re-add them here.
    let mut bindings_string = bindings.to_string();
    ["ImPlotInputMap", "ImPlotStyle"].iter().for_each(|name| {
        bindings_string = bindings_string.replace(
            &format!("pub struct {}", name),
            &format!("#[derive(Clone, Copy, Debug)]\npub struct {}", name),
        );
    });

    // Finally we write the bindings to a file.
    let out_path = sys_crate_path.join("src");
    let mut out_file =
        std::fs::File::create(&out_path.join("bindings.rs")).expect("Could not open bindings file");
    out_file
        .write_all(&bindings_string.into_bytes()[..])
        .expect("Couldn't write bindings");
}
