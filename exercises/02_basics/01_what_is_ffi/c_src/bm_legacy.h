#pragma once

/* A tiny slice of the legacy bookmark-manager C API. We'll be calling these
 * from Rust. Header is hand-written for now; in 03_bindgen we'll let bindgen
 * generate the Rust extern block from this file directly. */

int bm_add(int a, int b);
