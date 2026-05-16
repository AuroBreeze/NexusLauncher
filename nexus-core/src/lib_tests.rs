use super::*;

// ============================================================
// parse_major_version
// ============================================================

#[test]
fn test_java_8_legacy() {
    assert_eq!(parse_major_version("1.8.0_382"), Some(8));
}

#[test]
fn test_java_7_legacy() {
    assert_eq!(parse_major_version("1.7.0_80"), Some(7));
}

#[test]
fn test_java_17() {
    assert_eq!(parse_major_version("17.0.8"), Some(17));
}

#[test]
fn test_java_21() {
    assert_eq!(parse_major_version("21.0.5"), Some(21));
}

#[test]
fn test_java_26() {
    assert_eq!(parse_major_version("26.0.1"), Some(26));
}

#[test]
fn test_parse_major_version_empty() {
    assert_eq!(parse_major_version(""), None);
}

#[test]
fn test_parse_major_version_non_numeric() {
    assert_eq!(parse_major_version("abc"), None);
}

#[test]
fn test_parse_major_version_legacy_non_numeric() {
    assert_eq!(parse_major_version("1.x.0"), None);
}

// ============================================================
// maven_to_path
// ============================================================

#[test]
fn test_maven_standard() {
    assert_eq!(
        maven_to_path("net.minecraft:client:1.20"),
        "net/minecraft/client/1.20/client-1.20.jar"
    );
}

#[test]
fn test_maven_multi_part_group() {
    assert_eq!(
        maven_to_path("com.example.project:my-mod:2.0"),
        "com/example/project/my-mod/2.0/my-mod-2.0.jar"
    );
}

#[test]
fn test_maven_too_few_parts() {
    assert_eq!(maven_to_path("short"), "short");
    assert_eq!(maven_to_path("a:b"), "a:b");
}

// ============================================================
// Loaders
// ============================================================

#[test]
fn test_loaders_from_str_fabric() {
    assert!(matches!("fabric".parse::<Loaders>(), Ok(Loaders::Fabric)));
}

#[test]
fn test_loaders_from_str_quilt() {
    assert!(matches!("quilt".parse::<Loaders>(), Ok(Loaders::Quilt)));
}

#[test]
fn test_loaders_from_str_case_insensitive() {
    assert!(matches!("FABRIC".parse::<Loaders>(), Ok(Loaders::Fabric)));
    assert!(matches!("QuIlT".parse::<Loaders>(), Ok(Loaders::Quilt)));
}

// ============================================================
// validate_instance_name
// ============================================================

#[test]
fn test_valid_instance_names() {
    assert!(validate_instance_name("1.20").is_ok());
    assert!(validate_instance_name("my-instance").is_ok());
    assert!(validate_instance_name("1.20-fabric").is_ok());
}

#[test]
fn test_instance_name_empty() {
    assert!(validate_instance_name("").is_err());
}

#[test]
fn test_instance_name_with_path_separator() {
    assert!(validate_instance_name("a/b").is_err());
    assert!(validate_instance_name("a\\b").is_err());
}

#[test]
fn test_instance_name_with_dot_dot() {
    assert!(validate_instance_name("..").is_err());
    assert!(validate_instance_name("../escape").is_err());
    assert!(validate_instance_name("sub/../escape").is_err());
}

#[test]
fn test_instance_name_absolute() {
    assert!(validate_instance_name("/etc/passwd").is_err());
}

#[test]
fn test_loaders_from_str_invalid() {
    assert!("invalid".parse::<Loaders>().is_err());
}

#[test]
fn test_loaders_display() {
    assert_eq!(Loaders::Fabric.to_string(), "fabric");
    assert_eq!(Loaders::Quilt.to_string(), "quilt");
}
