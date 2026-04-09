use anyhow::Result;
use futures_util::StreamExt;
use hex_literal::hex;
use icu_locale::Locale;
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio::{
    fs::{self, File},
    io::{AsyncWriteExt, BufWriter},
};

#[derive(Debug)]
pub struct Model {
    pub model_url: &'static str,
    pub model_sha256: [u8; 32],
    pub languages_bcp47: Option<&'static [&'static str]>,
    pub dict_url: Option<&'static str>,
    pub dict_sha256: Option<[u8; 32]>,
}

pub const DET_MODELS: [Model; 1] = [Model {
    model_url: "https://www.modelscope.cn/models/RapidAI/RapidOCR/resolve/v3.6.0/onnx/PP-OCRv5/det/ch_PP-OCRv5_mobile_det.onnx",
    model_sha256: hex!("4d97c44a20d30a81aad087d6a396b08f786c4635742afc391f6621f5c6ae78ae"),
    languages_bcp47: None,
    dict_url: None,
    dict_sha256: None,
}];

const DEFAULT_RET_MODEL_INDEX: usize = 8;

pub const RET_MODELS: [Model; 12] = [
    Model {
        model_url: "https://www.modelscope.cn/models/RapidAI/RapidOCR/resolve/v3.6.0/onnx/PP-OCRv5/rec/arabic_PP-OCRv5_rec_mobile_infer.onnx",
        model_sha256: hex!("c1192e632d0baa9146ae5b756a0e635e3dc63c1733737ebfd1629e87144e9295"),
        languages_bcp47: Some(&[
            "ar",      // Arabic
            "fa",      // Persian
            "ug",      // Uyghur
            "ur",      // Urdu
            "ps",      // Pashto
            "sd",      // Sindhi
            "bal",     // Balochi
            "ku-Arab", // Kurdish (Arabic script)
        ]),
        dict_url: Some(
            "https://www.modelscope.cn/models/RapidAI/RapidOCR/resolve/v3.6.0/paddle/PP-OCRv5/rec/arabic_PP-OCRv5_rec_mobile_infer/ppocrv5_arabic_dict.txt",
        ),
        dict_sha256: Some(hex!(
            "7f92f7dbb9b75a4787a83bfb4f6d14a8ab515525130c9d40a9036f61cf6999e9"
        )),
    },
    Model {
        model_url: "https://www.modelscope.cn/models/RapidAI/RapidOCR/resolve/v3.6.0/onnx/PP-OCRv5/rec/ch_PP-OCRv5_rec_mobile_infer.onnx",
        model_sha256: hex!("5825fc7ebf84ae7a412be049820b4d86d77620f204a041697b0494669b1742c5"),
        languages_bcp47: Some(&["zh"]),
        dict_url: Some(
            "https://www.modelscope.cn/models/RapidAI/RapidOCR/resolve/v3.6.0/paddle/PP-OCRv5/rec/ch_PP-OCRv5_rec_mobile_infer/ppocrv5_dict.txt",
        ),
        dict_sha256: Some(hex!(
            "d1979e9f794c464c0d2e0b70a7fe14dd978e9dc644c0e71f14158cdf8342af1b"
        )),
    },
    Model {
        model_url: "https://www.modelscope.cn/models/RapidAI/RapidOCR/resolve/v3.6.0/onnx/PP-OCRv5/rec/cyrillic_PP-OCRv5_rec_mobile_infer.onnx",
        model_sha256: hex!("90f761b4bfcce0c8c561c0cb5c887b0971d3ec01c32164bdf7374a35b0982711"),
        languages_bcp47: Some(&[
            "sr-Cyrl",    // Serbian (Cyrillic)
            "bg",         // Bulgarian
            "mn-Cyrl",    // Mongolian (Cyrillic)
            "ab",         // Abkhaz
            "ady",        // Adyghe
            "kbd",        // Kabardian
            "av",         // Avar
            "dar",        // Dargwa
            "inh",        // Ingush
            "ce",         // Chechen
            "lbe",        // Lak
            "lez",        // Lezgian
            "tab",        // Tabasaran
            "kk-Cyrl",    // Kazakh
            "ky-Cyrl",    // Kyrgyz
            "tg-Cyrl",    // Tajik
            "mk",         // Macedonian
            "tt-Cyrl",    // Tatar
            "cv",         // Chuvash
            "ba",         // Bashkir
            "chm",        // Mari
            "ro-Cyrl-MD", // Moldovan (historical Cyrillic form)
            "udm",        // Udmurt
            "kv",         // Komi
            "os",         // Ossetian
            "bua",        // Buryat
            "xal",        // Kalmyk
            "tyv",        // Tuvan
            "sah",        // Yakut
            "kaa-Cyrl",   // Karakalpak
        ]),
        dict_url: Some(
            "https://www.modelscope.cn/models/RapidAI/RapidOCR/resolve/v3.6.0/paddle/PP-OCRv5/rec/cyrillic_PP-OCRv5_rec_mobile_infer/ppocrv5_cyrillic_dict.txt",
        ),
        dict_sha256: Some(hex!(
            "db40aa52ceb112055be80c694afdf655d5d2c4f7873704524cc16a447ca913ba"
        )),
    },
    Model {
        model_url: "https://www.modelscope.cn/models/RapidAI/RapidOCR/resolve/v3.6.0/onnx/PP-OCRv5/rec/devanagari_PP-OCRv5_rec_mobile_infer.onnx",
        model_sha256: hex!("d6f0a906580e3fa6b324a318718f1f31f268b6ea8ef985f91c2012a37f52c91e"),
        languages_bcp47: Some(&[
            "hi",  // Hindi
            "mr",  // Marathi
            "ne",  // Nepali
            "bih", // Bihari
            "mai", // Maithili
            "anp", // Angika
            "bho", // Bhojpuri
            "mag", // Magahi
            "sat", // Santali
            "new", // Newari
            "kok", // Konkani
            "sa",  // Sanskrit
            "bgc", // Haryanvi
        ]),
        dict_url: Some(
            "https://www.modelscope.cn/models/RapidAI/RapidOCR/resolve/v3.6.0/paddle/PP-OCRv5/rec/devanagari_PP-OCRv5_rec_mobile_infer/ppocrv5_devanagari_dict.txt",
        ),
        dict_sha256: Some(hex!(
            "09c7440bfc5477e5c41052304b6b185aff8c4a5e8b2b4c23c1c706f6fe1ee9fc"
        )),
    },
    Model {
        model_url: "https://www.modelscope.cn/models/RapidAI/RapidOCR/resolve/v3.6.0/onnx/PP-OCRv5/rec/el_PP-OCRv5_rec_mobile_infer.onnx",
        model_sha256: hex!("b4368bccd557123c702b7549fee6cd1e94b581337d1c9b65310f109131542b7f"),
        languages_bcp47: Some(&["el"]),
        dict_url: Some(
            "https://www.modelscope.cn/models/RapidAI/RapidOCR/resolve/v3.6.0/paddle/PP-OCRv5/rec/el_PP-OCRv5_rec_mobile_infer/ppocrv5_el_dict.txt",
        ),
        dict_sha256: Some(hex!(
            "31defc62c0c3ad3674a82da6192226a2ba98ef4ff014a7045cb88d59f9c3de31"
        )),
    },
    Model {
        model_url: "https://www.modelscope.cn/models/RapidAI/RapidOCR/resolve/v3.6.0/onnx/PP-OCRv5/rec/en_PP-OCRv5_rec_mobile_infer.onnx",
        model_sha256: hex!("c3461add59bb4323ecba96a492ab75e06dda42467c9e3d0c18db5d1d21924be8"),
        languages_bcp47: Some(&["en"]),
        dict_url: Some(
            "https://www.modelscope.cn/models/RapidAI/RapidOCR/resolve/v3.6.0/paddle/PP-OCRv5/rec/en_PP-OCRv5_rec_mobile_infer/ppocrv5_en_dict.txt",
        ),
        dict_sha256: Some(hex!(
            "e025a66d31f327ba0c232e03f407ae8d105e1e709e7ccb3f408aa778c24e70d6"
        )),
    },
    Model {
        model_url: "https://www.modelscope.cn/models/RapidAI/RapidOCR/resolve/v3.6.0/onnx/PP-OCRv5/rec/eslav_PP-OCRv5_rec_mobile_infer.onnx",
        model_sha256: hex!("08705d6721849b1347d26187f15a5e362c431963a2a62bfff4feac578c489aab"),
        languages_bcp47: Some(&["ru", "be", "uk"]),
        dict_url: Some(
            "https://www.modelscope.cn/models/RapidAI/RapidOCR/resolve/v3.6.0/paddle/PP-OCRv5/rec/eslav_PP-OCRv5_rec_mobile_infer/ppocrv5_eslav_dict.txt",
        ),
        dict_sha256: Some(hex!(
            "3e95f1581557162870cacdba5af91a4c6be2890710d395b0c3c7578e7ee5e6eb"
        )),
    },
    Model {
        model_url: "https://www.modelscope.cn/models/RapidAI/RapidOCR/resolve/v3.6.0/onnx/PP-OCRv5/rec/korean_PP-OCRv5_rec_mobile_infer.onnx",
        model_sha256: hex!("cd6e2ea50f6943ca7271eb8c56a877a5a90720b7047fe9c41a2e541a25773c9b"),
        languages_bcp47: Some(&["ko"]),
        dict_url: Some(
            "https://www.modelscope.cn/models/RapidAI/RapidOCR/resolve/v3.6.0/paddle/PP-OCRv5/rec/korean_PP-OCRv5_rec_mobile_infer/ppocrv5_korean_dict.txt",
        ),
        dict_sha256: Some(hex!(
            "a88071c68c01707489baa79ebe0405b7beb5cca229f4fc94cc3ef992328802d7"
        )),
    },
    Model {
        model_url: "https://www.modelscope.cn/models/RapidAI/RapidOCR/resolve/v3.6.0/onnx/PP-OCRv5/rec/latin_PP-OCRv5_rec_mobile_infer.onnx",
        model_sha256: hex!("b20bd37c168a570f583afbc8cd7925603890efbcdc000a59e22c269d160b5f5a"),
        languages_bcp47: Some(&[
            "fr", "de", "af", "it", "es", "bs", "pt", "cs", "cy", "da", "et", "ga", "hr",
            "uz-Latn", "hu", "sr-Latn", "id", "oc", "is", "lt", "mi", "ms", "nl", "no", "pl", "sk",
            "sl", "sq", "sv", "sw", "tl", "tr", "la", "az-Latn", "ku-Latn", "lv", "mt", "pi", "ro",
            "vi", "fi", "eu", "gl", "lb", "rm", "ca", "qu",
        ]),
        dict_url: Some(
            "https://www.modelscope.cn/models/RapidAI/RapidOCR/resolve/v3.6.0/paddle/PP-OCRv5/rec/latin_PP-OCRv5_rec_mobile_infer/ppocrv5_latin_dict.txt",
        ),
        dict_sha256: Some(hex!(
            "3c0a8a79b612653c25f765271714f71281e4e955962c153e272b7b8c1d2b13ff"
        )),
    },
    Model {
        model_url: "https://www.modelscope.cn/models/RapidAI/RapidOCR/resolve/v3.6.0/onnx/PP-OCRv5/rec/ta_PP-OCRv5_rec_mobile_infer.onnx",
        model_sha256: hex!("a42448808b7dea87597336f12438935f40353f1949e8360acd9e06b4da21bfe1"),
        languages_bcp47: Some(&["ta"]),
        dict_url: Some(
            "https://www.modelscope.cn/models/RapidAI/RapidOCR/resolve/v3.6.0/paddle/PP-OCRv5/rec/ta_PP-OCRv5_rec_mobile_infer/ppocrv5_ta_dict.txt",
        ),
        dict_sha256: Some(hex!(
            "85b541352ae18dc6ba6d47152d8bf8adff6b0266e605d2eef2990c1bf466117b"
        )),
    },
    Model {
        model_url: "https://www.modelscope.cn/models/RapidAI/RapidOCR/resolve/v3.6.0/onnx/PP-OCRv5/rec/te_PP-OCRv5_rec_mobile_infer.onnx",
        model_sha256: hex!("a3690451b50028a09a3316a1274f7c05728151ea3f8fd392696397a7fefcbd92"),
        languages_bcp47: Some(&["te"]),
        dict_url: Some(
            "https://www.modelscope.cn/models/RapidAI/RapidOCR/resolve/v3.6.0/paddle/PP-OCRv5/rec/te_PP-OCRv5_rec_mobile_infer/ppocrv5_te_dict.txt",
        ),
        dict_sha256: Some(hex!(
            "42f83f5d3fdb50778e4fa5b66c58d99a59ab7792151c5e74f34b8ffd7b61c9d6"
        )),
    },
    Model {
        model_url: "https://www.modelscope.cn/models/RapidAI/RapidOCR/resolve/v3.6.0/onnx/PP-OCRv5/rec/th_PP-OCRv5_rec_mobile_infer.onnx",
        model_sha256: hex!("de541dd83161c241ff426f7ecfd602a0ba77d686cf3ab9a6c255ea82fd08006e"),
        languages_bcp47: Some(&["th"]),
        dict_url: Some(
            "https://www.modelscope.cn/models/RapidAI/RapidOCR/resolve/v3.6.0/paddle/PP-OCRv5/rec/th_PP-OCRv5_rec_mobile_infer/ppocrv5_th_dict.txt",
        ),
        dict_sha256: Some(hex!(
            "57f5406f94bb6688fb7077f7be65f08bbd71cecf48c01ea26c522cb5c4836b7a"
        )),
    },
];

