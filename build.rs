/*
Original Author copyright follows, all code in the below file falls under
the terms of the MIT licence as written below
*/
/*
Copyright (c) 2015 Peter Marheine

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the "Software"), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
of the Software, and to permit persons to whom the Software is furnished to do
so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/


//FIXME: This file needs a cleanup
#[cfg(feature = "build-c-libs")]
extern crate semver;

#[cfg(feature = "build-c-libs")]
use semver::{Version, VersionReq};
#[cfg(feature = "build-c-libs")]
use std::process::Command;
#[cfg(feature = "build-c-libs")]
use std::env;

#[cfg(feature = "build-c-libs")]
const LLVM_CONFIG_NAME : &'static str =  "llvm-config-3.8";
#[cfg(feature = "build-c-libs")]
const CLANG : &'static str =  "clang++-3.8";
#[cfg(feature = "build-c-libs")]
const LLVM_CONFIG_ADDITIONAL: &'static str = "core mcjit native";


/// Get the output from running `llvm-config` with the given argument.
#[cfg(feature = "build-c-libs")]
fn llvm_config(arg: &str, addition_config:bool) -> String {
    let additional_config = if addition_config { LLVM_CONFIG_ADDITIONAL} else {" "};
    let output = Command::new(LLVM_CONFIG_NAME)
        .arg(arg)
        .output()
        .unwrap_or_else(|e| panic!("Couldn't execute llvm-config. Error: {}", e));

    let stderr = output.stderr;
    let string = String::from_utf8(stderr).ok().expect("llvm-config output was not UTF-8.");
    if string.len() > 0 {panic!("unexpected error messages from llvm config: {} \n with arg: {}", string, arg);}
    let stdout = output.stdout;
    let string = String::from_utf8(stdout).ok().expect("llvm-config output was not UTF-8.");
    return string;

}

/// Get the LLVM version using llvm-config.
#[cfg(feature = "build-c-libs")]
fn llvm_version() -> Version {
    match Version::parse(&llvm_config("--version",false)) {
        // Ignore partial error; particularly constructs like '3.8.0svn' should be accepted,
        // despite being invalid semver.
        Ok(v) => v,
        _ => panic!("Could not determine LLVM version from llvm-config."),
    }
}

#[cfg(feature = "build-c-libs")]
fn main() {
    // Check for LLVM 3.6 or greater.
    let minimum_llvm_version = VersionReq::parse(">=3.6").unwrap();
    let version = llvm_version();
    if minimum_llvm_version.matches(&version) {
        println!("Found LLVM version {}", version);
    } else {
        panic!("LLVM version 3.6 or higher is required. (Found {})", version);
    };

    // Parse library linking flags from llvm-config.
    for arg in llvm_config("--ldflags",true).split_whitespace() {
        if arg.starts_with("-L") {
            println!("cargo:rustc-link-search=native={}", &arg[2..]);
        }
    }

    for arg in llvm_config("--libs",true).split_whitespace() {
        if arg.starts_with("-l") {
            println!("cargo:rustc-link-lib={}", &arg[2..]);
        }
    }

    for arg in llvm_config("--system-libs",true).split_whitespace() {
        if arg.starts_with("-l") {
            println!("cargo:rustc-link-lib=dylib={}", &arg[2..]);
        }
    }
    let cxxflags = llvm_config("--cxxflags",true);

    // This breaks the link step on Windows with MSVC.
    if !cfg!(windows) {
        // Determine which C++ standard library to use: LLVM's or GCC's.
        let libcpp = if cxxflags.contains("stdlib=libc++") { "c++" } else { "stdc++" };
        println!("cargo:rustc-link-lib={}", libcpp);
    }



    let mut clang_run = Command::new(CLANG);
    for arg in cxxflags.split_whitespace()
    {
        clang_run.arg(arg);
    }

    clang_run
        .arg("-c")
        .arg("-O0")
        .arg("src/cpp/kaleidoscope.cpp")
        .arg("-o")
        .arg("kaleidoscope.o");
    if !clang_run.status().unwrap().success(){
        panic!("failed to generate c lib")
    }

    Command::new("ar").args(&["crus", "libkaleidoscope.a", "kaleidoscope.o"])
            .status().unwrap();

    println!("cargo:rustc-link-search=native={}",  env::current_dir().as_ref().unwrap().to_str().unwrap());
    println!("cargo:rustc-link-lib=static=kaleidoscope");

}

#[cfg(not(feature = "build-c-libs"))]
fn main()
{

}
