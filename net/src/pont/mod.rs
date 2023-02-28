use std::fmt::{Display, Formatter};
use anyhow::{bail, Result};
use serde::{Serialize, Deserialize};

use move_core_types::language_storage::{ModuleId, StructTag};
use move_core_types::account_address::AccountAddress;

use lang::ss58::address_to_ss58;
use crate::{Net, BytesForBlock};
mod move_types;
mod address;
mod bytecode;
mod wrappers;
mod abi;
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
        println!("================0x{}",hex::encode(bcs::to_bytes(module_id)?));
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
        // println!("{:?}", response.json::<Response>());
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

    fn encode_submission(
        &self,
        addr:&str,
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
                format!("0x{}", hex::encode(function.as_bytes()))
                ],
arguments.iter().map(|a| format!("0x{}", hex::encode(a.as_bytes()))).collect(),
               type_parameters.iter().map(|a| format!("0x{}", hex::encode(a.as_bytes()))).collect()
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
        println!("================0x{}",hex::encode(bcs::to_bytes(module_id)?));
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
        // println!("{:?}", response.json::<Response>());
        let resp = response.json::<Response>()?;
        if let Some(err) = resp.error {
            bail!("{:?}", err);
        }
        if let Some(result) = resp.result {
            println!("result=={:?}==={:?}", &result,std::str::from_utf8(&hex::decode(&result[2..]).unwrap()).unwrap());
            let s=std::str::from_utf8(&hex::decode(&result[2..]).unwrap()).unwrap().to_string();//.replace("\"","");
            println!("s====={:?}", &s);
            let res1:MoveModuleBytecode = serde_json::from_str(&s).unwrap();
            println!("res1====={:?}", res1);
            let res:MoveModuleBytecode = serde_json::from_slice(&hex::decode(&result[2..]).unwrap()).unwrap();
            println!("res====={:?}", res);
            Ok(Some(BytesForBlock(
                result.into(),
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
                    AccountAddress::from_hex_literal("0xD43593C715FDD31C61141ABD04A99FD6822C8558854CCDE39A5684E7A56DA27D").unwrap(),
                    Identifier::new("ScriptBook").unwrap(),
                ),
                &None,
            )
            .unwrap()
            .unwrap();
        println!("module={:?}",MoveModuleBytecode::new(module.0.clone()).try_parse_abi());
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
    fn test_get_resource() {
        let api = PontNet {
            api: "http://localhost:9933".to_string(),
        };

        let addr = ss58_to_address("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
        let module = api
            .get_resource(
                &addr,
                &StructTag {
                    address: addr,
                    module: Identifier::new("Store").unwrap(),
                    name: Identifier::new("U64").unwrap(),
                    type_params: vec![],
                },
                &None,
            )
            .unwrap()
            .unwrap();
        assert_eq!(module.0, [100, 0, 0, 0, 0, 0, 0, 0]);
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
                    AccountAddress::from_hex_literal("0xD43593C715FDD31C61141ABD04A99FD6822C8558854CCDE39A5684E7A56DA27D").unwrap(),
                    Identifier::new("ScriptBook").unwrap(),
                ),
                &None,
            )
            .unwrap()
            .unwrap();
            if let Ok(module_abi) = bcs::from_bytes::<ModuleAbi>(&module.0) {
                        println!("module_abi={:?}",module_abi);
            }
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
    fn test_encode_submission() {
        let api = PontNet {
            api: "http://localhost:9933".to_string(),
        };
println!("module={:?}",1);
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
        println!("module={:?}",hex::encode(&module.0));
        assert_eq!(module.0, [100, 0, 0, 0, 0, 0, 0, 0]);
    }
 // #[ignore]
    #[test]
    fn test_encode_submission1() {
        let api = PontNet {
            api: "http://localhost:9933".to_string(),
        };
println!("module={:?}",1);
        // let addr = ss58_to_address("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
        let module = api
            .encode_submission(
                "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
                "ScriptBook",
                 "sum_func",
                &["3","9"],
                &[],
                &None,
            )
            .unwrap()
            .unwrap();
        println!("module={:?}",hex::encode(&module.0));
        assert_eq!(module.0, [100, 0, 0, 0, 0, 0, 0, 0]);
    }
 // #[ignore]
    #[test]
    fn test_encode_submission2() {
        let api = PontNet {
            api: "http://localhost:9933".to_string(),
        };
println!("module={:?}",1);
        // let addr = ss58_to_address("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
        let module = api
            .encode_submission(
                "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
                "ScriptBook",
                 "sum_funct",
                &["3","9"],
                &["u8"],
                &None,
            )
            .unwrap()
            .unwrap();
        println!("module={:?}",hex::encode(&module.0));
        assert_eq!(module.0, [100, 0, 0, 0, 0, 0, 0, 0]);
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
                    AccountAddress::from_hex_literal("0xD43593C715FDD31C61141ABD04A99FD6822C8558854CCDE39A5684E7A56DA27D").unwrap(),
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
