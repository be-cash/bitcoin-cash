use proc_macro2::TokenStream;
use quote::quote;
use regex::Regex;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn format_stmts<'a>(
    max_line_widths: &[u32],
    stmts: impl Iterator<Item = &'a TokenStream>,
) -> Result<Vec<Vec<String>>, String> {
    let indent = Regex::new(r"\n {4}").unwrap();
    let separator =
        Regex::new(r"\s*__BITCOIN_CASH_SCRIPT_MACRO_LINE_SEPARATOR__\s*\(\s*\)\s*;?").unwrap();
    let token_stream = quote! {
        fn main() {
            __BITCOIN_CASH_SCRIPT_MACRO_LINE_SEPARATOR__();
            #(#stmts; __BITCOIN_CASH_SCRIPT_MACRO_LINE_SEPARATOR__());*
        }
    };
    let mut stmt_lines = Vec::with_capacity(max_line_widths.len());
    let dedent =
        |s: &str| -> String { indent.replace_all(&indent.replace(s, ""), "\n").to_string() };
    for formatted in format_token_stream(max_line_widths, &token_stream)? {
        let mut lines = separator
            .split(&formatted)
            .skip(1)
            .map(dedent)
            .collect::<Vec<_>>();
        lines.remove(lines.len() - 1);
        stmt_lines.push(lines);
    }
    Ok(stmt_lines)
}

fn format_token_stream(
    max_line_widths: &[u32],
    token_stream: &TokenStream,
) -> Result<Vec<String>, String> {
    let rustfmt = which_rustfmt().ok_or_else(|| "No rustfmt could be found.".to_string())?;
    let mut builder = tempfile::Builder::new();
    builder.prefix("cargo-expand");
    let outdir = builder.tempdir().map_err(fmt_err)?;
    let outfile_path = outdir.path().join("expanded");

    let unformatted = token_stream.to_string();
    let mut formatted = Vec::with_capacity(max_line_widths.len());
    for &max_line_width in max_line_widths {
        fs::write(&outfile_path, &unformatted).map_err(fmt_err)?;
        write_rustfmt_config(max_line_width + 4, &outdir).map_err(fmt_err)?;

        let status = Command::new(&rustfmt)
            .arg("--edition=2018")
            .arg(&outfile_path)
            .status()
            .map_err(fmt_err)?;
        if !status.success() {
            return Err("Rustfmt failed.".to_string());
        }
        formatted.push(fs::read_to_string(&outfile_path).map_err(fmt_err)?);
    }

    Ok(formatted)
}

fn fmt_err(err: impl std::fmt::Display) -> String {
    err.to_string()
}

fn which_rustfmt() -> Option<PathBuf> {
    match env::var_os("RUSTFMT") {
        Some(which) => {
            if which.is_empty() {
                None
            } else {
                Some(PathBuf::from(which))
            }
        }
        None => toolchain_find::find_installed_component("rustfmt"),
    }
}

fn write_rustfmt_config(max_line_width: u32, outdir: impl AsRef<Path>) -> std::io::Result<()> {
    let config_str = format!(
        "\
        max_width = {}
        reorder_imports = false
        reorder_modules = false
    ",
        max_line_width
    );

    let rustfmt_config_path = outdir.as_ref().join("rustfmt.toml");
    fs::write(rustfmt_config_path, config_str)
}
