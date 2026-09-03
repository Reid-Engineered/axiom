use std::fs;
use std::path::{Path, PathBuf};

use super::concept::parse_concept_file;
use super::error::KnowledgeError;
use super::example::parse_example_file;
use super::objective::parse_objective_file;
use super::problem_family::parse_problem_family_file;
use super::types::{Concept, Example, Objective, ProblemFamily};

pub(crate) struct DiscoveredEntities {
    pub concepts: Vec<Concept>,
    pub objectives: Vec<Objective>,
    pub examples: Vec<Example>,
    pub problem_families: Vec<ProblemFamily>,
}

pub(crate) fn read_package_toml(root: &Path) -> Result<String, KnowledgeError> {
    fs::read_to_string(root.join("package.toml")).map_err(|_| KnowledgeError::MissingPackageToml {
        path: root.to_owned(),
    })
}

pub(crate) fn read_sources_toml(root: &Path) -> Result<String, KnowledgeError> {
    fs::read_to_string(root.join("sources.toml")).map_err(|_| KnowledgeError::MissingSourcesToml {
        path: root.to_owned(),
    })
}

pub(crate) fn discover_entities(root: &Path) -> Result<DiscoveredEntities, KnowledgeError> {
    let concepts = load_entities(root, "concepts", parse_concept_file, |c: &Concept| {
        c.id.as_str().to_owned()
    })?;
    let objectives = load_entities(root, "objectives", parse_objective_file, |o: &Objective| {
        o.id.as_str().to_owned()
    })?;
    let examples = load_entities(root, "examples", parse_example_file, |e: &Example| {
        e.id.as_str().to_owned()
    })?;
    let problem_families = load_entities(
        root,
        "problems",
        parse_problem_family_file,
        |p: &ProblemFamily| p.id.as_str().to_owned(),
    )?;

    Ok(DiscoveredEntities {
        concepts,
        objectives,
        examples,
        problem_families,
    })
}

fn list_entity_files(root: &Path, subdir: &str) -> Result<Vec<PathBuf>, KnowledgeError> {
    let dir = root.join(subdir);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut entries = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|source| KnowledgeError::Io {
        path: dir.clone(),
        source,
    })? {
        let entry = entry.map_err(|source| KnowledgeError::Io {
            path: dir.clone(),
            source,
        })?;
        let path = entry.path();
        if path.is_dir() {
            return Err(KnowledgeError::NestedEntityDirectory { path });
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            return Err(KnowledgeError::UnexpectedEntityFile { path });
        }
        entries.push(path);
    }
    entries.sort();
    Ok(entries)
}

