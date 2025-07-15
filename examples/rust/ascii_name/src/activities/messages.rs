// Copyright 2025 Accenture.
//
// SPDX-License-Identifier: Apache-2.0

/// AsciiArt message
/// Contains the complete set of ASCII art lines
#[derive(Debug, Default, Clone)]
#[repr(C)]
pub struct AsciiArt {
    pub lines: Vec<String>,
}
