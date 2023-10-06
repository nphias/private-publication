use hdi::prelude::{holo_hash::AgentPubKeyB64, *};

#[derive(Serialize, Deserialize, Debug, SerializedBytes)]
pub struct Properties {
    progenitor: AgentPubKeyB64
}

#[hdk_extern]
pub fn progenitor(_:()) -> ExternResult<AgentPubKey> {
    let props: Properties = dna_info()?.properties.try_into()
    .map_err(|e| wasm_error!(WasmErrorInner::Serialize(e)))?;
    Ok(AgentPubKey::from(props.progenitor))
}
