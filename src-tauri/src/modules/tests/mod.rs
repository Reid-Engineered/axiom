use semver::Version;

use super::*;

mod conformance;
#[path = "fixtures/providers.rs"]
mod providers;
mod registry;

fn fixture(name: &str) -> &'static str {
    match name {
        "valid.toml" => include_str!("fixtures/valid.toml"),
        "missing-id.toml" => include_str!("fixtures/missing-id.toml"),
        "duplicate-capability.toml" => include_str!("fixtures/duplicate-capability.toml"),
        "unsupported-version.toml" => include_str!("fixtures/unsupported-version.toml"),
        "invalid-identifier.toml" => include_str!("fixtures/invalid-identifier.toml"),
        "malformed-version.toml" => include_str!("fixtures/malformed-version.toml"),
        "malformed-minimum-axiom-version.toml" => {
            include_str!("fixtures/malformed-minimum-axiom-version.toml")
        }
        "toml-syntax.toml" => include_str!("fixtures/toml-syntax.toml"),
        "wrong-field-type.toml" => include_str!("fixtures/wrong-field-type.toml"),
        "absent-id-field.toml" => include_str!("fixtures/absent-id-field.toml"),
        "incompatible-axiom-version.toml" => {
            include_str!("fixtures/incompatible-axiom-version.toml")
        }
        "duplicate-module-id-first.toml" => {
            include_str!("fixtures/duplicate-module-id-first.toml")
        }
        "duplicate-module-id-second.toml" => {
            include_str!("fixtures/duplicate-module-id-second.toml")
        }
        "duplicate-provider-first.toml" => {
            include_str!("fixtures/duplicate-provider-first.toml")
        }
        "duplicate-provider-second.toml" => {
            include_str!("fixtures/duplicate-provider-second.toml")
        }
        "workspace-secondary-provider.toml" => {
            include_str!("fixtures/workspace-secondary-provider.toml")
        }
        "parse-register-valid-echo.toml" => {
            include_str!("fixtures/parse-register-valid-echo.toml")
        }
        "parse-register-valid-secondary.toml" => {
            include_str!("fixtures/parse-register-valid-secondary.toml")
        }
        "parse-register-malformed.toml" => {
            include_str!("fixtures/parse-register-malformed.toml")
        }
        "conformance-multiple-capabilities.toml" => {
            include_str!("fixtures/conformance-multiple-capabilities.toml")
        }
        _ => panic!("unknown fixture: {name}"),
    }
}

#[test]
fn valid_manifest_parses_into_validated_types() {
    let manifest = parse(fixture("valid.toml")).unwrap();

    assert_eq!(
        manifest.id,
        ModuleId::new("org.axiom.test_fixture").unwrap()
    );
    assert_eq!(manifest.name, "Test Fixture");
    assert_eq!(manifest.version, Version::new(0, 1, 0));
    assert_eq!(manifest.minimum_axiom_version, Version::new(0, 1, 0));
    assert_eq!(manifest.offline, OfflineCapability::Full);
    assert_eq!(
        manifest.provides,
        vec![CapabilityDescriptor {
            id: CapabilityId::new("fixture.echo").unwrap(),
            version: 1,
        }]
    );
    assert_eq!(
        manifest.requires,
        vec![CapabilityRequirement {
            id: CapabilityId::new("knowledge.query").unwrap(),
            min_version: 1,
        }]
    );
    assert!(manifest.minimum_axiom_version <= axiom_version());
}

#[test]
fn embedded_source_discovers_the_test_fixture() {
    let discovered = EmbeddedManifestSource::default().discover().unwrap();

    assert_eq!(discovered.len(), 1);
    assert_eq!(
        discovered[0].0,
        ModuleId::new("org.axiom.test_fixture").unwrap()
    );
    assert_eq!(parse(&discovered[0].1).unwrap().id, discovered[0].0);
}

