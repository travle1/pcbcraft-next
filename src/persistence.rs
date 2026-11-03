//! Experimental bounded draft format; not KiCad or the final full PCB project format.
use crate::document::{Document, Error, Result};
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::Path,
};
pub const FORMAT: &str = "pcbcraft-schematic-draft/5";
pub const MAX_BYTES: usize = 4 * 1024 * 1024;
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    format: String,
    document: Document,
}
pub fn encode(doc: &Document) -> Result<Vec<u8>> {
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
    let value: Envelope = serde_json::from_value(raw).map_err(|e| format!("Invalid draft: {e}"))?;
    if ![
        FORMAT,
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
    value.document.validate()?;
    Ok(value.document)
}
pub fn read(path: &Path) -> Result<(Document, Vec<u8>)> {
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
    Ok((decode(&bytes)?, bytes))
}
pub fn write(path: &Path, doc: &Document, expected: Option<&[u8]>) -> Result<Vec<u8>> {
    let bytes = encode(doc)?;
    if path
        .symlink_metadata()
        .is_ok_and(|m| m.file_type().is_symlink())
    {
        return Err("Save to a regular file, not a symbolic link.".into());
    }
    if let Some(expected) = expected {
        let (_, current) = read(path)?;
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
