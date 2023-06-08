use std::fmt::{Display, Formatter};
use anyhow::{bail, Result};
use serde::{Serialize, Deserialize};

use move_core_types::language_storage::{ModuleId, StructTag};
use move_core_types::account_address::AccountAddress;

use lang::ss58::address_to_ss58;
use crate::{Net, BytesForBlock};
mod abi;
mod address;
mod bytecode;
pub mod move_types;
mod wrappers;
#[cfg(test)]
use abi::ModuleAbi;
use move_types::MoveModuleBytecode;
pub type Block = String;

pub struct PontNet {
    pub(crate) api: String,
}

impl Net for PontNet {
    fn get_module(
        &self,
        module_id: &ModuleId,
        height: &Option<Block>,
    ) -> Result<Option<BytesForBlock>> {
        let req = Request {
            id: 1,
            jsonrpc: "2.0",
            method: "mvm_getModule",
            params: vec![format!("0x{}", hex::encode(bcs::to_bytes(module_id)?))],
        };

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Content-Type",
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        let response = reqwest::blocking::Client::new()
            .post(&self.api)
            .headers(headers)
            .json(&req)
            .send()?;

        if response.status() != 200 {
            bail!(
                "Failed to get module :{}. Error:{}",
                module_id,
                response.status()
            );
        }
        let resp = response.json::<Response>()?;
        if let Some(err) = resp.error {
            bail!("{:?}", err);
        }
        if let Some(result) = resp.result {
            let result = hex::decode(&result[2..])?;
            Ok(Some(BytesForBlock(
                result,
                height.clone().unwrap_or_default(),
            )))
        } else {
            Ok(None)
        }
    }

    fn get_resource(
        &self,
        address: &AccountAddress,
        tag: &StructTag,
        height: &Option<Block>,
    ) -> Result<Option<BytesForBlock>> {
        let req = Request {
            id: 1,
            jsonrpc: "2.0",
            method: "mvm_getResource",
            params: vec![
                address_to_ss58(address),
                format!("0x{}", hex::encode(bcs::to_bytes(&tag)?)),
            ],
        };
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Content-Type",
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        let response = reqwest::blocking::Client::new()
            .post(&self.api)
            .headers(headers)
            .json(&req)
            .send()?;
        if response.status() != 200 {
            bail!(
                "Failed to get resource :{:?} {:?}. Error:{}",
                &address,
                &tag,
                response.status()
            );
        }

        let resp = response.json::<Response>()?;
        if let Some(err) = resp.error {
            bail!("{:?}", err);
        }
        if let Some(result) = resp.result {
            let result = hex::decode(&result[2..])?;
            Ok(Some(BytesForBlock(
                result,
                height.clone().unwrap_or_default(),
            )))
        } else {
            Ok(None)
        }
    }
    // fn get_resources(
    //     &self,
    //     address: &AccountAddress,
    //     tag: &StructTag,
    //     height: &Option<Block>,
    // ) -> Result<Option<BytesForBlock>> {
    //     let req = Request {
    //         id: 1,
    //         jsonrpc: "2.0",
    //         method: "mvm_getResources",
    //         params: vec![
    //             address_to_ss58(address),
    //             format!("0x{}", hex::encode(bcs::to_bytes(&tag)?)),
    //         ],
    //     };
    //     let mut headers = reqwest::header::HeaderMap::new();
    //     headers.insert(
    //         "Content-Type",
    //         reqwest::header::HeaderValue::from_static("application/json"),
    //     );
    //     let response = reqwest::blocking::Client::new()
    //         .post(&self.api)
    //         .headers(headers)
    //         .json(&req)
    //         .send()?;
    //     if response.status() != 200 {
    //         bail!(
    //             "Failed to get resource :{:?} {:?}. Error:{}",
    //             &address,
    //             &tag,
    //             response.status()
    //         );
    //     }

