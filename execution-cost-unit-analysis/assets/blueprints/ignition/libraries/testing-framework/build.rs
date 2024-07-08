// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

fn main() {
    decompress_state();
}

fn decompress_state() {
    use flate2::read::*;

    use std::env;
    use std::io::prelude::*;
    use std::path::*;
    use std::str::FromStr;

    println!("cargo:rerun-if-changed=\"./assets/state\"");
    let compressed = include_bytes!("./assets/state");
    let mut decoder = GzDecoder::new(&compressed[..]);
    let mut uncompressed = Vec::new();
    decoder
        .read_to_end(&mut uncompressed)
        .expect("Failed to decompress!");

    let path = PathBuf::from_str(env::var("OUT_DIR").unwrap().as_str())
        .unwrap()
        .join("uncompressed_state.bin");
    std::fs::write(path, uncompressed).unwrap();
}
