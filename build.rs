use reqwest::blocking::get;
use std::env;
use std::fs::File;
use std::io::copy;
use std::path::{Path, PathBuf};
use std::process::Command;

const ARDUINO_URL: &str =
    "https://github.com/bblanchon/ArduinoJson/releases/download/v7.0.3/ArduinoJson-v7.0.3.h";

fn main_bind() {
    // Path to store the downloaded file
    //let out_dir = env::var("OUT_DIR").unwrap();
    //let file_path = Path::new(&out_dir).join("ArduinoJson-v7.0.3.h");
    let file_path = Path::new("cpp/arduinojson.h");

    let response = get(ARDUINO_URL).expect("Failed to download file");
    let mut out_file = File::create(file_path).expect("Failed to create file");
    unsafe { env::set_var("ARDUINO_LIB", file_path) };
    //panic!("response: {response:?}");
    copy(&mut response.bytes().unwrap().as_ref(), &mut out_file).expect("Failed to copy content");

    println!("cargo:rustc-link-search=/cpp");
    //println!("cargo:rustc-link-lib=arduinojson.h");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .clang_args(["-x", "c++", "-std=c++20"])
        .header("cpp/arduinojson.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        //.blocklist_type("int_type")
        //.allowlist_type("std::*")
        //.allowlist_type("*")
        .allowlist_item("Json*")
        .allowlist_recursively(true)
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    println!("cargo:rerun-if-changed=build.rs");
    //println!("cargo:rerun-if-changed=main.rs");
}

// _________________________________________________________________________________________________________________________

fn main() {
    // URL of the file to download

    // Path to store the downloaded file
    //let out_dir = env::var("OUT_DIR").unwrap();
    //let file_path = Path::new(&out_dir).join("ArduinoJson-v7.0.3.h");

    let file_path = Path::new("cpp/arduinojson.h");

    let response = get(ARDUINO_URL).expect("Failed to download file");
    let mut out_file = File::create(file_path).expect("Failed to create file");

    unsafe { env::set_var("ARDUINO_LIB", file_path) };
    //panic!("response: {response:?}");
    copy(&mut response.bytes().unwrap().as_ref(), &mut out_file).expect("Failed to copy content");

    //panic!("response: {response:?}");
    //panic!("file: {file_path:?}");

    /*
    cc::Build::new()
        .cpp(true)
        .static_flag(true)
        .cpp_link_stdlib("libc++")
        //.compiler("g++")
        .archiver("llvm-lib")
        //.ar_flag("rcs")
        //.ranlib("ranlib")
        //.flag("--std=c++20")
        .file(file_path) // Adjust path as needed
        .compile("libarduinojson");
    */

    let Some(objfile) = cc::Build::new()
        .cpp(true)
        //.archiver("/usr/bin/ar")
        //.ranlib("/usr/bin/ranlib")
        .file(file_path) // Adjust path as needed
        //.include(file) // Include path if required
        .std("c++20")
        //.flag_if_supported("-Werror")
        .compile_intermediates()
        .first()
        .cloned()
    else {
        panic!("No object files")
    };
    //.compile("arduinojson");

    //let lib_path = Path::new(&out_dir).join("libarduinojson.a");
    let lib_path = Path::new("cpp/libarduinojson.a");

    /*
        Command::new("g++")
            .args(["-c", file_path.as_path().to_str().unwrap(), "-o"])
            .arg(out_dir.clone() + "/arduinojson.o")
            .status()
            .expect("Failed to compile library");
    */
    // Create the static library using `ar`
    if !Command::new("ar")
        .args(["rcs", lib_path.to_str().unwrap(), objfile.to_str().unwrap()])
        .status()
        .expect("Failed to create static library with ar")
        .success()
    {
        panic!("ar failed");
    }

    if !Command::new("ranlib")
        .arg(lib_path.to_str().expect("Invalid path"))
        .status()
        .expect("Failed to run ranlib on static library")
        .success()
    {
        panic!("ranlib failed to add index to static library");
    }

    // Tell Cargo to re-run build if build.rs changes
    println!("cargo:rerun-if-changed=build.rs");
    //println!("cargo:rustc-link-search=native={}", out_dir);
    //println!("cargo:rustc-link-lib=static=arduinojson");
}