// Per-kind ID uniqueness (spec §2) needs no separate duplicate-tracking pass here:
// a directory's filenames are unique by filesystem construction, and the
// filename/id check below already forces every file's own id to equal its own
// filename. Two different files can therefore never both validly declare the same
// id — whichever one doesn't match its own filename is rejected first. Contrast
// `sources.toml` (Task 4), where multiple `[[sources]]` blocks share one file and
// duplicate `SourceId`s are a real, independently-necessary check.
fn load_entities<T>(
    root: &Path,
    subdir: &str,
    parse: impl Fn(&Path, &str) -> Result<T, KnowledgeError>,
    id_of: impl Fn(&T) -> String,
) -> Result<Vec<T>, KnowledgeError> {
    let paths = list_entity_files(root, subdir)?;
    let mut result = Vec::with_capacity(paths.len());
    for path in paths {
        let raw = fs::read_to_string(&path).map_err(|source| KnowledgeError::Io {
            path: path.clone(),
            source,
        })?;
        let entity = parse(&path, &raw)?;
        let id = id_of(&entity);
        let filename_id = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or_default()
            .to_owned();
        if filename_id != id {
            return Err(KnowledgeError::FilenameIdMismatch {
                path,
                filename_id,
                frontmatter_id: id,
            });
        }
        result.push(entity);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_package_dir(test_name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("axiom_knowledge_discover_test_{test_name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    const VALID_CONCEPT: &str = "+++\nid = \"shell.a\"\nname = \"A\"\n\n[[provenance_refs]]\nsource_id = \"src.x\"\nkind = \"direct\"\n+++\n\nBody.\n";

    #[test]
    fn missing_concepts_directory_yields_zero_concepts() {
        let root = temp_package_dir("missing_concepts_directory_yields_zero_concepts");
        let discovered = discover_entities(&root).unwrap();
        assert!(discovered.concepts.is_empty());
    }

    #[test]
    fn discovers_and_sorts_concept_files() {
        let root = temp_package_dir("discovers_and_sorts_concept_files");
        let concepts_dir = root.join("concepts");
        fs::create_dir_all(&concepts_dir).unwrap();
        fs::write(concepts_dir.join("shell.a.md"), VALID_CONCEPT).unwrap();
        let discovered = discover_entities(&root).unwrap();
        assert_eq!(discovered.concepts.len(), 1);
        assert_eq!(discovered.concepts[0].id.as_str(), "shell.a");
    }

    #[test]
    fn filename_id_mismatch_is_rejected() {
        let root = temp_package_dir("filename_id_mismatch_is_rejected");
        let concepts_dir = root.join("concepts");
        fs::create_dir_all(&concepts_dir).unwrap();
        fs::write(concepts_dir.join("shell.wrong_name.md"), VALID_CONCEPT).unwrap();
        assert!(matches!(
            discover_entities(&root),
            Err(KnowledgeError::FilenameIdMismatch { .. })
        ));
    }

    #[test]
    fn two_files_declaring_the_same_id_are_rejected_via_filename_mismatch() {
        // Per-kind ID uniqueness for file-per-entity directories doesn't need a
        // separate duplicate-tracking pass: filenames are unique within a directory
        // by filesystem construction, and every file's own id MUST equal its own
        // filename (§3). So two different files can never both validly declare the
        // same id — whichever one doesn't match its own filename is rejected first.
        // This is different from sources.toml (Task 4), where multiple [[sources]]
        // blocks share one file and duplicate SourceIds are a real, separately
        // checked case.
        let root =
            temp_package_dir("two_files_declaring_the_same_id_are_rejected_via_filename_mismatch");
        let concepts_dir = root.join("concepts");
        fs::create_dir_all(&concepts_dir).unwrap();
        fs::write(concepts_dir.join("shell.a.md"), VALID_CONCEPT).unwrap();
        // Deliberately the same content, same declared id — the point of this test is
        // that shell.b.md's filename never matches "shell.a" regardless of what id it
        // declares, so this doesn't need a distinct id to make the case.
        fs::write(concepts_dir.join("shell.b.md"), VALID_CONCEPT).unwrap();
        assert!(matches!(
            discover_entities(&root),
            Err(KnowledgeError::FilenameIdMismatch { .. })
        ));
    }

    #[test]
    fn non_md_file_in_entity_directory_is_rejected() {
        let root = temp_package_dir("non_md_file_in_entity_directory_is_rejected");
        let concepts_dir = root.join("concepts");
        fs::create_dir_all(&concepts_dir).unwrap();
        fs::write(concepts_dir.join("notes.txt"), "stray file").unwrap();
        assert!(matches!(
            discover_entities(&root),
            Err(KnowledgeError::UnexpectedEntityFile { .. })
        ));
    }

    #[test]
    fn nested_directory_in_entity_directory_is_rejected() {
        let root = temp_package_dir("nested_directory_in_entity_directory_is_rejected");
        let concepts_dir = root.join("concepts");
        fs::create_dir_all(concepts_dir.join("nested")).unwrap();
        assert!(matches!(
            discover_entities(&root),
            Err(KnowledgeError::NestedEntityDirectory { .. })
        ));
    }

    #[test]
    fn missing_package_toml_is_reported() {
        let root = temp_package_dir("missing_package_toml_is_reported");
        assert!(matches!(
            read_package_toml(&root),
            Err(KnowledgeError::MissingPackageToml { .. })
        ));
    }

    #[test]
    fn missing_sources_toml_is_reported() {
        let root = temp_package_dir("missing_sources_toml_is_reported");
        assert!(matches!(
            read_sources_toml(&root),
            Err(KnowledgeError::MissingSourcesToml { .. })
        ));
    }
}
