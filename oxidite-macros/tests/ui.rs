#[test]
fn model_derive_ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/pass_table_name.rs");
    t.pass("tests/ui/pass_table_alias.rs");
    t.compile_fail("tests/ui/fail_non_struct.rs");
    t.compile_fail("tests/ui/fail_unnamed_struct.rs");
    t.compile_fail("tests/ui/fail_missing_id.rs");
    t.compile_fail("tests/ui/fail_email_non_string.rs");
    t.compile_fail("tests/ui/fail_bad_model_attr.rs");
    t.compile_fail("tests/ui/fail_id_not_i64.rs");
    t.compile_fail("tests/ui/fail_deleted_at_not_option.rs");
    t.compile_fail("tests/ui/fail_table_and_table_name.rs");
}
