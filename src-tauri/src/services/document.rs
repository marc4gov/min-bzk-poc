use std::path::Path;
use std::fs;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DocumentError {
    #[error("Unsupported file type: {0}")]
    UnsupportedFileType(String),
    #[error("Failed to read file: {0}")]
    ReadError(#[from] std::io::Error),
    #[error("Failed to extract text: {0}")]
    ExtractError(String),
}

#[derive(Debug, Clone)]
pub struct ExtractedDocument {
    pub filename: String,
    pub content: String,
    pub file_type: String,
}

pub fn extract_text_from_file(file_path: &str) -> Result<ExtractedDocument, DocumentError> {
    let path = Path::new(file_path);
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    let content = match extension.as_str() {
        "txt" => extract_from_txt(file_path)?,
        "pdf" => extract_from_pdf(file_path)?,
        "docx" => extract_from_docx(file_path)?,
        "md" => extract_from_txt(file_path)?,
        _ => {
            return Err(DocumentError::UnsupportedFileType(extension));
        }
    };

    Ok(ExtractedDocument {
        filename,
        content,
        file_type: extension,
    })
}

fn extract_from_txt(file_path: &str) -> Result<String, DocumentError> {
    fs::read_to_string(file_path).map_err(DocumentError::from)
}

fn extract_from_pdf(file_path: &str) -> Result<String, DocumentError> {
    let bytes = fs::read(file_path)?;
    pdf_extract::extract_text_from_mem(&bytes)
        .map_err(|e| DocumentError::ExtractError(format!("PDF: {}", e)))
}

fn extract_from_docx(file_path: &str) -> Result<String, DocumentError> {
    // Docx is een zip bestand, we extraheren document.xml en parsen de tekst
    use std::io::Read;

    let file = std::fs::File::open(file_path)
        .map_err(|e| DocumentError::ReadError(e))?;
    let mut zip = zip::ZipArchive::new(file)
        .map_err(|e| DocumentError::ExtractError(format!("Zip error: {:?}", e)))?;

    let mut document_xml = zip.by_name("word/document.xml")
        .map_err(|e| DocumentError::ExtractError(format!("word/document.xml niet gevonden: {:?}", e)))?;

    let mut xml_content = String::new();
    document_xml.read_to_string(&mut xml_content)
        .map_err(|e| DocumentError::ExtractError(format!("Read error: {:?}", e)))?;

    // Simpele tekst extractie: neem alles tussen <w:t> tags
    let mut text = String::new();
    let mut in_text_tag = false;
    let mut current_text = String::new();

    let chars: Vec<char> = xml_content.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '<' {
            // Check of dit het begin is van een <w:t> tag
            if i + 4 < chars.len() && chars[i+1] == 'w' && chars[i+2] == ':' && chars[i+3] == 't' && chars[i+4] == '>' {
                in_text_tag = true;
                i += 5;
                continue;
            }
            // Check of dit het einde is van een </w:t> tag
            if i + 6 < chars.len() && chars[i+1] == '/' && chars[i+2] == 'w' && chars[i+3] == ':' && chars[i+4] == 't' && chars[i+5] == '>' {
                text.push_str(&current_text);
                text.push(' ');
                current_text.clear();
                in_text_tag = false;
                i += 6;
                continue;
            }
            // Skip alle andere tags
            while i < chars.len() && chars[i] != '>' {
                i += 1;
            }
            i += 1; // skip de '>'
        } else if in_text_tag {
            current_text.push(chars[i]);
            i += 1;
        } else {
            i += 1;
        }
    }

    Ok(text.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_txt_extraction() {
        // Simple test - would need actual files for integration tests
        assert_eq!(
            extract_from_txt("test.txt").unwrap_err().to_string().contains("No such file"),
            true
        );
    }
}
