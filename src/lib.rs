use std::collections::HashMap;
use std::env;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::Write;

use base64::prelude::*;
use flate2::read::ZlibDecoder;
use serde::{de, Deserialize, Serialize};
use serde_json::json;

// translation of
// https://github.com/dagster-io/dagster/blob/258d9ca0db/python_modules/dagster-pipes/dagster_pipes/__init__.py#L354-L367
fn decode_env_var<T>(param: &str) -> T
where
    T: de::DeserializeOwned,
{
    let zlib_compressed_slice = BASE64_STANDARD.decode(param).unwrap();
    let mut decoder = ZlibDecoder::new(&zlib_compressed_slice[..]);
    let mut json_str = String::new();
    decoder.read_to_string(&mut json_str).unwrap();
    serde_json::from_str(&json_str).unwrap()
}

// partial translation of
// https://github.com/dagster-io/dagster/blob/258d9ca0db/python_modules/dagster-pipes/dagster_pipes/__init__.py#L94-L108
#[derive(Debug, Deserialize)]
struct PipesContextData {
    asset_keys: Option<Vec<String>>,
    run_id: String,
    extras: HashMap<String, serde_json::Value>,
}

// translation of
// https://github.com/dagster-io/dagster/blob/258d9ca0db/python_modules/dagster-pipes/dagster_pipes/__init__.py#L83-L88
#[derive(Debug, Serialize)]
struct PipesMessage {
    __dagster_pipes_version: String,
    method: String,
    params: Option<HashMap<String, serde_json::Value>>,
}

// partial translation of
// https://github.com/dagster-io/dagster/blob/258d9ca0db/python_modules/dagster-pipes/dagster_pipes/__init__.py#L859-L871
#[derive(Debug)]
pub struct PipesContext {
    data: PipesContextData,
    writer: PipesFileMessageWriter,
}

impl PipesContext {
    pub fn report_asset_materialization(&mut self, asset_key: &str, metadata: serde_json::Value) {
        let params: HashMap<String, serde_json::Value> = HashMap::from([
            ("asset_key".to_string(), json!(asset_key)),
            ("metadata".to_string(), metadata),
            ("data_version".to_string(), json!(null)), // TODO - support data versions
        ]);

        let msg = PipesMessage {
            __dagster_pipes_version: "0.1".to_string(),
            method: "report_asset_materialization".to_string(),
            params: Some(params),
        };
        self.writer.write_message(msg);
    }

    pub fn report_asset_check(
        &mut self,
        check_name: &str,
        passed: bool,
        asset_key: &str,
        metadata: serde_json::Value,
    ) {
        let params: HashMap<String, serde_json::Value> = HashMap::from([
            ("asset_key".to_string(), json!(asset_key)),
            ("check_name".to_string(), json!(check_name)),
            ("passed".to_string(), json!(passed)),
            ("severity".to_string(), json!("ERROR")), // hardcode for now
            ("metadata".to_string(), metadata),
        ]);

        let msg = PipesMessage {
            __dagster_pipes_version: "0.1".to_string(),
            method: "report_asset_check".to_string(),
            params: Some(params),
        };
        self.writer.write_message(msg);
    }
}

#[derive(Debug)]
struct PipesFileMessageWriter {
    path: String,
}
impl PipesFileMessageWriter {
    fn write_message(&mut self, message: PipesMessage) {
        let serialized_msg = serde_json::to_string(&message).unwrap();
        let mut file = OpenOptions::new().append(true).open(&self.path).unwrap();
        writeln!(file, "{}", serialized_msg).unwrap();

        // TODO - optional `stderr` based writing
        //eprintln!("{}", serialized_msg);
    }
}

#[derive(Debug, Deserialize)]
struct PipesContextParams {
    data: Option<PipesContextData>, // direct in env var
    path: Option<String>,           // load from file (unsupported)
}

#[derive(Debug, Deserialize)]
struct PipesMessagesParams {
    path: Option<String>,  // write to file
    stdio: Option<String>, // stderr | stdout (unsupported)
}

// partial translation of
// https://github.com/dagster-io/dagster/blob/258d9ca0db/python_modules/dagster-pipes/dagster_pipes/__init__.py#L798-L838
pub fn open_dagster_pipes() -> PipesContext {
    // approximation of PipesEnvVarParamsLoader
    let context_env_var = env::var("DAGSTER_PIPES_CONTEXT").unwrap();
    let context_params: PipesContextParams = decode_env_var(&context_env_var);
    let context_data = context_params
        .data
        .expect("Unable to load dagster pipes context, only direct env var data supported.");

    let msg_env_var = env::var("DAGSTER_PIPES_MESSAGES").unwrap();
    let messages_params: PipesMessagesParams = decode_env_var(&msg_env_var);
    let path = messages_params.path.expect(
        "Unable to write Dagster messages, only temporary file message writing is supported.",
    );

    //if stdio != "stderr" {
    //    panic!("only stderr supported for dagster pipes messages")
    //}

    PipesContext {
        data: context_data,
        writer: PipesFileMessageWriter { path },
    }
}
