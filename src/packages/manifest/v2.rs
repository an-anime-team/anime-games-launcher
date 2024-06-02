use std::collections::HashMap;

use serde_json::Value as Json;

use crate::packages::prelude::*;

/// Parse manifest v2 file from given JSON object
pub fn parse_v2(manifest: &Json) -> anyhow::Result<Manifest> {
    let Some(inputs) = manifest.get("inputs").and_then(Json::as_object) else {
        anyhow::bail!("Incorrect manifest v2 file format: `inputs` field is missing")
    };

    let Some(outputs) = manifest.get("outputs").and_then(Json::as_array) else {
        anyhow::bail!("Incorrect manifest v2 file format: `outputs` field is missing")
    };

    let metadata = manifest.get("metadata");

    Ok(Manifest {
        manifest_version: 2,

        metadata: ManifestMetadata {
            homepage: metadata.and_then(|metadata| {
                    metadata.get("homepage")
                        .and_then(Json::as_str)
                        .map(String::from)
                }),

            maintainers: metadata.and_then(|metadata| {
                    metadata.get("maintainers")
                        .and_then(Json::as_array)
                        .map(|maintainers| {
                            maintainers.iter()
                                .filter_map(|maintainer| maintainer.as_str())
                                .map(String::from)
                                .collect::<Vec<_>>()
                        })
                })
        },

        inputs: inputs.iter()
            .map(|(name, input)| {
                let uri = input.get("uri")
                    .and_then(Json::as_str)
                    .map(String::from)
                    .ok_or_else(|| anyhow::anyhow!("Incorrect manifest v2 file format: `inputs[].uri` field is missing"))?;

                let input = ManifestInput {
                    format: input.get("format")
                        .and_then(Json::as_str)
                        .and_then(ManifestInputFormat::from_str)
                        .unwrap_or_else(|| ManifestInputFormat::from_uri(&uri)),

                    hash: input.get("hash")
                        .and_then(Json::as_str)
                        .ok_or_else(|| anyhow::anyhow!("Incorrect manifest v2 file format: `inputs[].hash` field is missing"))
                        .and_then(Hash::try_from)?,

                    uri
                };

                Ok((name.clone(), input))
            })
            .collect::<anyhow::Result<HashMap<_, _>>>()?,

        outputs: outputs.iter()
            .map(|output| {
                Ok(ManifestOutput {
                    format: output.get("format")
                        .and_then(Json::as_str)
                        .and_then(ManifestOutputFormat::from_str)
                        .unwrap_or_default(),

                    path: output.get("path")
                        .and_then(Json::as_str)
                        .map(String::from)
                        .ok_or_else(|| anyhow::anyhow!("Incorrect manifest v2 file format: `outputs[].path` field is missing"))?,

                    hash: output.get("hash")
                        .and_then(Json::as_str)
                        .ok_or_else(|| anyhow::anyhow!("Incorrect manifest v2 file format: `outputs[].hash` field is missing"))
                        .and_then(Hash::try_from)?,

                    metadata: output.get("metadata")
                        .ok_or_else(|| anyhow::anyhow!("Incorrect manifest v2 file format: `outputs[].metadata` field is missing"))
                        .and_then(|metadata| {
                            let name = metadata.get("name")
                                .and_then(Json::as_str)
                                .map(String::from)
                                .ok_or_else(|| anyhow::anyhow!("Incorrect manifest v2 file format: `outputs[].metadata.name` field is missing"))?;

                            Ok(ManifestOutputMetadata {
                                uuid: metadata.get("uuid")
                                    .and_then(Json::as_str)
                                    .map(Uuid::try_from)
                                    .unwrap_or_else(|| Ok(Uuid::new_from_str(&name)))?,

                                title: metadata.get("title")
                                    .and_then(Json::as_str)
                                    .map(String::from)
                                    .unwrap_or_else(|| name.clone()),

                                standard: metadata.get("standard")
                                    .and_then(Json::as_u64)
                                    .unwrap_or(2),

                                name
                            })
                        })?
                })
            })
            .collect::<anyhow::Result<Vec<_>>>()?
    })
}
