use std::str::FromStr;
use std::fmt::Debug;
use anyhow::{Error, Result};
use move_symbol_pool::Symbol;
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::{CORE_CODE_ADDRESS, TypeTag};
use move_package::source_package::parsed_manifest::AddressDeclarations;
use lang::bytecode::accessor::BytecodeType;
use lang::bytecode::{find, SearchParams};
use lang::bytecode::info::{BytecodeInfo, Type};
use crate::context::Context;
use crate::call::model::{Signer, ScriptArg, Transaction, Signers, EnrichedTransaction, Call};
use crate::call::parser::parse_vec;
use crate::call::bytecode::DoveBytecode;

fn diem_root_address() -> AccountAddress {
    AccountAddress::from_hex_literal("0xA550C18")
        .expect("Parsing valid hex literal should always succeed")
}

/// Transaction config.
pub struct Config {
    /// Is transaction for chain execution.
    tx_context: bool,
    /// Prohibit the definition of signers.
    deny_signers_definition: bool,
}

impl Config {
    /// Returns transaction config for chain transaction.
    pub fn for_tx() -> Config {
        Config {
            tx_context: true,
            deny_signers_definition: true,
        }
    }

    /// Returns transaction config for local execution.
    pub fn for_run() -> Config {
        Config {
            tx_context: false,
            deny_signers_definition: false,
        }
    }
}

