use std::fs;
use std::path::Path;

#[test]
fn presentation_composition_stays_out_of_orchestration() {
    let main = include_str!("../src/main.rs").replace("\r\n", "\n");
    let production = main.split("#[cfg(test)]\nmod tests").next().unwrap();
    assert!(
        !production.contains("fn render_"),
        "product rendering belongs in views, not main.rs"
    );
    assert!(
        !production.contains(".style("),
        "widget styling belongs in ui/views, not orchestration"
    );
    assert!(!production.contains("use ui::*;"));
}

#[test]
fn presentation_has_no_legacy_facade_or_staged_dead_code() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let facade = fs::read_to_string(src.join("ui/mod.rs")).unwrap();
    assert!(!facade.contains("pub use styles::*"));
    assert!(!facade.contains("pub use tokens::*"));
    check_sources(&src);
}

fn check_sources(path: &Path) {
    for entry in fs::read_dir(path).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            check_sources(&path);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            let source = fs::read_to_string(&path).unwrap();
            for obsolete in [
                "allow(dead_code)",
                "primary_button_style",
                "subtle_button_style",
                "action_button_style",
                "render_dialog_frame",
                "render_modal_overlay",
                "render_help_badge",
            ] {
                assert!(
                    !source.contains(obsolete),
                    "{} retains {obsolete}",
                    path.display()
                );
            }
        }
    }
}
