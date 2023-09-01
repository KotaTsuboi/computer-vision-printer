extern crate printpdf;
use printpdf::*;
use serde_json::Value;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

static SUBSCRIPTION_KEY: &str = "6670bf7bedfa4e1c837384056c261180";

async fn get_result_url(input_path: &PathBuf) -> Result<String, Box<dyn Error>> {
    let file = tokio::fs::File::open(input_path).await?;
    let client = reqwest::Client::new();
    let url = "https://quality-test-ocr.cognitiveservices.azure.com/vision/v3.2/read/analyze";

    let response = client
        .post(url)
        .body(reqwest::Body::from(file))
        .header("Content-Type", "application/octet-stream")
        .header("Ocp-Apim-Subscription-Key", SUBSCRIPTION_KEY)
        .send()
        .await?;

    let headers = response.headers();
    let result_url = headers["Operation-Location"].to_str().unwrap();

    println!("result url: {}", result_url);

    Ok(result_url.to_string())
}

async fn get(url: String, output_path: &str) -> Result<(), Box<dyn Error>> {
    loop {
        let response = get_response(url.clone()).await?;
        let status = response.status();

        if !status.is_success() {
            // status != 200~299
            println!("{:?}", response);
            panic!("response code is {}", status);
        } else {
            // status == 200~299
            let body = response.text().await?;
            let value: Value = serde_json::from_str(&body).unwrap();
            let status = value["status"].as_str().unwrap();

            match status {
                "notStarted" | "running" => {
                    println!("Status is {}", status);
                    println!("Waiting 10 secs......");
                    thread::sleep(Duration::from_secs(10));
                }
                "failed" => {
                    panic!("OCR failed: {}", body);
                }
                "succeeded" => {
                    println!("OCR succeeded");
                    response_to_pdf(&body, output_path)?;
                    return Ok(());
                }
                _ => {
                    panic!("Unexpected status: {}", body);
                }
            }
        }
    }
}

async fn get_response(url: String) -> Result<reqwest::Response, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let request_builder = client
        .get(url.clone())
        .header("Ocp-Apim-Subscription-Key", SUBSCRIPTION_KEY);

    let response = request_builder.send().await?;

    Ok(response)
}

fn response_to_pdf(response_json: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let value: Value = serde_json::from_str(response_json).unwrap();

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

fn list_file(input_dir: &str) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let paths = fs::read_dir(input_dir).unwrap();
    let mut result: Vec<PathBuf> = Vec::new();

    for dir_entry in paths {
        let dir_entry = dir_entry?;

        if dir_entry.file_type()?.is_dir() {
            continue;
        }

        let file_path = dir_entry.path();

        if file_path.extension().unwrap() != "pdf" {
            continue;
        }

        result.push(file_path);
    }

    Ok(result)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let paths = list_file("input")?;

    for path in paths {
        let url = get_result_url(&path).await?;

        println!("Waiting 10 secs......");
        thread::sleep(Duration::from_secs(10));

        let file_name = path.file_name().unwrap().to_str().unwrap();
        get(url, &format!("output/{}", file_name)).await?;
    }

    Ok(())
}