fn locale_matches(requested: &Locale, supported: &str) -> Option<u8> {
    let supported: Locale = supported.parse().ok()?;

    let req_lang = requested.id.language;
    let req_script = requested.id.script;
    let req_region = requested.id.region;

    let sup_lang = supported.id.language;
    let sup_script = supported.id.script;
    let sup_region = supported.id.region;

    // 0 = best
    if req_lang == sup_lang && req_script == sup_script && req_region == sup_region {
        return Some(0);
    }

    // Match language+script, ignoring region.
    if req_lang == sup_lang && req_script.is_some() && req_script == sup_script {
        return Some(1);
    }

    // Match plain language, only when the supported tag is also plain language.
    // This prevents `sr` from ambiguously matching `sr-Latn` or `sr-Cyrl`.
    if req_lang == sup_lang && req_script.is_none() && sup_script.is_none() {
        return Some(2);
    }

    None
}

fn best_ret_model(lang_tag: &str) -> Option<&'static Model> {
    let requested: Locale = lang_tag.parse().ok()?;

    let mut best: Option<(&'static Model, u8)> = None;

    for model in RET_MODELS.iter() {
        let Some(langs) = model.languages_bcp47 else {
            continue;
        };

        for &supported in langs {
            if let Some(score) = locale_matches(&requested, supported) {
                match best {
                    None => best = Some((model, score)),
                    Some((_, best_score)) if score < best_score => best = Some((model, score)),
                    _ => {}
                }
            }
        }
    }

    best.map(|(model, _)| model)
}

