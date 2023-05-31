// SPDX-FileCopyrightText: Copyright 2023 Savi
// SPDX-License-Identifier: GPL-3.0-only 

fn main() {
    let config =
        slint_build::CompilerConfiguration::new()
        .with_style("fluent-dark".into());
    slint_build::compile_with_config("ui/app.slint", config).unwrap();
    slint_build::print_rustc_flags().unwrap();
}
