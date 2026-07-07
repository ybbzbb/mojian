#[test]
fn mojian_home_overrides_data_dir_and_helpers() {
    let tmp = std::env::temp_dir().join(format!("mojian-paths-test-{}", std::process::id()));

    std::env::set_var("MOJIAN_HOME", &tmp);

    let data_dir = mojian_core::paths::data_dir().expect("data_dir resolves with MOJIAN_HOME set");
    assert_eq!(data_dir, tmp);
    assert_eq!(
        mojian_core::paths::central_db_path().unwrap(),
        tmp.join("central.db")
    );
    assert_eq!(
        mojian_core::paths::spec_master_dir().unwrap(),
        tmp.join("spec")
    );
    assert_eq!(mojian_core::paths::logs_dir().unwrap(), tmp.join("logs"));

    std::env::remove_var("MOJIAN_HOME");
}
