mod aes_util;
mod http_util;

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use dbx_plugin_sdk::{
    PluginEmitter, PluginError, PluginHandler, PluginMetadata, PluginServer, RequestContext,
};
use serde_json::{json, Value};

#[derive(Default)]
struct Plugin {
    connections: Mutex<HashSet<String>>,
    aes_keys: Mutex<HashMap<String, String>>,
    aes_key_labels: Mutex<HashMap<String, String>>,
    aes_key_interfaces: Mutex<HashMap<String, String>>,
}
impl PluginHandler for Plugin {
    fn handle(
        &self,
        _context: RequestContext,
        method: &str,
        params: Value,
        _emitter: &PluginEmitter,
    ) -> Result<Value, PluginError> {
        match method {
            "connection/test" => {
                let connection = params.get("connection").cloned().unwrap_or_default();
                Ok(json!({
                    "success": true,
                    "message": format!(
                        "aes-tools is ready for {}:{}.",
                        connection.get("host").and_then(Value::as_str).unwrap_or("localhost"),
                        connection.get("port").and_then(Value::as_u64).unwrap_or(0)
                    )
                }))
            }
            "connection/connect" => {
                let connection_id = connection_id(&params)?;
                let connection = params.get("connection").cloned().unwrap_or_default();
                let aes_key = optional_string(
                    connection.get("external_config").and_then(Value::as_object),
                    "aes_key",
                );
                let aes_key_label = optional_string(
                    connection.get("external_config").and_then(Value::as_object),
                    "aes_key_label",
                );
                let interface_path = optional_string(
                    connection.get("external_config").and_then(Value::as_object),
                    "interface_path",
                );
                let protocol = optional_string(
                    connection.get("external_config").and_then(Value::as_object),
                    "protocol",
                );
                let host = connection
                    .get("host")
                    .and_then(Value::as_str)
                    .unwrap_or("localhost");
                let port = connection.get("port").and_then(Value::as_u64).unwrap_or(0);
                let path = format!("{}://{}:{}{}", protocol, host, port, interface_path);
                self.aes_keys
                    .lock()
                    .unwrap() // 拿到 MutexGuard
                    .insert(connection_id.to_string(), aes_key.to_string());
                self.aes_key_interfaces
                    .lock()
                    .unwrap() // 拿到 MutexGuard
                    .insert(connection_id.to_string(), path.to_string());
                self.aes_key_labels
                    .lock()
                    .unwrap() // 拿到 MutexGuard
                    .insert(connection_id.to_string(), aes_key_label.to_string());
                self.connections
                    .lock()
                    .map_err(|_| PluginError::new(-32000, "Connection registry is poisoned"))?
                    .insert(connection_id.to_string());

                Ok(json!({ "success": true,"aesKey": aes_key }))
            }
            "connection/disconnect" => {
                let connection_id = connection_id(&params)?;
                self.connections
                    .lock()
                    .map_err(|_| PluginError::new(-32000, "Connection registry is poisoned"))?
                    .remove(connection_id);
                self.aes_keys
                    .lock()
                    .unwrap() // 拿到 MutexGuard
                    .remove(connection_id);
                self.aes_key_labels
                    .lock()
                    .unwrap() // 拿到 MutexGuard
                    .remove(connection_id);
                self.aes_key_interfaces
                    .lock()
                    .unwrap() // 拿到 MutexGuard
                    .remove(connection_id);
                Ok(json!({ "success": true }))
            }
            "aes-rust-plugin/ping" => Ok(json!({
                "ok": true,
                "plugin": "aes-tools",
                "language": "rust",
                "connectionId": params.get("connectionId").cloned().unwrap_or(Value::Null)
            })),
            "aes-rust-plugin/aesEncrypt" => {
                let connection_id = params
                    .get("connectionId")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                let aes_key = self.aes_keys.lock().unwrap().get(connection_id).cloned();
                let key = params
                    .get("key")
                    .and_then(Value::as_str)
                    .unwrap_or("1234567890123456"); // 16 字节
                let plaintext = params
                    .get("plaintext")
                    .and_then(Value::as_str)
                    .unwrap_or(""); // 16 字节
                let aes_content = match aes_util::aes_encrypt(plaintext, key) {
                    Ok(v) => v,
                    Err(e) => return Ok(json!({ "message": e.to_string(), "code": 500 })),
                };
                Ok(json!({
                    "code":0,
                    "data": aes_content,
                    "aes_key":aes_key,
                }))
            }
            "aes-rust-plugin/aesKey" => {
                let connection_id = params
                    .get("connectionId")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                let aes_key = self.aes_keys.lock().unwrap().get(connection_id).cloned();
                Ok(json!({
                    "code":0,
                    "data": aes_key
                }))
            }
            "aes-rust-plugin/aesKeyLables" => {
                let connection_id = params
                    .get("connectionId")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                let aes_key_label = self
                    .aes_key_labels
                    .lock()
                    .unwrap()
                    .get(connection_id)
                    .cloned();
                let aes_key_interface = self
                    .aes_key_interfaces
                    .lock()
                    .unwrap()
                    .get(connection_id)
                    .cloned();
                match http_util::HttpUtil::get_with_timeout(&aes_key_interface.unwrap().as_str(), 3)
                {
                    Ok(body) => {
                        if is_json_array(&body) {
                            Ok(json!({
                                "code":0,
                                "data": body
                            }))
                        } else {
                            Ok(json!({
                                "code":0,
                                "data": aes_key_label
                            }))
                        }
                    }
                    #[warn(unused_variables)]
                    Err(_e) => Ok(json!({
                        "code":0,
                        "data": aes_key_label
                    })),
                }
            }
            "aes-rust-plugin/aesDecrypt" => {
                let key = params
                    .get("key")
                    .and_then(Value::as_str)
                    .unwrap_or("1234567890123456"); // 16 字节
                let plaintext = params
                    .get("plaintext")
                    .and_then(Value::as_str)
                    .unwrap_or(""); // 16 字节
                let aes_content = match aes_util::aes_decrypt(plaintext, key) {
                    Ok(v) => v,
                    Err(e) => return Ok(json!({ "message": e.to_string(), "code": 500 })),
                };
                Ok(json!({
                    "code":0,
                    "data": aes_content
                }))
            }
            _ => Err(PluginError::method_not_found(method)),
        }
    }
}
/// 判断字符串是否是合法 JSON
pub fn is_json(s: &str) -> bool {
    serde_json::from_str::<Value>(s).is_ok()
}
pub fn is_json_array(s: &str) -> bool {
    matches!(serde_json::from_str::<Value>(s), Ok(Value::Array(_)))
}
fn connection_id(params: &Value) -> Result<&str, PluginError> {
    params
        .get("connection")
        .and_then(|connection| connection.get("id"))
        .and_then(Value::as_str)
        .ok_or_else(|| PluginError::new(-32602, "Missing connection id"))
}
fn optional_string(object: Option<&serde_json::Map<String, Value>>, key: &str) -> String {
    object
        .and_then(|object| object.get(key))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_string()
}
fn main() -> std::io::Result<()> {
    let metadata =
        PluginMetadata::new("aes-tools", env!("CARGO_PKG_VERSION")).with_capability("connections");
    PluginServer::new(metadata, Plugin::default()).serve()
}
