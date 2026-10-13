//! The two VieNeu-TTS v3 models, the files each needs and their preset
//! voices.
//!
//! Every file is fetched from where its authors publish it, pinned to the
//! commit this port was checked against: the ONNX graphs from the
//! VieNeu-TTS and MOSS Audio Tokenizer repositories on Hugging Face, the
//! preset voices from the VieNeu-TTS GitHub repository, and the phoneme
//! dictionary from sea-g2p's. Nothing is re-hosted.

use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::{nano, turbo, HF, SEA_G2P_DICT, SEA_G2P_DICT_URL, VIENEU_RAW};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Model {
    /// VieNeu-TTS v3 Turbo: the most natural voices, 48 kHz, 25 presets;
    /// about real time on 4 CPU cores.
    Turbo,
    /// VieNeu-TTS v3 Nano: smaller and faster, 24 kHz, 11 presets.
    Nano,
}

impl Model {
    pub const ALL: [Model; 2] = [Model::Turbo, Model::Nano];

    /// A short id, also a good directory name.
    pub fn id(self) -> &'static str {
        match self {
            Model::Turbo => "vieneu-tts-v3-turbo",
            Model::Nano => "vieneu-tts-v3-nano",
        }
    }

    pub fn sample_rate(self) -> u32 {
        match self {
            Model::Turbo => turbo::SAMPLE_RATE,
            Model::Nano => nano::SAMPLE_RATE,
        }
    }

    /// VieNeu's own default voice for the model.
    pub fn default_voice(self) -> &'static str {
        match self {
            Model::Turbo => "Hải Đăng",
            Model::Nano => "Minh Quân",
        }
    }

    /// The Hugging Face page of the model, with its card and license.
    pub fn homepage(self) -> String {
        match self {
            Model::Turbo => format!("{HF}/{}", turbo::REPO),
            Model::Nano => format!("{HF}/{}", nano::REPO),
        }
    }

    /// Download size in bytes of all of [`Model::files`].
    pub fn size(self) -> u64 {
        match self {
            Model::Turbo => 583_291_956,
            Model::Nano => 346_903_105,
        }
    }

    /// `(url, file name)` of every file the model needs in its directory.
    pub fn files(self) -> Vec<(String, String)> {
        let hf = |repo: &str, rev: &str, path: &str| format!("{HF}/{repo}/resolve/{rev}/{path}");
        let name = |f: &str| f.rsplit('/').next().unwrap_or(f).to_string();
        let mut files: Vec<(String, String)> = match self {
            Model::Turbo => turbo::FILES
                .iter()
                .map(|f| {
                    let path = format!("{}/{f}", turbo::SUBFOLDER);
                    (hf(turbo::REPO, turbo::REVISION, &path), name(f))
                })
                .chain(
                    turbo::CODEC_FILES
                        .iter()
                        .map(|f| (hf(turbo::CODEC_REPO, turbo::CODEC_REVISION, f), name(f))),
                )
                .chain([(
                    format!("{VIENEU_RAW}/src/vieneu/assets/{}", turbo::VOICES),
                    turbo::VOICES.into(),
                )])
                .collect(),
            Model::Nano => nano::FILES
                .iter()
                .map(|f| (hf(nano::REPO, nano::REVISION, f), name(f)))
                .chain([(
                    format!("{VIENEU_RAW}/src/vieneu/assets/{}", nano::VOICES),
                    nano::VOICES.into(),
                )])
                .collect(),
        };
        files.push((SEA_G2P_DICT_URL.into(), SEA_G2P_DICT.into()));
        files
    }

    /// Whether every file is present in `dir`.
    pub fn is_complete(self, dir: &Path) -> bool {
        self.files().iter().all(|(_, n)| dir.join(n).is_file())
    }

    fn voices_file(self) -> &'static str {
        match self {
            Model::Turbo => turbo::VOICES,
            Model::Nano => nano::VOICES,
        }
    }
}

/// A preset voice.
#[derive(Debug, Clone)]
pub struct Voice {
    pub name: String,
    /// VieNeu's description, e.g. "Nam · Bắc · Phong cách tự nhiên"
    /// (gender, region, style).
    pub description: String,
    pub gender: String,
}

/// The preset voices of a downloaded model, in VieNeu's order.
pub fn voices(model: Model, dir: &Path) -> Result<Vec<Voice>> {
    #[derive(Deserialize)]
    struct File {
        presets: serde_json::Map<String, serde_json::Value>,
    }
    #[derive(Deserialize)]
    struct Meta {
        #[serde(default)]
        description: String,
        #[serde(default)]
        gender: String,
    }
    let path = dir.join(model.voices_file());
    let bytes = std::fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
    // serde_json is built with `preserve_order`, so presets keep file order.
    let file: File = serde_json::from_slice(&bytes)?;
    let voices = file
        .presets
        .into_iter()
        .map(|(name, v)| {
            let meta: Meta = serde_json::from_value(v).unwrap_or(Meta {
                description: String::new(),
                gender: String::new(),
            });
            Voice {
                name,
                description: meta.description,
                gender: meta.gender,
            }
        })
        .collect();
    Ok(voices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn files_point_at_the_authors_repositories() {
        let turbo = Model::Turbo.files();
        assert!(turbo.iter().any(|(u, n)| n == "vieneu_prefill.onnx"
            && u == "https://huggingface.co/pnnbao-ump/VieNeu-TTS-v3-Turbo/resolve/61b85e3d937fbbacb387714180e8182823512523/onnx_update/vieneu_prefill.onnx"));
        assert!(turbo
            .iter()
            .any(|(u, _)| u.contains("/OpenMOSS-Team/MOSS-Audio-Tokenizer-Nano-ONNX/")));
        assert!(turbo.iter().any(|(_, n)| n == "voices_v3_turbo.json"));
        assert_eq!(Model::Nano.files().len(), 8);
        for m in Model::ALL {
            assert!(m.files().iter().any(|(_, n)| n == SEA_G2P_DICT));
            assert!(m
                .files()
                .iter()
                .all(|(u, _)| u.starts_with("https://huggingface.co/pnnbao-ump/")
                    || u.starts_with("https://huggingface.co/OpenMOSS-Team/")
                    || u.starts_with("https://raw.githubusercontent.com/pnnbao97/")));
        }
    }
}
