use flate2::Compression;
use flate2::write::GzEncoder;
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=java-docs/all-packages-methods.json");

    let out_dir = env::var("OUT_DIR").unwrap();
    let input_path = "java-docs/all-packages-methods.json";
    let output_path = Path::new(&out_dir).join("all-packages-methods.json.gz");

    // Read the JSON file
    let json_data =
        fs::read(input_path).expect("Failed to read java-docs/all-packages-methods.json");

    // Compress the data
    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder
        .write_all(&json_data)
        .expect("Failed to compress JSON data");
    let compressed_data = encoder.finish().expect("Failed to finish compression");

    // Write compressed data to output directory
    fs::write(&output_path, &compressed_data).expect("Failed to write compressed JSON");

    let original_size = json_data.len() as f64 / (1024.0 * 1024.0);
    let compressed_size = compressed_data.len() as f64 / (1024.0 * 1024.0);
    let ratio = (compressed_size / original_size) * 100.0;

    println!("cargo:warning=Java docs compression complete:");
    println!("cargo:warning=  Original size: {:.2} MB", original_size);
    println!("cargo:warning=  Compressed size: {:.2} MB", compressed_size);
    println!("cargo:warning=  Compression ratio: {:.1}%", ratio);
}
