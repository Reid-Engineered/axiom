#[cfg(test)]
mod canonical;

#[cfg(test)]
mod conformance;

#[cfg(test)]
mod conformance_problem_family;

#[cfg(test)]
mod migration;

#[cfg(test)]
pub(crate) mod support {
    use std::fs;
    use std::path::{Path, PathBuf};

    pub(crate) fn temp_root(case: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("axiom_knowledge_conformance_{case}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// A minimal but complete valid package: 1 Source, 2 Concepts (one prerequisite
    /// of the other), 1 Objective, 1 Example — enough surface to mutate into every
    /// spec §18 case without each case needing its own bespoke shape.
    pub(crate) fn write_base_package(root: &Path) {
        fs::write(
            root.join("package.toml"),
            "id = \"org.axiom.conformance\"\nschema_version = 1\nversion = \"1.0.0\"\ntitle = \"Conformance\"\ndescription = \"Conformance base package.\"\n",
        )
        .unwrap();
        fs::write(
            root.join("sources.toml"),
            "[[sources]]\nid = \"src.a\"\ntitle = \"Source A\"\n",
        )
        .unwrap();
        fs::create_dir_all(root.join("concepts")).unwrap();
        fs::write(
            root.join("concepts/shell.a.md"),
            "+++\nid = \"shell.a\"\nname = \"A\"\nprerequisite_ids = []\nrelated_ids = []\n\n[[provenance_refs]]\nsource_id = \"src.a\"\nkind = \"direct\"\n+++\n\nConcept A body.\n",
        )
        .unwrap();
        fs::write(
            root.join("concepts/shell.b.md"),
            "+++\nid = \"shell.b\"\nname = \"B\"\nprerequisite_ids = [\"shell.a\"]\nrelated_ids = []\n\n[[provenance_refs]]\nsource_id = \"src.a\"\nkind = \"direct\"\n+++\n\nConcept B body.\n",
        )
        .unwrap();
        fs::create_dir_all(root.join("objectives")).unwrap();
        fs::write(
            root.join("objectives/shell.obj.md"),
            "+++\nid = \"shell.obj\"\nconcept_id = \"shell.a\"\n\n[[provenance_refs]]\nsource_id = \"src.a\"\nkind = \"direct\"\n+++\n\nObjective body.\n",
        )
        .unwrap();
        fs::create_dir_all(root.join("examples")).unwrap();
        fs::write(
            root.join("examples/shell.ex.md"),
            "+++\nid = \"shell.ex\"\nconcept_id = \"shell.a\"\nobjective_ids = [\"shell.obj\"]\n\n[[provenance_refs]]\nsource_id = \"src.a\"\nkind = \"direct\"\n+++\n\n## Problem\n\nP.\n\n## Solution\n\nS.\n",
        )
        .unwrap();
    }
}