    //     let resp = response.json::<Response>()?;
    //     if let Some(err) = resp.error {
    //         bail!("{:?}", err);
    //     }
    //     if let Some(result) = resp.result {
    //         let result = hex::decode(&result[2..])?;
    //         Ok(Some(BytesForBlock(
    //             result,
    //             height.clone().unwrap_or_default(),
    //         )))
    //     } else {
    //         Ok(None)
    //     }
    // }
    // fn get_resources2(
    //     &self,
    //     address: &AccountAddress,
    //     tag: &StructTag,
    //     height: &Option<Block>,
    // ) -> Result<Option<BytesForBlock>> {
    //     let req = Request {
    //         id: 1,
    //         jsonrpc: "2.0",
    //         method: "mvm_getResources2",
    //         params: vec![
    //             address_to_ss58(address),
    //             format!("0x{}", hex::encode(bcs::to_bytes(&tag)?)),
    //         ],
    //     };
    //     let mut headers = reqwest::header::HeaderMap::new();
    //     headers.insert(
    //         "Content-Type",
    //         reqwest::header::HeaderValue::from_static("application/json"),
    //     );
    //     let response = reqwest::blocking::Client::new()
    //         .post(&self.api)
    //         .headers(headers)
    //         .json(&req)
    //         .send()?;
    //     if response.status() != 200 {
    //         bail!(
    //             "Failed to get resource :{:?} {:?}. Error:{}",
    //             &address,
    //             &tag,
    //             response.status()
    //         );
    //     }

    //     let resp = response.json::<Response>()?;
    //     if let Some(err) = resp.error {
    //         bail!("{:?}", err);
    //     }
    //     if let Some(result) = resp.result {
    //         println!(
    //             "result=={:?}==={:?}",
    //             &result,
    //             std::str::from_utf8(&hex::decode(&result[2..]).unwrap()).unwrap()
    //         );

