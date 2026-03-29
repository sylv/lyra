use lyra_keyframe_extractor::extract_keyframes;

#[tokio::main]
async fn main() {
    let input = std::env::args().nth(1).expect("missing input file path");
    let file_path = std::path::Path::new(&input);
    let start = std::time::Instant::now();
    let keyframes = extract_keyframes(&file_path, None).await.unwrap();
    println!("keyframes: {:?}", keyframes);
    println!("extracted keyframes in {:.2?}", start.elapsed());
}
