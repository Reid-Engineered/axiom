use std::collections::HashSet;

use semver::Version;

use super::error::KnowledgeError;
use super::identifier::{KnowledgePackageId, SourceId};
use super::raw::{RawKnowledgePackage, RawSource, RawSourcesFile};
use super::types::Source;

const SUPPORTED_SCHEMA_VERSION: u32 = 1;

pub(crate) struct PackageIdentity {
    pub id: KnowledgePackageId,
    pub schema_version: u32,
    pub version: Version,
    pub title: String,
    pub description: String,
}

pub(crate) fn parse_package_toml(raw_toml: &str) -> Result<PackageIdentity, KnowledgeError> {
    let raw: RawKnowledgePackage =
        toml::from_str(raw_toml).map_err(|source| KnowledgeError::TomlSyntax {
            path: "package.toml".into(),
            source,
        })?;

    if raw.schema_version != SUPPORTED_SCHEMA_VERSION {
        return Err(KnowledgeError::UnsupportedSchemaVersion {
            found: raw.schema_version,
        });
    }

    let id = KnowledgePackageId::new(raw.id)?;
    let version = Version::parse(&raw.version).map_err(|_| KnowledgeError::MalformedVersion {
        field: "version",
        value: raw.version,
    })?;

    if raw.title.trim().is_empty() {
        return Err(KnowledgeError::EmptyField {
            path: "package.toml".into(),
            field: "title",
        });
    }
    if raw.description.trim().is_empty() {
        return Err(KnowledgeError::EmptyField {
            path: "package.toml".into(),
            field: "description",
        });
    }

    Ok(PackageIdentity {
        id,
        schema_version: raw.schema_version,
        version,
        title: raw.title,
        description: raw.description,
    })
}

pub(crate) fn parse_sources_toml(raw_toml: &str) -> Result<Vec<Source>, KnowledgeError> {
    let raw: RawSourcesFile =
        toml::from_str(raw_toml).map_err(|source| KnowledgeError::TomlSyntax {
            path: "sources.toml".into(),
            source,
        })?;

    let mut seen_ids = HashSet::new();
    let mut sources = Vec::with_capacity(raw.sources.len());
    for RawSource {
        id,
        title,
        authors,
        edition,
        license,
    } in raw.sources
    {
        let id = SourceId::new(id)?;
        if !seen_ids.insert(id.clone()) {
            return Err(KnowledgeError::DuplicateSourceId {
                id: id.as_str().to_owned(),
            });
        }
        sources.push(Source {
            id,
            title,
            authors,
            edition,
            license,
        });
    }
    Ok(sources)
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_PACKAGE: &str = r#"
        id = "org.axiom.calculus_shells"
        schema_version = 1
        version = "1.0.0"
        title = "Cylindrical Shells"
        description = "A tiny reference package."
    "#;

    #[test]
    fn valid_package_toml_parses_into_identity() {
        let identity = parse_package_toml(VALID_PACKAGE).unwrap();
        assert_eq!(identity.id.as_str(), "org.axiom.calculus_shells");
        assert_eq!(identity.schema_version, 1);
        assert_eq!(identity.version, semver::Version::new(1, 0, 0));
        assert_eq!(identity.title, "Cylindrical Shells");
    }

    #[test]
    fn unsupported_schema_version_is_rejected() {
        let toml = VALID_PACKAGE.replace("schema_version = 1", "schema_version = 2");
        assert!(matches!(
            parse_package_toml(&toml),
            Err(KnowledgeError::UnsupportedSchemaVersion { found: 2 })
        ));
    }

    #[test]
    fn missing_schema_version_is_a_structural_failure() {
        let toml = r#"
            id = "org.axiom.calculus_shells"
            version = "1.0.0"
            title = "Cylindrical Shells"
            description = "A tiny reference package."
        "#;
        assert!(matches!(
            parse_package_toml(toml),
            Err(KnowledgeError::TomlSyntax { .. })
        ));
    }

    #[test]
    fn non_integer_schema_version_is_a_structural_failure() {
        let toml = VALID_PACKAGE.replace("schema_version = 1", r#"schema_version = "1""#);
        assert!(matches!(
            parse_package_toml(&toml),
            Err(KnowledgeError::TomlSyntax { .. })
        ));
    }

    #[test]
    fn malformed_package_version_is_rejected() {
        let toml = VALID_PACKAGE.replace(r#"version = "1.0.0""#, r#"version = "not-semver""#);
        assert!(matches!(
            parse_package_toml(&toml),
            Err(KnowledgeError::MalformedVersion {
                field: "version",
                ..
            })
        ));
    }

    #[test]
    fn empty_title_is_rejected() {
        let toml = VALID_PACKAGE.replace(r#"title = "Cylindrical Shells""#, r#"title = """#);
        assert!(matches!(
            parse_package_toml(&toml),
            Err(KnowledgeError::EmptyField { field: "title", .. })
        ));
    }

    #[test]
    fn duplicate_source_ids_are_rejected() {
        let toml = r#"
            [[sources]]
            id = "src.openstax_calc2"
            title = "Calculus Volume 2"

            [[sources]]
            id = "src.openstax_calc2"
            title = "Calculus Volume 2 (duplicate)"
        "#;
        assert!(matches!(
            parse_sources_toml(toml),
            Err(KnowledgeError::DuplicateSourceId { .. })
        ));
    }

    #[test]
    fn sources_toml_may_declare_zero_sources() {
        assert_eq!(parse_sources_toml("").unwrap().len(), 0);
    }
}
