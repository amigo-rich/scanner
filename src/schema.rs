use crate::error::{Error, SchemaFileProblem};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub fn filename_u16(path: &Path) -> Result<u16, Error> {
    if let Some(os_name) = path.file_name() {
        if let Some(name) = os_name.to_str() {
            let components: Vec<&str> = name.split('-').collect();
            if components.is_empty() {
                return Err(Error::InvalidSchemaFile(SchemaFileProblem::NoComponents));
            } else if components.len() != 2 {
                return Err(Error::InvalidSchemaFile(
                    SchemaFileProblem::WrongNumberOfComponents,
                ));
            }
            let parsed_value: u16 = components[0].parse()?;
            Ok(parsed_value)
        } else {
            Err(Error::InvalidSchemaFile(SchemaFileProblem::InvalidUTF8))
        }
    } else {
        Err(Error::InvalidSchemaFile(SchemaFileProblem::InvalidPath))
    }
}

pub fn read_schemas(schema_path: &Path) -> Result<Option<Vec<String>>, Error> {
    if !schema_path.is_dir() {
        return Err(Error::InvalidSchemaDirectory(schema_path.to_path_buf()));
    }

    let mut schemas: BTreeMap<u16, String> = BTreeMap::new();
    for entry in fs::read_dir(&schema_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let value = filename_u16(&path)?;
            let content = fs::read_to_string(path)?;
            schemas.insert(value, content);
        }
    }
    let contents: Vec<String> = schemas.into_values().collect();
    if contents.is_empty() {
        return Ok(None);
    }
    Ok(Some(contents))
}
