use lyra_probe::probe;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let file_path = std::env::args()
        .nth(1)
        .expect("Usage: lyra-probe <media_file>");
    let file_path = std::path::Path::new(&file_path);

    // let keyframes = keyframes::extract_keyframes(file_path, None).await?;
    // if let Some(keyframes) = keyframes {
    //     println!("Keyframe timestamps (seconds):");
    //     for timestamp in keyframes {
    //         println!("{}", timestamp);
    //     }
    // } else {
    //     println!("Keyframe extraction was cancelled.");
    // }

    let probe = probe(file_path).await?;
    println!("Media info: {:#?}", probe);

    Ok(())
}
