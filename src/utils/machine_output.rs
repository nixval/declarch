use crate::error::Result;
use chrono::Utc;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct MachineEnvelope<T>
where
    T: Serialize,
{
    pub version: String,
    pub command: String,
    pub ok: bool,
    pub data: T,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub meta: MachineMeta,
}

#[derive(Debug, Serialize)]
pub struct MachineMeta {
    pub generated_at: String,
}

pub fn emit_v1<T>(
    command: &str,
    data: T,
    warnings: Vec<String>,
    errors: Vec<String>,
    format: &str,
) -> Result<()>
where
    T: Serialize,
{
    let envelope = MachineEnvelope {
        version: "v1".to_string(),
        command: command.to_string(),
        ok: errors.is_empty(),
        data,
        warnings,
        errors,
        meta: MachineMeta {
            generated_at: Utc::now().to_rfc3339(),
        },
    };

    match format {
        "json" => {
            let out = serde_json::to_string_pretty(&envelope)?;
            println!("{}", out);
        }
        "yaml" => {
            let out = serde_yml::to_string(&envelope)?;
            println!("{}", out);
        }
        _ => {}
    }

    Ok(())
}

#[cfg(test)]
mod tests;