    //         let result = hex::decode(&result[2..])?;
    //         Ok(Some(BytesForBlock(
    //             result,
    //             height.clone().unwrap_or_default(),
    //         )))
    //     } else {
    //         Ok(None)
    //     }
    // }
    fn get_resources(
        &self,
        address: &AccountAddress,
        tag: &str,
        height: &Option<Block>,
    ) -> Result<Option<BytesForBlock>> {
        let req = Request {
            id: 1,
            jsonrpc: "2.0",
            method: "mvm_getResources",
            params: vec![
                address_to_ss58(address),
                format!("0x{}", hex::encode(tag.as_bytes())),
            ],
        };
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Content-Type",
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        let response = reqwest::blocking::Client::new()
            .post(&self.api)
            .headers(headers)
            .json(&req)
            .send()?;
        if response.status() != 200 {
            bail!(
                "Failed to get resource :{:?} {:?}. Error:{}",
                &address,
                &tag,
                response.status()
            );
        }

        let resp = response.json::<Response>()?;
        if let Some(err) = resp.error {
            bail!("{:?}", err);
        }
        if let Some(result) = resp.result {
            let result = hex::decode(&result[2..])?;
            Ok(Some(BytesForBlock(
                result,
                height.clone().unwrap_or_default(),
            )))
        } else {
            Ok(None)
        }
    }
    fn get_table_entry(
        &self,
        address: &AccountAddress,
        handle: &str,
        key: &str,
        key_type: &str,
        value_type: &str,
        height: &Option<Block>,
    ) -> Result<Option<BytesForBlock>> {
        let req = Request {
            id: 1,
            jsonrpc: "2.0",
            method: "mvm_getTableEntry",
            params: vec![
                // address_to_ss58(address),
                format!("0x{}", hex::encode(handle.as_bytes())),
                format!("0x{}", hex::encode(key.as_bytes())),
                format!("0x{}", hex::encode(key_type.as_bytes())),
                format!("0x{}", hex::encode(value_type.as_bytes())),
            ],
        };
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Content-Type",
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        let response = reqwest::blocking::Client::new()
            .post(&self.api)
            .headers(headers)
            .json(&req)
            .send()?;
        if response.status() != 200 {
            bail!(
                "Failed to get table entry :{:?} .{:?}.{:?}.{:?}.{:?}. Error:{}",
                &address,
                &handle,
                &key,
                &key_type,
                &value_type,
                response.status()
            );
        }

        let resp = response.json::<Response>()?;
        if let Some(err) = resp.error {
            bail!("{:?}", err);
        }
        if let Some(result) = resp.result {
            let result = hex::decode(&result[2..])?;
            Ok(Some(BytesForBlock(
                result,
                height.clone().unwrap_or_default(),
            )))
        } else {
            Ok(None)
        }
    }
    fn get_module_abi(
        &self,
        module_id: &ModuleId,
        height: &Option<Block>,
    ) -> Result<Option<BytesForBlock>> {
        let req = Request {
            id: 1,
            jsonrpc: "2.0",
            method: "mvm_getModuleABI",
            params: vec![format!("0x{}", hex::encode(bcs::to_bytes(module_id)?))],
        };
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Content-Type",
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        let response = reqwest::blocking::Client::new()
            .post(&self.api)
            .headers(headers)
            .json(&req)
            .send()?;

        if response.status() != 200 {
            bail!(
                "Failed to get module abis:{}. Error:{}",
                module_id,
                response.status()
            );
        }
        let resp = response.json::<Response>()?;
        if let Some(err) = resp.error {
            bail!("{:?}", err);
        }
        if let Some(result) = resp.result {
            let result = hex::decode(&result[2..])?;
            Ok(Some(BytesForBlock(
                result.into(),
                height.clone().unwrap_or_default(),
            )))
        } else {
            Ok(None)
        }
    }
    fn get_module_abis(
        &self,
        module_id: &ModuleId,
        height: &Option<Block>,
    ) -> Result<Option<BytesForBlock>> {
        let req = Request {
            id: 1,
            jsonrpc: "2.0",
            method: "mvm_getModuleABIs",
            params: vec![format!("0x{}", hex::encode(bcs::to_bytes(module_id)?))],
        };
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Content-Type",
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        let response = reqwest::blocking::Client::new()
            .post(&self.api)
            .headers(headers)
            .json(&req)
            .send()?;

        if response.status() != 200 {
            bail!(
                "Failed to get module abis:{}. Error:{}",
                module_id,
                response.status()
            );
        }
        let resp = response.json::<Response>()?;
        if let Some(err) = resp.error {
            bail!("{:?}", err);
        }
        if let Some(result) = resp.result {
            let s = std::str::from_utf8(&hex::decode(&result[2..]).unwrap())
                .unwrap()
                .to_string();
            let _res1: MoveModuleBytecode = serde_json::from_str(&s).unwrap();
            let _res: MoveModuleBytecode =
                serde_json::from_slice(&hex::decode(&result[2..]).unwrap()).unwrap();
            Ok(Some(BytesForBlock(
                result.into(),
                height.clone().unwrap_or_default(),
            )))
        } else {
            Ok(None)
        }
    }
    fn encode_submission(
        &self,
        addr: &str,
        module: &str,
        function: &str,
        arguments: &[&str],
        type_parameters: &[&str],
        height: &Option<Block>,
    ) -> Result<Option<BytesForBlock>> {
        let req = Requests {
            id: 1,
            jsonrpc: "2.0",
            method: "mvm_encodeSubmission",
            params: vec![
                vec![
                    format!("0x{}", hex::encode(addr.as_bytes())),
                    format!("0x{}", hex::encode(module.as_bytes())),
                    format!("0x{}", hex::encode(function.as_bytes())),
                ],
                arguments
                    .iter()
                    .map(|a| format!("0x{}", hex::encode(a.as_bytes())))
                    .collect(),
                type_parameters
                    .iter()
                    .map(|a| format!("0x{}", hex::encode(a.as_bytes())))
                    .collect(),
            ],
        };
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Content-Type",
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        let response = reqwest::blocking::Client::new()
            .post(&self.api)
            .headers(headers)
            .json(&req)
            .send()?;
        if response.status() != 200 {
            bail!(
                "Failed to get resource :{:?} {:?}.{:?} {:?}.{:?} . Error:{}",
                addr,
                module,
                function,
                arguments,
                type_parameters,
                response.status()
            );
        }

        let resp = response.json::<Response>()?;
        if let Some(err) = resp.error {
            bail!("{:?}", err);
        }
        if let Some(result) = resp.result {
            let result = hex::decode(&result[2..])?;
            Ok(Some(BytesForBlock(
                result,
                height.clone().unwrap_or_default(),
            )))
        } else {
            Ok(None)
        }
    }
}

