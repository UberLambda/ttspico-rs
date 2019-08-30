// Copyright (c) 2019 Paolo Jovon <paolo.jovon@gmail.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use cc;
use glob::glob;

const PICO_SRC_DIR: &str = "build/pico/lib";
const PICO_LIB_NAME: &str = "svoxpico";

fn main() {
    let glob_pat = format!("{}/*.c", PICO_SRC_DIR);
    let src_files = glob(glob_pat.as_str())
        .expect("Failed to list *.c files")
        .map(|p| p.unwrap());

    cc::Build::new()
        .include(PICO_SRC_DIR)
        .files(src_files)
        .warnings(false)
        .extra_warnings(false)
        .compile(PICO_LIB_NAME); // (static library)

    println!("cargo:rustc-link-lib=static={}", PICO_LIB_NAME);
}
