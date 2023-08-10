extern crate printpdf;
use printpdf::*;
use serde_json::Value;
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::io::Read;

fn read_file(path: String) -> String {
    let mut file = File::open(path).unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    data
}

async fn get_response(input_path: &str) -> Result<(), Box<dyn Error>> {
    let file = tokio::fs::File::open(input_path).await?;
    let client = reqwest::Client::new();
    let url = "https://quality-test-ocr.cognitiveservices.azure.com/vision/v3.2/read/analyze";

    let response = client
        .post(url)
        .body(reqwest::Body::from(file))
        .header("Content-Type", "application/octet-stream")
        .header(
            "Ocp-Apim-Subscription-Key",
            "{key}",
        )
        .send()
        .await?;

    println!("{:?}", response);

    Ok(())
}

fn response_to_pdf(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let data = read_file(input_path.to_string());
    let value: Value = serde_json::from_str(&data).unwrap();

    let (doc, page1, layer1) =
        PdfDocument::new("PDF_Document_title", Mm(210.0), Mm(297.0), "Layer 1");
    let current_layer = doc.get_page(page1).get_layer(layer1);

    let mut font_reader =
        std::io::Cursor::new(include_bytes!("../assets/fonts/yumin.ttf").as_ref());

    let font = doc.add_external_font(&mut font_reader).unwrap();

    for line in value["analyzeResult"]["readResults"][0]["lines"]
        .as_array()
        .unwrap()
    {
        let text = line["text"].as_str().unwrap();
        let bbox: Vec<f64> = line["boundingBox"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e.as_f64().unwrap())
            .collect();
        let x_mm = 25.4 * (bbox[0] + bbox[2] + bbox[4] + bbox[6]) / 4.0;
        let y_mm = 297.0 - 25.4 * ((bbox[1] + bbox[3] + bbox[5] + bbox[7]) / 4.0);
        //let height = 72.0 * ((bbox[7] - bbox[1]) + (bbox[5] - bbox[3])) / 2.0;
        let height = 10.0;

        // `use_text` is a wrapper around making a simple string
        // 左下を基準として座標を指定する
        // 単位はMmとPtとPxがありそう
        current_layer.use_text(text, height, Mm(x_mm), Mm(y_mm), &font);
    }

    doc.save(&mut BufWriter::new(File::create(output_path).unwrap()))
        .unwrap();
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    get_response("ocr.pdf").await?;
    //response_to_pdf("response.json", "test_fonts.pdf")?;
    Ok(())
}