#[test]
fn empty_module_id_is_missing() {
    assert!(matches!(
        parse(fixture("missing-id.toml")),
        Err(ManifestError::MissingModuleId)
    ));
}

#[test]
fn duplicate_provided_capability_is_rejected() {
    assert!(matches!(
        parse(fixture("duplicate-capability.toml")),
        Err(ManifestError::DuplicateCapability {
            module_id,
            capability_id,
        }) if module_id == "org.axiom.duplicate" && capability_id == "fixture.echo"
    ));
}

#[test]
fn unsupported_manifest_version_is_rejected() {
    assert!(matches!(
        parse(fixture("unsupported-version.toml")),
        Err(ManifestError::UnsupportedManifestVersion {
            found: 2,
            supported: &[1],
        })
    ));
}

#[test]
fn invalid_identifier_is_rejected() {
    assert!(matches!(
        parse(fixture("invalid-identifier.toml")),
        Err(ManifestError::InvalidIdentifier { value }) if value == "Fixture.Echo"
    ));
}

#[test]
fn malformed_module_and_minimum_axiom_versions_are_distinguished() {
    assert!(matches!(
        parse(fixture("malformed-version.toml")),
        Err(ManifestError::MalformedVersion {
            field: "version",
            value,
        }) if value == "not-semver"
    ));
    assert!(matches!(
        parse(fixture("malformed-minimum-axiom-version.toml")),
        Err(ManifestError::MalformedVersion {
            field: "minimum_axiom_version",
            value,
        }) if value == "still-not-semver"
    ));
}

#[test]
fn newer_minimum_axiom_version_is_incompatible() {
    assert!(matches!(
        parse(fixture("incompatible-axiom-version.toml")),
        Err(ManifestError::IncompatibleAxiomVersion { required, running })
            if required == Version::new(99, 0, 0) && running == axiom_version()
    ));
}

#[test]
fn malformed_toml_and_wrong_field_types_are_syntax_errors() {
    assert!(matches!(
        parse(fixture("toml-syntax.toml")),
        Err(ManifestError::TomlSyntax(_))
    ));
    assert!(matches!(
        parse(fixture("wrong-field-type.toml")),
        Err(ManifestError::TomlSyntax(_))
    ));
    assert!(matches!(
        parse(fixture("absent-id-field.toml")),
        Err(ManifestError::TomlSyntax(_))
    ));
}

#[test]
fn identifiers_enforce_the_locked_grammar() {
    for valid in ["practice.generate", "org.axiom.practice", "a.b", "a0.b_1"] {
        assert!(ModuleId::new(valid).is_ok());
        assert!(CapabilityId::new(valid).is_ok());
    }

    for invalid in [
        "Practice",
        "practice",
        "practice..generate",
        ".practice",
        "practice.",
        "practice.Generate",
        "practice.generate-more",
    ] {
        assert!(matches!(
            ModuleId::new(invalid),
            Err(ManifestError::InvalidIdentifier { .. })
        ));
        assert!(matches!(
            CapabilityId::new(invalid),
            Err(ManifestError::InvalidIdentifier { .. })
        ));
    }
}

#[test]
fn manifest_types_round_trip_through_json() {
    let manifest = parse(fixture("valid.toml")).unwrap();
    let manifest_json = serde_json::to_string(&manifest).unwrap();
    assert_eq!(
        serde_json::from_str::<ModuleManifest>(&manifest_json).unwrap(),
        manifest
    );

    let descriptor = manifest.provides[0].clone();
    let descriptor_json = serde_json::to_string(&descriptor).unwrap();
    assert_eq!(
        serde_json::from_str::<CapabilityDescriptor>(&descriptor_json).unwrap(),
        descriptor
    );

    let requirement = manifest.requires[0].clone();
    let requirement_json = serde_json::to_string(&requirement).unwrap();
    assert_eq!(
        serde_json::from_str::<CapabilityRequirement>(&requirement_json).unwrap(),
        requirement
    );
}
