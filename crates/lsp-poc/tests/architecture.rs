//! Architecture checks via `arch-lint`, wired programmatically.

use std::path::Path;

use arch_lint::{
    Analyzer, RuleBox, Severity,
    declarative::load_rules_from_toml,
    rules::{
        NoErrorSwallowing, NoSilentResultDrop, NoSyncIo, RequireThiserror, RequireTracing,
        TracingEnvInit,
    },
};

#[test]
fn architecture_rules_hold() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let rules: Vec<RuleBox> = vec![
        Box::new(NoSyncIo::new()),
        Box::new(NoErrorSwallowing::new()),
        Box::new(NoSilentResultDrop::new()),
        Box::new(RequireThiserror::new()),
        Box::new(RequireTracing::new()),
        Box::new(TracingEnvInit::new()),
    ];

    let mut builder = Analyzer::builder().root(root).exclude("**/target/**");
    for rule in rules {
        builder = builder.rule_box(rule);
    }

    // arch-lint: allow(no-sync-io) reason="the analyzer setup reads its own config synchronously; this is test code"
    let config = std::fs::read_to_string(root.join("arch-lint.toml"))
        .expect("arch-lint.toml is committed at the crate root");
    for rule in load_rules_from_toml(&config).expect("arch-lint.toml parses") {
        builder = builder.rule_box(rule);
    }

    let analyzer = builder.build().expect("analyzer builds");
    let result = analyzer.analyze().expect("analysis completes");

    assert!(
        !result.has_violations_at(Severity::Error),
        "{}",
        result.format_test_report(Severity::Error)
    );
}