pub(crate) fn make_script_call(
    ctx: &Context,
    addr_map: &AddressDeclarations,
    name: Identifier,
    type_tag: Vec<TypeTag>,
    args: Vec<String>,
    package_name: Option<String>,
    cfg: Config,
) -> Result<EnrichedTransaction, Error> {
    let access = DoveBytecode::new(ctx);
    let functions = find(
        access,
        SearchParams {
            tp: Some(BytecodeType::Script),
            package: package_name.as_deref(),
            name: Some(name.as_str()),
        },
    )?
    .filter_map(|f| f.ok());
    let (signers, args, info) =
        select_function(functions, &name, &args, &type_tag, &cfg, addr_map)?;

    Ok(if cfg.tx_context {
        let (_, mut tx) = match signers {
            Signers::Explicit(signers) => (
                signers,
                Transaction::new_script_tx(vec![], vec![], args, type_tag)?,
            ),
            Signers::Implicit(signers) => (
                vec![],
                Transaction::new_script_tx(signers, vec![], args, type_tag)?,
            ),
        };

        let mut buff = Vec::new();
        info.serialize(&mut buff)?;

        match &mut tx.inner_mut().call {
            Call::Script { code, .. } => *code = buff,
            Call::ScriptFunction { .. } => {
                // no-op
            }
        }
        EnrichedTransaction::Global {
            bi: info,
            tx,
            name: name.into_string(),
        }
    } else {
        let signers = match signers {
            Signers::Explicit(signers) => signers,
            Signers::Implicit(_) => vec![],
        };

        EnrichedTransaction::Local {
            bi: info,
            args,
            signers,
            type_tag,
            func_name: None,
        }
    })
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn make_function_call(
    ctx: &Context,
    addr_map: &AddressDeclarations,
    address: Option<AccountAddress>,
    module: Identifier,
    func: Identifier,
    type_tag: Vec<TypeTag>,
    args: Vec<String>,
    package_name: Option<String>,
    cfg: Config,
) -> Result<EnrichedTransaction, Error> {
    let access = DoveBytecode::new(ctx);
    let modules = find(
        access,
        SearchParams {
            tp: Some(BytecodeType::Module),
            package: package_name.as_deref(),
            name: Some(module.as_str()),
        },
    )?
    .filter_map(|info| info.ok())
    .filter(|info| {
        if address.is_some() {
            info.address() == address
        } else {
            true
        }
    })
    .filter(|info| info.name() == module.as_str());
    let (signers, args, info) =
        select_function(modules, &func, &args, &type_tag, &cfg, addr_map)?;

    let addr = info.address().unwrap_or(CORE_CODE_ADDRESS);
    let tx_name = format!("{}_{}", module, func);

    if cfg.tx_context {
        let tx = match signers {
            Signers::Explicit(_) => {
                Transaction::new_func_tx(vec![], addr, module, func, args, type_tag)?
            }
            Signers::Implicit(signers) => {
                Transaction::new_func_tx(signers, addr, module, func, args, type_tag)?
            }
        };
        Ok(EnrichedTransaction::Global {
            bi: info,
            tx,
            name: tx_name,
        })
    } else {
        let signers = match signers {
            Signers::Explicit(signers) => signers,
            Signers::Implicit(_) => vec![],
        };

        Ok(EnrichedTransaction::Local {
            bi: info,
            signers,
            args,
            type_tag,
            func_name: Some(func.into_string()),
        })
    }
}

fn select_function<I>(
    info_iter: I,
    name: &Identifier,
    args: &[String],
    type_tag: &[TypeTag],
    cfg: &Config,
    addr_map: &AddressDeclarations,
) -> Result<(Signers, Vec<ScriptArg>, BytecodeInfo), Error>
where
    I: Iterator<Item = BytecodeInfo>,
{
    let mut functions = info_iter
        .filter_map(|info| info.find_script_function(name.as_str()).map(|f| (info, f)))
        .filter(|(_, f)| type_tag.len() == f.type_params_count())
        .map(|(i, script)| {
            prepare_function_signature(
                &script.parameters,
                args,
                !cfg.deny_signers_definition,
                addr_map,
            )
            .map(|(signers, args)| (i, script, signers, args))
        })
        .collect::<Vec<Result<_, _>>>();
    let count = functions.iter().filter(|r| r.is_ok()).count();
    if count == 0 {
        if functions.is_empty() {
            bail!("Couldn't find a function with  given signature.functions is empty.");
        } else {
            functions.remove(0)?;
            unreachable!();
        }
    } else if count > 1 {
        bail!(
            "More than one functions with the given signature was found.\
                   Please pass the package name to specify the package or use unique signatures."
        );
    } else {
        let (bytecode_info, _, signers, args) = functions
            .into_iter()
            .find_map(|res| res.ok())
            .ok_or_else(|| anyhow!("Couldn't find a function with given signature."))?;
        Ok((signers, args, bytecode_info))
    }
}

fn prepare_function_signature(
    code_args: &[Type],
    call_args: &[String],
    use_explicit_signers: bool,
    addr_map: &AddressDeclarations,
) -> Result<(Signers, Vec<ScriptArg>), Error> {
    let signers_count = code_args
        .iter()
        .take_while(|tp| **tp == Type::Signer)
        .count();
    let params_count = code_args.len() - signers_count;

    if call_args.len() < params_count {
        bail!(
            "The function accepts {} parameters, {} are passed",
            params_count,
            call_args.len()
        );
    }

    let args_index = call_args.len() - params_count;
    let params = code_args[signers_count..]
        .iter()
        .zip(&call_args[args_index..])
        .map(|(tp, val)| prepare_arg(tp, val, addr_map))
        .collect::<Result<Vec<_>, Error>>()?;

    if use_explicit_signers {
        let signers = call_args[..args_index]
            .iter()
            .map(|arg| {
                if arg.starts_with("0x") {
                    AccountAddress::from_hex_literal(arg)
                        .map_err(|err| anyhow!("Failed to parse signer:{}", err))
                } else {
                    Signer::from_str(arg).and_then(|s| {
                        Ok(match s {
                            Signer::Root => diem_root_address(),
                            Signer::Placeholder => {
                                return Err(anyhow!(
                                    "Use explicit signer instead of placeholder"
                                ));
                            }
                            Signer::Name(name) => {
                                addr_map.get(&name).and_then(|addr| *addr).ok_or_else(|| {
                                    anyhow!("Failed to find address with name:{}", arg)
                                })?
                            }
                        })
                    })
                }
            })
            .collect::<Result<Vec<AccountAddress>, Error>>()
            .map_err(|err| anyhow!("Failed to parse signer:{}", err))?;
        ensure!(
            signers.len() == signers_count,
            "The function accepts {} signers, {} are passed",
            signers_count,
            signers.len()
        );
        Ok((Signers::Explicit(signers), params))
    } else {
        let mut signers = (0..signers_count)
            .take_while(|i| *i < args_index)
            .map(|i| Signer::from_str(&call_args[i]).ok())
            .take_while(|s| s.is_some())
            .flatten()
            .collect::<Vec<_>>();
        let explicit_signers = signers.len();

        for _ in explicit_signers..signers_count {
            signers.push(Signer::Placeholder);
        }

        Ok((Signers::Implicit(signers), params))
    }
}

fn prepare_arg(
    arg_type: &Type,
    arg_value: &str,
    addr_map: &AddressDeclarations,
) -> Result<ScriptArg, Error> {
    macro_rules! parse_primitive {
        ($script_arg:expr) => {
            $script_arg(
                arg_value
                    .parse()
                    .map_err(|err| parse_err(arg_type, arg_value, err))?,
            )
        };
    }

    Ok(match arg_type {
        Type::Bool => parse_primitive!(ScriptArg::Bool),
        Type::U8 => parse_primitive!(ScriptArg::U8),
        Type::U64 => parse_primitive!(ScriptArg::U64),
        Type::U128 => parse_primitive!(ScriptArg::U128),
        Type::Address => ScriptArg::Address(parse_address(arg_value, addr_map)?),
        Type::Vector(tp) => match tp.as_ref() {
            Type::Bool => ScriptArg::VectorBool(
                parse_vec(arg_value, "bool")
                    .map_err(|err| parse_err(arg_type, arg_value, err))?,
            ),
            Type::U8 => ScriptArg::VectorU8(if arg_value.contains('[') {
                parse_vec(arg_value, "u8").map_err(|err| parse_err(arg_type, arg_value, err))?
            } else {
                hex::decode(arg_value).map_err(|err| parse_err(arg_type, arg_value, err))?
            }),
            Type::U64 => ScriptArg::VectorU64(
                parse_vec(arg_value, "u64").map_err(|err| parse_err(arg_type, arg_value, err))?,
            ),
            Type::U128 => ScriptArg::VectorU128(
                parse_vec(arg_value, "u64").map_err(|err| parse_err(arg_type, arg_value, err))?,
            ),
            Type::Address => {
                let addresses = parse_vec::<String>(arg_value, "vector<address>")
                    .map_err(|err| parse_err(arg_type, arg_value, err))?
                    .into_iter()
                    .map(|addr| parse_address(&addr, addr_map))
                    .collect::<Result<Vec<_>, Error>>()?;
                ScriptArg::VectorAddress(addresses)
            }
            Type::Signer
            | Type::Vector(_)
            | Type::Struct(_)
            | Type::Reference(_)
            | Type::MutableReference(_)
            | Type::TypeParameter(_) => {
                anyhow::bail!("Unexpected script parameter: {:?}", arg_type)
            }
        },
        Type::Signer
        | Type::Struct(_)
        | Type::Reference(_)
        | Type::MutableReference(_)
        | Type::TypeParameter(_) => anyhow::bail!("Unexpected script parameter: {:?}", arg_type),
    })
}

fn pontem_parse_address(addr: &str) -> Result<AccountAddress> {
    if let Ok(address) = pontem::ss58_to_address(addr) {
        // first try ss58 parsing
        Ok(address)
    } else {
        let mut addr = addr.to_string();
        if !addr.starts_with("0x") {
            addr = format!("0x{}", addr);
        }
        // try parsing hex diem/aptos address with optional 0x prefix
        let max_hex_len = AccountAddress::LENGTH * 2 + 2;
        if addr.len() > max_hex_len {
            return Err(anyhow::anyhow!(
                "Unable to parse AccountAddress. Maximum address length is {}.  Actual {}",
                max_hex_len,
                addr
            ));
        }
        use anyhow::Context;
        AccountAddress::from_hex_literal(&addr)
            .with_context(|| format!("Address {:?} is not a valid diem/pont address", addr))
    }
}

fn parse_address(
    arg_value: &str,
    addr_map: &AddressDeclarations,
) -> Result<AccountAddress, Error> {
    match pontem_parse_address(arg_value) {
        Ok(addr) => Ok(addr),
        Err(_) => addr_map
            .get(&Symbol::from(arg_value))
            .and_then(|addr| *addr)
            .ok_or_else(|| anyhow!("Failed to find address with name:{}", arg_value)),
    }
}

fn parse_err<D: Debug>(tp: &Type, value: &str, err: D) -> Error {
    anyhow!(
        "Parameter has type {:?}. Failed to parse {}. Error:'{:?}'",
        tp,
        value,
        err
    )
}

#[cfg(test)]
mod call_tests {
    use move_core_types::language_storage::CORE_CODE_ADDRESS;
    use move_core_types::account_address::AccountAddress;
    use lang::bytecode::info::Type;
    use crate::call::model::ScriptArg;
    use crate::call::fn_call::prepare_function_signature;

    fn s(v: &str) -> String {
        v.to_string()
    }

    fn addr(v: &str) -> AccountAddress {
        AccountAddress::from_hex_literal(v).unwrap()
    }

    #[test]
    fn test_args_types() {
        let (signers, args) =
            prepare_function_signature(&[], &[], true, &Default::default()).unwrap();
        assert_eq!(signers.len(), 0);
        assert_eq!(args.len(), 0);

        let (signers, args) =
            prepare_function_signature(&[Type::U8], &[s("1")], true, &Default::default())
                .unwrap();
        assert_eq!(signers.len(), 0);
        assert_eq!(args, vec![ScriptArg::U8(1)]);

        let (signers, args) = prepare_function_signature(
            &[Type::Bool, Type::Bool],
            &[s("true"), s("false")],
            true,
            &Default::default(),
        )
        .unwrap();
        assert_eq!(signers.len(), 0);
        assert_eq!(args, vec![ScriptArg::Bool(true), ScriptArg::Bool(false)]);

        let (signers, args) = prepare_function_signature(
            &[Type::U64, Type::U64, Type::U128],
            &[s("0"), s("1000000000"), s("10000000000000000")],
            true,
            &Default::default(),
        )
        .unwrap();
        assert_eq!(signers.len(), 0);
        assert_eq!(
            args,
            vec![
                ScriptArg::U64(0),
                ScriptArg::U64(1000000000),
                ScriptArg::U128(10000000000000000),
            ]
        );

        let (signers, args) =
            prepare_function_signature(&[Type::Address], &[s("0x1")], true, &Default::default())
                .unwrap();
        assert_eq!(signers.len(), 0);
        assert_eq!(args, vec![ScriptArg::Address(CORE_CODE_ADDRESS)]);

        let (signers, args) = prepare_function_signature(
            &[
                Type::Vector(Box::new(Type::Bool)),
                Type::Vector(Box::new(Type::U8)),
                Type::Vector(Box::new(Type::U8)),
                Type::Vector(Box::new(Type::U8)),
                Type::Vector(Box::new(Type::U64)),
                Type::Vector(Box::new(Type::U128)),
                Type::Vector(Box::new(Type::Address)),
            ],
            &[
                s("[true, false]"),
                s("[100]"),
                s("[]"),
                s("0102"),
                s("[1000, 0]"),
                s("[0]"),
                s("[0x1, 0x2]"),
            ],
            true,
            &Default::default(),
        )
        .unwrap();
        assert_eq!(signers.len(), 0);
        assert_eq!(
            args,
            vec![
                ScriptArg::VectorBool(vec![true, false]),
                ScriptArg::VectorU8(vec![100]),
                ScriptArg::VectorU8(vec![]),
                ScriptArg::VectorU8(vec![1, 2]),
                ScriptArg::VectorU64(vec![1000, 0]),
                ScriptArg::VectorU128(vec![0]),
                ScriptArg::VectorAddress(vec![addr("0x1"), addr("0x2")]),
            ]
        );
    }
}
