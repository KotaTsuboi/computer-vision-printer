extern crate printpdf;
use printpdf::*;
use std::fs::File;
use std::io::BufWriter;

fn main() {
    let (doc, page1, layer1) =
        PdfDocument::new("PDF_Document_title", Mm(210.0), Mm(297.0), "Layer 1");
    let current_layer = doc.get_page(page1).get_layer(layer1);

    let text = "Lorem ipsum";

    let mut font_reader =
        std::io::Cursor::new(include_bytes!("../assets/fonts/RobotoMedium.ttf").as_ref());

    let font = doc.add_external_font(&mut font_reader).unwrap();

    // `use_text` is a wrapper around making a simple string
    // 左下を基準として座標を指定する
    // 単位はMmとPtとPxがありそう
    current_layer.use_text(text, 10.0, Mm(20.0), Mm(20.0), &font);

    doc.save(&mut BufWriter::new(File::create("test_fonts.pdf").unwrap()))
        .unwrap();
}
