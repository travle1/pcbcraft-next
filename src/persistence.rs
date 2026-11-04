//! Experimental bounded draft format; not KiCad or the final full PCB project format.
use crate::document::{Document, Error, Result};
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::Path,
};
pub const PROJECT_FORMAT: &str = "pcbcraft-project-draft/7";
pub const FORMAT: &str = "pcbcraft-schematic-draft/6";
pub const MAX_BYTES: usize = 4 * 1024 * 1024;
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    format: String,
    document: Document,
}
fn has_text_boxes(doc: &serde_json::Value) -> bool {
    doc["text_overrides"].as_array().is_some_and(|items| {
        items
            .iter()
            .any(|item| item["appearance"].get("box_size").is_some())
    })
}
pub fn encode(doc: &Document) -> Result<Vec<u8>> {
    if !doc.ports.is_empty() || !doc.sheet_symbols.is_empty() {
        return Err("Hierarchy requires a complete project save.".into());
    }
    doc.validate()?;
    let bytes = serde_json::to_vec_pretty(&Envelope {
        format: FORMAT.into(),
        document: doc.clone(),
    })
    .map_err(|e| Error::Io(e.to_string()))?;
    if bytes.len() > MAX_BYTES {
        return Err("Draft exceeds the 4 MiB limit.".into());
    }
    Ok(bytes)
}
pub fn decode(bytes: &[u8]) -> Result<Document> {
    if bytes.len() > MAX_BYTES {
        return Err("Draft exceeds the 4 MiB limit.".into());
    }
    let mut raw: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| format!("Invalid draft: {e}"))?;
    if [Some(PROJECT_FORMAT), Some("pcbcraft-project-draft/6")]
        .contains(&raw.get("format").and_then(serde_json::Value::as_str))
    {
        let project = decode_project(bytes)?;
        if project.occurrences.len() != 1
            || !project.main.ports.is_empty()
            || !project.main.sheet_symbols.is_empty()
        {
            return Err("Open this file as a complete project, not a single sheet.".into());
        }
        return Ok(project.main);
    }
    let format = raw
        .get("format")
        .and_then(serde_json::Value::as_str)
        .ok_or("Missing draft format.")?
        .to_owned();
    if [
        "pcbcraft-schematic-draft/1",
        "pcbcraft-schematic-draft/2",
        "pcbcraft-schematic-draft/3",
        "pcbcraft-schematic-draft/4",
    ]
    .contains(&format.as_str())
    {
        if raw["document"].get("text_overrides").is_some() {
            return Err("Text formatting requires draft version 5.".into());
        }
        if let Some(notes) = raw
            .get_mut("document")
            .and_then(|d| d.get_mut("annotations"))
            .and_then(serde_json::Value::as_array_mut)
        {
            if notes.len() > 256 {
                return Err("Too many sheet notes.".into());
            }
            for note in notes {
                let object = note.as_object_mut().ok_or("Invalid sheet note.")?;
                if object.contains_key("id") {
                    return Err("Text identities require draft version 5.".into());
                }
                object.insert(
                    "id".into(),
                    serde_json::to_value(crate::document::AnnotationId::fresh()?)
                        .map_err(|e| e.to_string())?,
                );
            }
        }
    }
    if format != FORMAT
        && (raw["document"].get("page").is_some() || has_text_boxes(&raw["document"]))
    {
        return Err("Pages and text boxes require schematic draft version 6.".into());
    }
    let value: Envelope = serde_json::from_value(raw).map_err(|e| format!("Invalid draft: {e}"))?;
    if ![
        FORMAT,
        "pcbcraft-schematic-draft/5",
        "pcbcraft-schematic-draft/1",
        "pcbcraft-schematic-draft/2",
        "pcbcraft-schematic-draft/3",
        "pcbcraft-schematic-draft/4",
    ]
    .contains(&value.format.as_str())
    {
        return Err(Error::UnsupportedFormat(value.format));
    }
    if value.format == "pcbcraft-schematic-draft/1"
        && (value.document.builtin_definitions_version != 1
            || !value.document.annotations.is_empty()
            || value.document.wires.iter().any(|w| w.label.is_some()))
    {
        return Err("Version 1 does not support version 2 data.".into());
    }
    if [
        "pcbcraft-schematic-draft/1",
        "pcbcraft-schematic-draft/2",
        "pcbcraft-schematic-draft/3",
    ]
    .contains(&value.format.as_str())
        && (!value.document.junctions.is_empty()
            || value
                .document
                .wires
                .iter()
                .any(|w| w.bends.is_some() || w.from.pin().is_none() || w.to.pin().is_none()))
    {
        return Err("Authored wires and junctions require draft version 4.".into());
    }
    if ["pcbcraft-schematic-draft/1", "pcbcraft-schematic-draft/2"].contains(&value.format.as_str())
        && value.document.wires.iter().any(|w| w.draw_path)
    {
        return Err("Named paths require draft version 3.".into());
    }
    if !value.document.ports.is_empty() || !value.document.sheet_symbols.is_empty() {
        return Err("Hierarchy requires project draft version 6.".into());
    }
    value.document.validate()?;
    Ok(value.document)
}
pub fn read(path: &Path) -> Result<(Document, Vec<u8>)> {
    let bytes = read_bytes(path)?;
    Ok((decode(&bytes)?, bytes))
}
fn read_bytes(path: &Path) -> Result<Vec<u8>> {
    let file = std::fs::File::open(path).map_err(|e| Error::Io(e.to_string()))?;
    if !file
        .metadata()
        .map_err(|e| Error::Io(e.to_string()))?
        .is_file()
    {
        return Err("Open a regular project file.".into());
    }
    let mut bytes = Vec::new();
    file.take((MAX_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|e| Error::Io(e.to_string()))?;
    Ok(bytes)
}
pub fn write(path: &Path, doc: &Document, expected: Option<&[u8]>) -> Result<Vec<u8>> {
    save_bytes(path, encode(doc)?, expected)
}
fn save_bytes(path: &Path, bytes: Vec<u8>, expected: Option<&[u8]>) -> Result<Vec<u8>> {
    if path
        .symlink_metadata()
        .is_ok_and(|m| m.file_type().is_symlink())
    {
        return Err("Save to a regular file, not a symbolic link.".into());
    }
    if let Some(expected) = expected {
        let current = read_bytes(path)?;
        if current != expected {
            return Err(Error::FileChanged);
        }
    }
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut file = tempfile::NamedTempFile::new_in(parent).map_err(|e| Error::Io(e.to_string()))?;
    file.write_all(&bytes)
        .map_err(|e| Error::Io(e.to_string()))?;
    file.as_file()
        .sync_all()
        .map_err(|e| Error::Io(e.to_string()))?;
    file.persist(path).map_err(|e| Error::Io(e.to_string()))?;
    Ok(bytes)
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectEnvelope {
    format: String,
    project: crate::project::Project,
}
pub fn encode_project(project: &crate::project::Project) -> Result<Vec<u8>> {
    project.validate()?;
    let bytes = serde_json::to_vec_pretty(&ProjectEnvelope {
        format: PROJECT_FORMAT.into(),
        project: project.clone(),
    })
    .map_err(|e| Error::Io(e.to_string()))?;
    if bytes.len() > MAX_BYTES {
        return Err("Project exceeds the 4 MiB limit.".into());
    }
    Ok(bytes)
}
pub fn decode_project(bytes: &[u8]) -> Result<crate::project::Project> {
    if bytes.len() > MAX_BYTES {
        return Err("Project exceeds the 4 MiB limit.".into());
    }
    let raw: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| format!("Invalid project: {e}"))?;
    if ![Some(PROJECT_FORMAT), Some("pcbcraft-project-draft/6")]
        .contains(&raw.get("format").and_then(serde_json::Value::as_str))
    {
        return crate::project::Project::from_document(decode(bytes)?);
    }
    if raw["format"] == "pcbcraft-project-draft/6" {
        let project = &raw["project"];
        if project["main"].get("page").is_some()
            || has_text_boxes(&project["main"])
            || project["sheets"].as_array().is_some_and(|sheets| {
                sheets
                    .iter()
                    .any(|s| s.get("page").is_some() || has_text_boxes(s))
            })
        {
            return Err("Pages and text boxes require project version 7.".into());
        }
    }
    let envelope: ProjectEnvelope =
        serde_json::from_value(raw).map_err(|e| format!("Invalid project: {e}"))?;
    envelope.project.validate()?;
    Ok(envelope.project)
}
pub fn read_project(path: &Path) -> Result<(crate::project::Project, Vec<u8>)> {
    let bytes = read_bytes(path)?;
    Ok((decode_project(&bytes)?, bytes))
}
pub fn write_project(
    path: &Path,
    project: &crate::project::Project,
    expected: Option<&[u8]>,
) -> Result<Vec<u8>> {
    save_bytes(path, encode_project(project)?, expected)
}
