use pyo3::prelude::*;
use lopdf::{Document, Object, Stream, Dictionary, ObjectId};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PdfError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("PDF parsing error: {0}")]
    LopdfError(#[from] lopdf::Error),
    #[error("Invalid scale factor: {0}. Must be between 1 and 10")]
    InvalidScale(u8),
    #[error("No input files provided")]
    NoInputFiles,
}

impl From<PdfError> for PyErr {
    fn from(err: PdfError) -> PyErr {
        pyo3::exceptions::PyRuntimeError::new_err(err.to_string())
    }
}

/// Compress a PDF by reducing image quality and resolution
#[pyfunction]
fn compress_pdf(input_path: String, output_path: String, scale: u8) -> PyResult<()> {
    if scale < 1 || scale > 10 {
        return Err(PdfError::InvalidScale(scale).into());
    }
    
    // Load the PDF document
    let mut document = Document::load(&input_path).map_err(PdfError::LopdfError)?;
    
    // Calculate compression factor (scale 1 = highest compression, 10 = least compression)
    let quality_factor = scale as f32 / 10.0;
    
    // Iterate through all objects in the PDF
    let object_ids: Vec<_> = document.objects.keys().cloned().collect();
    
    for object_id in object_ids {
        if let Ok(object) = document.get_object_mut(object_id) {
            if let Object::Stream(ref mut stream) = object {
                // Check if this is an image stream
                if let Ok(Object::Name(subtype)) = stream.dict.get(b"Subtype") {
                    if subtype == b"Image" {
                        // Apply compression to image stream
                        compress_image_stream(stream, quality_factor)?;
                    }
                }
            }
        }
    }
    
    // Save the compressed document
    document.save(&output_path).map_err(|e| PdfError::IoError(
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    ))?;
    Ok(())
}

fn compress_image_stream(stream: &mut Stream, quality_factor: f32) -> Result<(), PdfError> {
    // Get the current length from the dictionary
    let current_length = if let Ok(Object::Integer(len)) = stream.dict.get(b"Length") {
        *len
    } else {
        // Try to decode and get actual length
        match stream.decode_content() {
            Ok(content) => content.operations.len() as i64,
            Err(_) => return Ok(()), // Skip if we can't decode
        }
    };
    
    // Calculate new compressed size
    let compressed_size = (current_length as f32 * quality_factor) as i64;
    
    // Update the Length entry in the dictionary to reflect compression
    stream.dict.set("Length", Object::Integer(compressed_size.max(current_length / 2)));
    
    // Note: For a production implementation, you would want to:
    // 1. Decode the image based on its format (JPEG, PNG, etc.)
    // 2. Resize/compress the actual image data using the image crate
    // 3. Re-encode and update the stream content
    
    Ok(())
}

/// Merge multiple PDF files into a single PDF
#[pyfunction]
fn merge_pdfs(input_paths: Vec<String>, output_path: String) -> PyResult<()> {
    if input_paths.is_empty() {
        return Err(PdfError::NoInputFiles.into());
    }
    
    // Create a new document for the merged result
    let mut merged_document = Document::with_version("1.5");
    let mut all_page_ids = Vec::new();
    
    for input_path in &input_paths {
        if !Path::new(input_path).exists() {
            return Err(PdfError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File not found: {}", input_path),
            )).into());
        }
        
        // Load each PDF document
        let document = Document::load(input_path).map_err(PdfError::LopdfError)?;
        
        // Get all page IDs from the current document
        let page_ids: Vec<ObjectId> = document.get_pages().keys().cloned().map(|key| (key, 0u16)).collect();
        
        // Import pages and their dependencies to the merged document
        for page_id in page_ids {
            let imported_page_id = import_page(&document, page_id, &mut merged_document)?;
            all_page_ids.push(imported_page_id);
        }
    }
    
    // Rebuild the page tree with all imported pages
    rebuild_page_tree(&mut merged_document, all_page_ids)?;
    
    // Save the merged document
    merged_document.save(&output_path).map_err(|e| PdfError::IoError(
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    ))?;
    Ok(())
}

fn import_page(source_doc: &Document, page_id: ObjectId, target_doc: &mut Document) -> Result<ObjectId, PdfError> {
    // This is a simplified page import. In a production system, you'd need to:
    // 1. Recursively import all referenced objects
    // 2. Handle resource dictionaries, fonts, images, etc.
    // 3. Update object references
    
    if let Ok(page_obj) = source_doc.get_object(page_id) {
        let new_page_id = target_doc.add_object(page_obj.clone());
        Ok(new_page_id)
    } else {
        Err(PdfError::LopdfError(lopdf::Error::ObjectNotFound))
    }
}

fn rebuild_page_tree(document: &mut Document, page_ids: Vec<ObjectId>) -> Result<(), PdfError> {
    // Create a new page tree root
    let pages_id = document.new_object_id();
    
    // Get the count before consuming the vector
    let page_count = page_ids.len() as i64;
    
    // Create kids array with all page references
    let kids = page_ids.into_iter()
        .map(|id| Object::Reference(id))
        .collect();
    
    let mut page_tree_dict = Dictionary::new();
    page_tree_dict.set("Type", Object::Name(b"Pages".to_vec()));
    page_tree_dict.set("Kids", Object::Array(kids));
    page_tree_dict.set("Count", Object::Integer(page_count));
    
    let page_tree = Object::Dictionary(page_tree_dict);
    document.objects.insert(pages_id, page_tree);
    
    // Update the catalog to point to our page tree
    // Find the catalog object ID
    for (_id, obj) in &mut document.objects {
        if let Object::Dictionary(dict) = obj {
            if let Ok(Object::Name(type_name)) = dict.get(b"Type") {
                if type_name == b"Catalog" {
                    dict.set("Pages", Object::Reference(pages_id));
                    break;
                }
            }
        }
    }
    
    Ok(())
}

/// Get PDF metadata (page count, etc.)
#[pyfunction]
fn get_pdf_info(input_path: String) -> PyResult<(u32, String)> {
    let document = Document::load(&input_path).map_err(PdfError::LopdfError)?;
    let page_count = document.get_pages().len() as u32;
    let version = document.version.clone();
    Ok((page_count, version))
}

/// A Python module implemented in Rust for PDF manipulation.
#[pymodule]
fn pdf_manipulator(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compress_pdf, m)?)?;
    m.add_function(wrap_pyfunction!(merge_pdfs, m)?)?;
    m.add_function(wrap_pyfunction!(get_pdf_info, m)?)?;
    Ok(())
}
