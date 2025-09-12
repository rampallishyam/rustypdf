use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::io::Cursor;
use image::codecs::jpeg::JpegEncoder;

use lopdf::{Document, Object, ObjectId};
use image::GenericImageView;
use thiserror::Error;

#[derive(Error, Debug)]
enum RustyPdfError {
    #[error("IO error: {0}")] 
    Io(#[from] std::io::Error),
    #[error("PDF parse error: {0}")]
    PdfParse(#[from] lopdf::Error),
    #[error("Invalid scale value (expected 1..=10): {0}")]
    BadScale(i32),
}

impl From<RustyPdfError> for PyErr {
    fn from(e: RustyPdfError) -> Self {
        PyValueError::new_err(e.to_string())
    }
}

/// Merge multiple PDFs preserving page order.
fn merge_impl(inputs: &[&str], output: &str) -> Result<(), RustyPdfError> {
    let mut target_doc = Document::with_version("1.5");
    let mut max_id = 1u32;

    // Helper to resolve inheritable key up the Pages tree
    fn resolve_inherited(doc: &Document, id: ObjectId, key: &[u8]) -> Option<Object> {
        let mut current_id = Some(id);
        while let Some(cid) = current_id {
            if let Ok(obj) = doc.get_object(cid) {
                if let Ok(d) = obj.as_dict() {
                    if let Ok(v) = d.get(key) { return Some(v.clone()); }
                    if let Ok(&Object::Reference(parent_id)) = d.get(b"Parent") { current_id = Some(parent_id); } else { current_id = None; }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        None
    }

    for path in inputs {
        let mut doc = Document::load(path)?;
        // Renumber to avoid collisions in the target document
        doc.renumber_objects_with(max_id);
        max_id = doc.max_id + 1;

        // Ensure inheritable attributes are explicit on each Page before we remove original Pages tree
        let page_map = doc.get_pages(); // page_num -> ObjectId
        for (_page_num, page_id) in page_map {
            // First determine which keys are missing and resolve their values (immutable borrows only)
            let mut to_set: Vec<(&[u8], Object)> = Vec::new();
            if let Ok(obj) = doc.get_object(page_id) {
                if let Ok(dict) = obj.as_dict() {
                    for &key in [&b"Resources"[..], &b"MediaBox"[..], &b"CropBox"[..], &b"Rotate"[..]].iter() {
                        if !dict.has(key) {
                            if let Some(val) = resolve_inherited(&doc, page_id, key) { to_set.push((key, val)); }
                        }
                    }
                }
            }
            // Now mutably set the resolved values
            if !to_set.is_empty() {
                if let Ok(page_obj) = doc.get_object_mut(page_id) {
                    if let Ok(dict) = page_obj.as_dict_mut() {
                        for (key, val) in to_set { dict.set(key, val); }
                    }
                }
            }
        }

        // Move all objects into target
        let moved: Vec<(ObjectId, Object)> = doc.objects.iter().map(|(id, o)| (*id, o.clone())).collect();
        target_doc.objects.extend(moved.into_iter());
    }

    // Build new Pages tree (simple concatenation)
    let mut page_refs: Vec<Object> = Vec::new();
    for (id, obj) in target_doc.objects.iter() {
        if let Ok(dict) = obj.as_dict() {
            if let Ok(Object::Name(t)) = dict.get(b"Type") {
                if t == b"Page" { page_refs.push(Object::Reference(*id)); }
            }
        }
    }

    // Create Pages root
    let pages_id = target_doc.add_object(lopdf::dictionary!{ "Type" => "Pages", "Kids" => Object::Array(page_refs.clone()), "Count" => page_refs.len() as i64 });

    // Update each page's Parent pointer
    for obj in target_doc.objects.values_mut() {
        if let Ok(dict) = obj.as_dict_mut() {
            if let Ok(Object::Name(t)) = dict.get(b"Type") {
                if t == b"Page" { dict.set("Parent", pages_id); }
            }
        }
    }

    // Catalog
    let catalog_id = target_doc.add_object(lopdf::dictionary!{ "Type" => "Catalog", "Pages" => pages_id });
    target_doc.trailer.set("Root", catalog_id);

    target_doc.compress();
    target_doc.save(output)?;
    Ok(())
}

/// Very naive compression: downscale JPEG/PNG images by a factor derived from scale (1..10) and re-embed.
/// This is a placeholder; full fidelity PDF image handling is complex.
fn compress_impl(input: &str, output: &str, scale: i32) -> Result<(), RustyPdfError> {
    if !(1..=10).contains(&scale) { return Err(RustyPdfError::BadScale(scale)); }

    let mut doc = Document::load(input)?;

    // scale_factor 1.0 (scale=1) -> 0.25 (scale=10) (linear mapping)
    let scale_factor = 1.0 - ((scale - 1) as f32 / 9.0) * 0.75; // 1.0 .. 0.25

    let mut to_update: Vec<ObjectId> = Vec::new();
    for (id, obj) in doc.objects.iter() {
        if let Ok(stream) = obj.as_stream() {
            if let Ok(Object::Name(subtype)) = stream.dict.get(b"Subtype") {
                if subtype == b"Image" { to_update.push(*id); }
            }
        }
    }

    for id in to_update {
        if let Some(obj) = doc.objects.get_mut(&id) {
            if let Ok(stream) = obj.as_stream_mut() {
                let data = stream.content.clone();
                // JPEG: starts with 0xFFD8
                if data.starts_with(&[0xFF, 0xD8]) {
                    if let Ok(img) = image::load_from_memory(&data) {
                        let (w, h) = img.dimensions();
                        let new_w = ((w as f32) * scale_factor).max(1.0) as u32;
                        let new_h = ((h as f32) * scale_factor).max(1.0) as u32;
                        let resized = img.resize(new_w, new_h, image::imageops::FilterType::Lanczos3);
                        let mut buf: Vec<u8> = Vec::new();
                        // JPEG quality: 100 (scale=1) to 30 (scale=10)
                        let jpeg_quality = 100 - ((scale - 1) as u8 * 70 / 9); // 100..30
                        let mut encoder = JpegEncoder::new_with_quality(&mut buf, jpeg_quality);
                        if encoder.encode_image(&resized).is_ok() {
                            stream.set_plain_content(buf);
                            stream.dict.set("Width", new_w as i64);
                            stream.dict.set("Height", new_h as i64);
                        }
                    }
                } else if data.starts_with(&[0x89, b'P', b'N', b'G']) {
                    if let Ok(img) = image::load_from_memory(&data) {
                        let (w, h) = img.dimensions();
                        let new_w = ((w as f32) * scale_factor).max(1.0) as u32;
                        let new_h = ((h as f32) * scale_factor).max(1.0) as u32;
                        let resized = img.resize(new_w, new_h, image::imageops::FilterType::Lanczos3);
                        let mut buf: Vec<u8> = Vec::new();
                        if resized.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png).is_ok() {
                            stream.set_plain_content(buf);
                            stream.dict.set("Width", new_w as i64);
                            stream.dict.set("Height", new_h as i64);
                        }
                    }
                } else {
                    // Unsupported image type -> skip
                    continue;
                }
            }
        }
    }

    doc.compress();
    doc.save(output)?;
    Ok(())
}

#[pyfunction]
fn merge_pdfs(py: Python<'_>, inputs: Vec<String>, output: String) -> PyResult<()> {
    // release GIL while heavy work
    py.allow_threads(|| merge_impl(&inputs.iter().map(|s| s.as_str()).collect::<Vec<_>>() , &output))?;
    Ok(())
}

#[pyfunction]
fn compress_pdf(py: Python<'_>, input: String, output: String, scale: i32) -> PyResult<()> {
    py.allow_threads(|| compress_impl(&input, &output, scale))?;
    Ok(())
}

#[pymodule]
fn _rustypdf(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(merge_pdfs, m)?)?;
    m.add_function(wrap_pyfunction!(compress_pdf, m)?)?;
    Ok(())
}