pub fn best_models_for_language(lang_tag: Option<&str>) -> (&'static Model, &'static Model) {
    let ret = lang_tag
        .and_then(best_ret_model)
        .unwrap_or(&RET_MODELS[DEFAULT_RET_MODEL_INDEX]);
    (&DET_MODELS[0], ret)
}

pub async fn upsert_model(
    model_dir: &PathBuf,
    model: &Model,
) -> Result<(PathBuf, Option<PathBuf>)> {
    if !model_dir.exists() {
        tokio::fs::create_dir_all(model_dir).await?;
    }

    let mut client: Option<Client> = None;

    let model_path = model_dir.join(format!("{}.onnx", hex::encode(model.model_sha256)));
    if !model_path.exists() {
        let client = client.get_or_insert_with(build_client);
        download_and_verify_sha256(client, model.model_url, &model_path, &model.model_sha256)
            .await?;
    }

    let mut dict_path = None;
    match (&model.dict_url, &model.dict_sha256) {
        (Some(url), Some(expected_sha256)) => {
            let dict_path = dict_path.get_or_insert_with(|| model_path.with_extension("dict.txt"));
            if !dict_path.exists() {
                let client = client.get_or_insert_with(build_client);
                download_and_verify_sha256(client, url, &dict_path, expected_sha256).await?;
            }
        }
        (None, None) => {}
        _ => {
            return Err(anyhow::anyhow!(
                "model dict url and sha256 must both be present or both be absent"
            ));
        }
    }

    Ok((model_path, dict_path))
}