#[derive(Serialize)]
struct Request {
    id: u64,
    jsonrpc: &'static str,
    method: &'static str,
    params: Vec<String>,
}
#[derive(Serialize)]
struct Requests {
    id: u64,
    jsonrpc: &'static str,
    method: &'static str,
    params: Vec<Vec<String>>,
}
#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct Response {
    id: u64,
    jsonrpc: String,
    result: Option<String>,
    error: Option<ErrorMsg>,
}
#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct Responses {
    id: u64,
    jsonrpc: String,
    result: Option<MoveModuleBytecode>,
    error: Option<ErrorMsg>,
}
#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct ErrorMsg {
    code: Option<i64>,
    message: Option<String>,
}

impl Display for ErrorMsg {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(msg) = self.message.as_ref() {
            write!(f, "{}; Code:{}", msg, self.code.unwrap_or_default())
        } else {
            write!(f, "Code:{}", self.code.unwrap_or_default())
        }
    }
}

#[cfg(test)]
mod tests {
    use move_core_types::account_address::AccountAddress;
    use move_core_types::identifier::Identifier;
    use move_core_types::language_storage::{ModuleId, StructTag};

    use lang::ss58::ss58_to_address;

    use super::*;
    use crate::Net;

    /// If the node is raised to "localhost:9933".
    #[ignore]
    #[test]
    fn test_get_module() {
        let api = PontNet {
            api: "http://localhost:9933".to_string(),
        };
        let module = api
            .get_module(
                &ModuleId::new(
                    AccountAddress::from_hex_literal("0x1").unwrap(),
                    Identifier::new("Hash").unwrap(),
                ),
                &None,
            )
            .unwrap()
            .unwrap();
        assert_eq!(
            [
                161, 28, 235, 11, 2, 0, 0, 0, 6, 1, 0, 2, 3, 2, 10, 5, 12, 3, 7, 15, 23, 8, 38,
                32, 12, 70, 8, 0, 0, 0, 1, 0, 0, 0, 0, 2, 0, 0, 0, 1, 10, 2, 4, 72, 97, 115, 104,
                8, 115, 104, 97, 50, 95, 50, 53, 54, 8, 115, 104, 97, 51, 95, 50, 53, 54, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 1, 0, 1, 2, 0, 1, 1, 2, 0, 0
            ],
            module.0.as_slice()
        );
    }

