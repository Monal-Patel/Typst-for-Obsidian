use fast_image_resize as fr;
use typst::layout::PagedDocument;
use typst::World;
use wasm_bindgen::prelude::*;
use web_sys::ImageData;

mod diagnostic;
mod file_entry;
mod render;
mod world;

use crate::world::SystemWorld;

#[wasm_bindgen]
pub struct Compiler {
    resizer: fr::Resizer,
    world: SystemWorld,
    last_document: Option<PagedDocument>,
}

#[wasm_bindgen]
impl Compiler {
    #[wasm_bindgen(constructor)]
    pub fn new(root: String, request_data: &js_sys::Function) -> Self {
        console_error_panic_hook::set_once();

        Self {
            world: SystemWorld::new(root, request_data),
            resizer: fr::Resizer::default(),
            last_document: None,
        }
    }

    pub fn compile_image(
        &mut self,
        text: String,
        path: String,
        pixel_per_pt: f32,
        fill: String,
        size: u32,
        display: bool,
    ) -> Result<ImageData, JsValue> {
        let document = self.world.compile(text, path)?;
        render::to_image(&mut self.resizer, document, pixel_per_pt, fill, size, display)
    }

    pub fn compile_svg(&mut self, text: String, path: String) -> Result<String, JsValue> {
        let document = self.world.compile(text, path)?;
        Ok(render::to_svg(&document))
    }

    pub fn compile_pdf(&mut self, text: String, path: String) -> Result<Vec<u8>, JsValue> {
        let document = self.world.compile(text, path)?;
        let pdf_bytes = render::to_pdf(&document)?;
        self.last_document = Some(document);
        Ok(pdf_bytes)
    }

    pub fn jump_from_click(&self, page: u32, x: f32, y: f32) -> JsValue {
        use typst::layout::{Abs, Point};
        use typst_ide::Jump;

        let doc = match &self.last_document {
            Some(d) => d,
            None => return JsValue::NULL,
        };

        let frame = match doc.pages.get(page as usize) {
            Some(p) => &p.frame,
            None => return JsValue::NULL,
        };

        let click = Point::new(Abs::pt(x as f64), Abs::pt(y as f64));

        match typst_ide::jump_from_click(&self.world, doc, frame, click) {
            Some(Jump::File(file_id, offset)) => {
                if let Ok(source) = self.world.source(file_id) {
                    let text = source.text();
                    let offset = offset.min(text.len());
                    let before = &text[..offset];
                    let line = before.bytes().filter(|&b| b == b'\n').count();
                    let last_newline = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
                    let col = before[last_newline..].len();
                    let obj = js_sys::Object::new();
                    let _ = js_sys::Reflect::set(
                        &obj,
                        &"line".into(),
                        &JsValue::from_f64((line + 1) as f64),
                    );
                    let _ = js_sys::Reflect::set(
                        &obj,
                        &"col".into(),
                        &JsValue::from_f64((col + 1) as f64),
                    );
                    obj.into()
                } else {
                    JsValue::NULL
                }
            }
            _ => JsValue::NULL,
        }
    }

    pub fn jump_from_cursor(&self, line: u32, col: u32) -> JsValue {
        let doc = match &self.last_document {
            Some(d) => d,
            None => return JsValue::NULL,
        };
        if let Ok(source) = self.world.source(self.world.main()) {
            let offset = Self::line_col_to_offset(source.text(), line as usize, col as usize);
            let positions = typst_ide::jump_from_cursor(doc, &source, offset);
            if let Some(pos) = positions.first() {
                let obj = js_sys::Object::new();
                let _ = js_sys::Reflect::set(
                    &obj,
                    &"page".into(),
                    &JsValue::from_f64((pos.page.get() - 1) as f64),
                );
                let _ = js_sys::Reflect::set(
                    &obj,
                    &"x".into(),
                    &JsValue::from_f64(pos.point.x.to_pt()),
                );
                let _ = js_sys::Reflect::set(
                    &obj,
                    &"y".into(),
                    &JsValue::from_f64(pos.point.y.to_pt()),
                );
                obj.into()
            } else {
                JsValue::NULL
            }
        } else {
            JsValue::NULL
        }
    }

    fn line_col_to_offset(text: &str, target_line: usize, target_col: usize) -> usize {
        let line_start: usize = text
            .split('\n')
            .take(target_line.saturating_sub(1))
            .map(|l| l.len() + 1)
            .sum();
        line_start + target_col.saturating_sub(1)
    }

    pub fn format_source(&self, source: String) -> String {
        let typstyle = typstyle_core::Typstyle::default();
        typstyle
            .format_text(source.clone())
            .render()
            .unwrap_or(source)
    }

    pub fn add_font(&mut self, data: Vec<u8>) {
        self.world.add_font(data);
    }

    pub fn reset_fonts(&mut self) {
        self.world.reset_fonts();
    }
}