pub async fn download_and_verify_sha256(
    client: &Client,
    url: &str,
    dest: impl AsRef<Path>,
    expected_sha256: &[u8; 32],
) -> anyhow::Result<()> {
    let response = client.get(url).send().await?.error_for_status()?;
    let mut stream = response.bytes_stream();

    let dest = dest.as_ref();
    let temp_dest = temp_download_path(dest);
    let _ = fs::remove_file(&temp_dest).await;

    let file = File::create(&temp_dest).await?;
    let mut writer = BufWriter::new(file);
    let mut hasher = Sha256::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        hasher.update(&chunk);
        writer.write_all(&chunk).await?;
    }

    writer.flush().await?;
    // Optional but useful if you care that the bytes are really on disk:
    writer.get_ref().sync_all().await?;

    let actual = hasher.finalize();
    if actual.as_slice() != expected_sha256 {
        let _ = fs::remove_file(&temp_dest).await;
        anyhow::bail!(
            "sha256 mismatch: expected {:x?}, got {:x?}",
            expected_sha256,
            actual
        );
    }

    fs::rename(&temp_dest, dest).await?;
    tracing::debug!("successfully downloaded {} with verified sha256", url);
    Ok(())
}

// Temporary downloads live beside the final artifact so promotion is an atomic rename.
fn temp_download_path(dest: &Path) -> PathBuf {
    let mut extension = dest
        .extension()
        .map(|ext| ext.to_string_lossy().into_owned())
        .unwrap_or_default();

    if extension.is_empty() {
        extension.push_str("part");
    } else {
        extension.push_str(".part");
    }

    dest.with_extension(extension)
}

fn build_client() -> Client {
    let user_agent = format!("lyra/{}", env!("CARGO_PKG_VERSION"));
    Client::builder().user_agent(user_agent).build().unwrap()
}