    #[test]
    fn test_get_module_accound_id() {
        let api = PontNet {
            api: "http://localhost:9933".to_string(),
        };
        let module = api
            .get_module(
                &ModuleId::new(
                    AccountAddress::from_hex_literal(
                        "0xD43593C715FDD31C61141ABD04A99FD6822C8558854CCDE39A5684E7A56DA27D",
                    )
                    .unwrap(),
                    Identifier::new("ScriptBook").unwrap(),
                ),
                &None,
            )
            .unwrap()
            .unwrap();

        assert_eq!(
            [
                161, 28, 235, 11, 2, 0, 0, 0, 6, 1, 0, 2, 3, 2, 10, 5, 12, 3, 7, 15, 23, 8, 38,
                32, 12, 70, 8, 0, 0, 0, 1, 0, 0, 0, 0, 2, 0, 0, 0, 1, 10, 2, 4, 72, 97, 115, 104,
                8, 115, 104, 97, 50, 95, 50, 53, 54, 8, 115, 104, 97, 51, 95, 50, 53, 54, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 1, 0, 1, 2, 0, 1, 1, 2, 0, 0
            ],
            module.0.as_slice()
        );
    }
    /// If the node is raised to "localhost:9933"
    ///     and there is a resource on "5grwvaef5zxb26fz9rcqpdws57cterhpnehxcpcnohgkutqy::Store::U64".
    // #[ignore]
    #[test]
    fn test_get_resource_sum() {
        let api = PontNet {
            api: "http://localhost:9933".to_string(),
        };

        let addr = ss58_to_address("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
        let module = api
            .get_resource(
                &addr,
                &StructTag {
                    address: addr,
                    module: Identifier::new("Storage").unwrap(),
                    name: Identifier::new("Sum").unwrap(),
                    type_params: vec![],
                },
                &None,
            )
            .unwrap()
            .unwrap();
        assert_eq!(module.0, [12, 0, 0, 0, 0, 0, 0, 0]);
    }
    //     // #[ignore]
    //     #[test]
    //     fn test_get_resources1() {
    //         let api = PontNet {
    //             api: "http://localhost:9933".to_string(),
    //         };

    //         let addr = ss58_to_address("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
    //         let module = api
    //             .get_resources(
    //                 &addr,
    //                 &StructTag {
    //                     address: addr,
    //                     module: Identifier::new("Storage").unwrap(),
    //                     name: Identifier::new("Sum").unwrap(),
    //                     type_params: vec![],
    //                 },
    //                 &None,
    //             )
    //             .unwrap()
    //             .unwrap();
    //         assert_eq!(module.0, [100, 0, 0, 0, 0, 0, 0, 0]);
    //     }
    //  // #[ignore]
    //     #[test]
    //     fn test_get_resources_todo1() {
    //         let api = PontNet {
    //             api: "http://localhost:9933".to_string(),
    //         };

    //         let addr = ss58_to_address("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
    //         let module = api
    //             .get_resources(
    //                 &addr,
    //                 &StructTag {
    //                     address: addr,
    //                     module: Identifier::new("TodoList").unwrap(),
    //                     name: Identifier::new("TodoList").unwrap(),
    //                     type_params: vec![],
    //                 },
    //                 &None,
    //             )
    //             .unwrap()
    //             .unwrap();
    //         assert_eq!(module.0, [100, 0, 0, 0, 0, 0, 0, 0]);
    //     }
    // // #[ignore]
    // #[test]
    // fn test_get_resources2() {
    //     let api = PontNet {
    //         api: "http://localhost:9933".to_string(),
    //     };

    //     let addr = ss58_to_address("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
    //     let module = api
    //         .get_resources2(
    //             &addr,
    //             &StructTag {
    //                 address: addr,
    //                 module: Identifier::new("Storage").unwrap(),
    //                 name: Identifier::new("Sum").unwrap(),
    //                 type_params: vec![],
    //             },
    //             &None,
    //         )
    //         .unwrap()
    //         .unwrap();
    //     assert_eq!(module.0, [100, 0, 0, 0, 0, 0, 0, 0]);
    // }
    // #[ignore]
    // #[test]
    // fn test_get_resources_todo2() {
    //     let api = PontNet {
    //         api: "http://localhost:9933".to_string(),
    //     };

    //     let addr = ss58_to_address("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
    //     let module = api
    //         .get_resources2(
    //             &addr,
    //             &StructTag {
    //                 address: addr,
    //                 module: Identifier::new("TodoList").unwrap(),
    //                 name: Identifier::new("TodoList").unwrap(),
    //                 type_params: vec![],
    //             },
    //             &None,
    //         )
    //         .unwrap()
    //         .unwrap();
    //     assert_eq!(module.0, [100, 0, 0, 0, 0, 0, 0, 0]);
    // }
    // #[ignore]
    #[test]
    fn test_get_resources_sum() {
        let api = PontNet {
            api: "http://localhost:9933".to_string(),
        };

        let addr = ss58_to_address("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
        let module = api
            .get_resources(
                &addr,
"0xD43593C715FDD31C61141ABD04A99FD6822C8558854CCDE39A5684E7A56DA27D::Storage::Sum",
                &None,
            )
            .unwrap()
            .unwrap();
        assert_eq!(
            module.0,
            [
                123, 34, 116, 121, 112, 101, 34, 58, 34, 68, 52, 51, 53, 57, 51, 67, 55, 49, 53,
                70, 68, 68, 51, 49, 67, 54, 49, 49, 52, 49, 65, 66, 68, 48, 52, 65, 57, 57, 70,
                68, 54, 56, 50, 50, 67, 56, 53, 53, 56, 56, 53, 52, 67, 67, 68, 69, 51, 57, 65,
                53, 54, 56, 52, 69, 55, 65, 53, 54, 68, 65, 50, 55, 68, 58, 58, 83, 116, 111,
                114, 97, 103, 101, 58, 58, 83, 117, 109, 34, 44, 34, 100, 97, 116, 97, 34, 58,
                123, 34, 118, 97, 108, 34, 58, 49, 50, 125, 125
            ]
        );
    }
    // #[ignore]
    #[test]
    fn test_get_resources_todo() {
        let api = PontNet {
            api: "http://localhost:9933".to_string(),
        };

        let addr = ss58_to_address("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
        let module = api
            .get_resources(
                &addr,
"0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d::TodoList::TodoList",
                &None,
            )
            .unwrap()
            .unwrap();
        assert_eq!(
            module.0,
            [
                123, 34, 116, 121, 112, 101, 34, 58, 34, 68, 52, 51, 53, 57, 51, 67, 55, 49, 53,
                70, 68, 68, 51, 49, 67, 54, 49, 49, 52, 49, 65, 66, 68, 48, 52, 65, 57, 57, 70,
                68, 54, 56, 50, 50, 67, 56, 53, 53, 56, 56, 53, 52, 67, 67, 68, 69, 51, 57, 65,
                53, 54, 56, 52, 69, 55, 65, 53, 54, 68, 65, 50, 55, 68, 58, 58, 84, 111, 100,
                111, 76, 105, 115, 116, 58, 58, 84, 111, 100, 111, 76, 105, 115, 116, 34, 44, 34,
                100, 97, 116, 97, 34, 58, 123, 34, 115, 101, 116, 95, 116, 97, 115, 107, 95, 101,
                118, 101, 110, 116, 34, 58, 123, 34, 99, 111, 117, 110, 116, 101, 114, 34, 58,
                49, 44, 34, 103, 117, 105, 100, 34, 58, 123, 34, 103, 117, 105, 100, 34, 58, 123,
                34, 105, 100, 34, 58, 123, 34, 97, 100, 100, 114, 34, 58, 91, 50, 49, 50, 44, 53,
                51, 44, 49, 52, 55, 44, 49, 57, 57, 44, 50, 49, 44, 50, 53, 51, 44, 50, 49, 49,
                44, 50, 56, 44, 57, 55, 44, 50, 48, 44, 50, 54, 44, 49, 56, 57, 44, 52, 44, 49,
                54, 57, 44, 49, 53, 57, 44, 50, 49, 52, 44, 49, 51, 48, 44, 52, 52, 44, 49, 51,
                51, 44, 56, 56, 44, 49, 51, 51, 44, 55, 54, 44, 50, 48, 53, 44, 50, 50, 55, 44,
                49, 53, 52, 44, 56, 54, 44, 49, 51, 50, 44, 50, 51, 49, 44, 49, 54, 53, 44, 49,
                48, 57, 44, 49, 54, 50, 44, 49, 50, 53, 93, 44, 34, 99, 114, 101, 97, 116, 105,
                111, 110, 95, 110, 117, 109, 34, 58, 48, 125, 125, 44, 34, 108, 101, 110, 95, 98,
                121, 116, 101, 115, 34, 58, 52, 48, 125, 125, 44, 34, 116, 97, 115, 107, 95, 99,
                111, 117, 110, 116, 101, 114, 34, 58, 49, 44, 34, 116, 97, 115, 107, 115, 34, 58,
                123, 34, 104, 97, 110, 100, 108, 101, 34, 58, 34, 49, 51, 50, 54, 57, 50, 52, 48,
                57, 56, 52, 57, 54, 55, 57, 51, 53, 56, 56, 56, 55, 54, 51, 49, 48, 54, 50, 51,
                50, 55, 55, 55, 49, 52, 53, 51, 52, 51, 55, 34, 125, 125, 125
            ]
        );
    }
    // #[ignore]
    #[test]
    fn test_get_table_entry() {
        // match serde_json::from_value::<u64>(
        //     serde_json::from_str::<serde_json::Value>("1").unwrap(),
        // ) {
        //     Ok(s) => {
        //         println!("s={:?}", s);
        //     }
        //     Err(s) => {
        //         println!("s=error =={:?}", s);
        //     }
        // }
        let api = PontNet {
            api: "http://localhost:9933".to_string(),
        };

        let addr = ss58_to_address("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
        let module = api
            .get_table_entry(
                &addr,
                "132692409849679358887631062327771453437",
                "1",
                "u64",
                "0xD43593C715FDD31C61141ABD04A99FD6822C8558854CCDE39A5684E7A56DA27D::TodoList::Task",
                &None,
            )
            .unwrap()
            .unwrap();
        assert_eq!(
            module.0,
            [
                123, 34, 97, 100, 100, 114, 101, 115, 115, 34, 58, 91, 50, 49, 50, 44, 53, 51,
                44, 49, 52, 55, 44, 49, 57, 57, 44, 50, 49, 44, 50, 53, 51, 44, 50, 49, 49, 44,
                50, 56, 44, 57, 55, 44, 50, 48, 44, 50, 54, 44, 49, 56, 57, 44, 52, 44, 49, 54,
                57, 44, 49, 53, 57, 44, 50, 49, 52, 44, 49, 51, 48, 44, 52, 52, 44, 49, 51, 51,
                44, 56, 56, 44, 49, 51, 51, 44, 55, 54, 44, 50, 48, 53, 44, 50, 50, 55, 44, 49,
                53, 52, 44, 56, 54, 44, 49, 51, 50, 44, 50, 51, 49, 44, 49, 54, 53, 44, 49, 48,
                57, 44, 49, 54, 50, 44, 49, 50, 53, 93, 44, 34, 99, 111, 109, 112, 108, 101, 116,
                101, 100, 34, 58, 102, 97, 108, 115, 101, 44, 34, 99, 111, 110, 116, 101, 110,
                116, 34, 58, 34, 48, 120, 52, 50, 52, 50, 52, 50, 34, 44, 34, 116, 97, 115, 107,
                95, 105, 100, 34, 58, 49, 125
            ]
        );
    }
    // #[ignore]
    #[test]
    fn test_get_module_abi() {
        let api = PontNet {
            api: "http://localhost:9933".to_string(),
        };
        let module = api
            .get_module_abi(
                &ModuleId::new(
                    AccountAddress::from_hex_literal(
                        "0xD43593C715FDD31C61141ABD04A99FD6822C8558854CCDE39A5684E7A56DA27D",
                    )
                    .unwrap(),
                    Identifier::new("ScriptBook").unwrap(),
                ),
                &None,
            )
            .unwrap()
            .unwrap();
        // if let Ok(module_abi) = bcs::from_bytes::<ModuleAbi>(&module.0) {
        //     println!("module_abi={:?}", module_abi);
        // }
        assert_eq!(
            [
                161, 28, 235, 11, 2, 0, 0, 0, 6, 1, 0, 2, 3, 2, 10, 5, 12, 3, 7, 15, 23, 8, 38,
                32, 12, 70, 8, 0, 0, 0, 1, 0, 0, 0, 0, 2, 0, 0, 0, 1, 10, 2, 4, 72, 97, 115, 104,
                8, 115, 104, 97, 50, 95, 50, 53, 54, 8, 115, 104, 97, 51, 95, 50, 53, 54, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 1, 0, 1, 2, 0, 1, 1, 2, 0, 0
            ],
            module.0.as_slice()
        );
    }

    // #[ignore]
    #[test]
    fn test_encode_submission0() {
        let api = PontNet {
            api: "http://localhost:9933".to_string(),
        };
        // let addr = ss58_to_address("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
        let module = api
            .encode_submission(
                "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
                "ScriptBook",
                "test",
                &[],
                &[],
                &None,
            )
            .unwrap()
            .unwrap();
        assert_eq!(
            module.0,
            [
                0, 0, 1, 212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214,
                130, 44, 133, 88, 133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 162, 125, 10,
                83, 99, 114, 105, 112, 116, 66, 111, 111, 107, 4, 116, 101, 115, 116, 0, 0
            ]
        );
    }
    // #[ignore]
    #[test]
    fn test_encode_submission1() {
        let api = PontNet {
            api: "http://localhost:9933".to_string(),
        };
        // let addr = ss58_to_address("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
        let module = api
            .encode_submission(
                "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
                "ScriptBook",
                "sum_func",
                &["3", "9"],
                &[],
                &None,
            )
            .unwrap()
            .unwrap();
        assert_eq!(
            module.0,
            [
                0, 1, 1, 1, 212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159,
                214, 130, 44, 133, 88, 133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 162, 125,
                10, 83, 99, 114, 105, 112, 116, 66, 111, 111, 107, 8, 115, 117, 109, 95, 102,
                117, 110, 99, 2, 8, 3, 0, 0, 0, 0, 0, 0, 0, 8, 9, 0, 0, 0, 0, 0, 0, 0, 0
            ]
        );
    }
    // #[ignore]
    #[test]
    fn test_encode_submission2() {
        let api = PontNet {
            api: "http://localhost:9933".to_string(),
        };
        // let addr = ss58_to_address("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
        let module = api
            .encode_submission(
                "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
                "ScriptBook",
                "sum_funct",
                &["3", "9"],
                &["u8"],
                &None,
            )
            .unwrap()
            .unwrap();
        assert_eq!(
            module.0,
            [
                0, 1, 1, 1, 212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159,
                214, 130, 44, 133, 88, 133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 162, 125,
                10, 83, 99, 114, 105, 112, 116, 66, 111, 111, 107, 9, 115, 117, 109, 95, 102,
                117, 110, 99, 116, 2, 8, 3, 0, 0, 0, 0, 0, 0, 0, 8, 9, 0, 0, 0, 0, 0, 0, 0, 0
            ]
        );
    }
    // #[ignore]
    #[test]
    fn test_encode_submission3() {
        let api = PontNet {
            api: "http://localhost:9933".to_string(),
        };
        // let addr = ss58_to_address("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
        let module = api
            .encode_submission(
                "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
                "ScriptBook",
                "store_sum_func",
                &["3", "9"],
                &[],
                &None,
            )
            .unwrap()
            .unwrap();
        assert_eq!(
            module.0,
            [
                0, 1, 1, 1, 212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159,
                214, 130, 44, 133, 88, 133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 162, 125,
                10, 83, 99, 114, 105, 112, 116, 66, 111, 111, 107, 14, 115, 116, 111, 114, 101,
                95, 115, 117, 109, 95, 102, 117, 110, 99, 2, 8, 3, 0, 0, 0, 0, 0, 0, 0, 8, 9, 0,
                0, 0, 0, 0, 0, 0, 0
            ]
        );
    }
    // #[ignore]
    #[test]
    fn test_get_module_abis() {
        let api = PontNet {
            api: "http://localhost:9933".to_string(),
        };
        let module = api
            .get_module_abis(
                &ModuleId::new(
                    AccountAddress::from_hex_literal(
                        "0xD43593C715FDD31C61141ABD04A99FD6822C8558854CCDE39A5684E7A56DA27D",
                    )
                    .unwrap(),
                    Identifier::new("ScriptBook").unwrap(),
                ),
                &None,
            )
            .unwrap()
            .unwrap();
        assert_eq!(
            [
                161, 28, 235, 11, 2, 0, 0, 0, 6, 1, 0, 2, 3, 2, 10, 5, 12, 3, 7, 15, 23, 8, 38,
                32, 12, 70, 8, 0, 0, 0, 1, 0, 0, 0, 0, 2, 0, 0, 0, 1, 10, 2, 4, 72, 97, 115, 104,
                8, 115, 104, 97, 50, 95, 50, 53, 54, 8, 115, 104, 97, 51, 95, 50, 53, 54, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 1, 0, 1, 2, 0, 1, 1, 2, 0, 0
            ],
            module.0.as_slice()
        );
    }
}
