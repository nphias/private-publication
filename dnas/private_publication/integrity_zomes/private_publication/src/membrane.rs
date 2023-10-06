use hdi::prelude::*;
use membrane_proof::PrivatePublicationMembraneProof;
use std::sync::Arc;

// Uncomment this line
use crate::properties::progenitor;

/**
 * Add your edits to the bottom of this file
 */

pub fn is_membrane_proof_valid(
    for_agent: AgentPubKey,
    membrane_proof: Option<MembraneProof>,
) -> ExternResult<ValidateCallbackResult> {
    let progenitor_pub_key = progenitor(())?;
    if for_agent == progenitor_pub_key {
        return Ok(ValidateCallbackResult::Valid);
    } 
    match membrane_proof {
        None => Ok(ValidateCallbackResult::Invalid(
            "Invalid agent: no membrane proof present".into(),
        )),
        Some(proof) => {
            let bytes = Arc::try_unwrap(proof)
                .map_err(|err| wasm_error!(WasmErrorInner::Guest(format!("{:?}", err))))?;
            let record = Record::try_from(bytes).map_err(|err| wasm_error!(err))?;

            if !record.action().author().eq(&progenitor_pub_key) {
                return Ok(ValidateCallbackResult::Invalid(
                    "The author of the record is not the progenitor".into(),
                ));
            }

            if !verify_signature(
                progenitor_pub_key,
                record.signature().clone(),
                record.action_hashed().as_content(),
            )? {
                return Ok(ValidateCallbackResult::Invalid(
                    "The signature of the record is not valid".into(),
                ));
            }

            let maybe_private_publication_membrane_proof: Option<PrivatePublicationMembraneProof> =
                record
                    .entry()
                    .to_app_option()
                    .map_err(|err| wasm_error!(err))?;

            match maybe_private_publication_membrane_proof {
                Some(private_publication_membrane_proof) => {
                    let actual_entry_hash = hash_entry(&private_publication_membrane_proof)?;
                    let entry_hash_in_action = record.action().entry_hash().ok_or(
                        wasm_error!(WasmErrorInner::Guest(String::from("The given record doesn't contain an entry hash")))
                    )?.clone();

                    if !entry_hash_in_action.eq(&actual_entry_hash) {
                        return Ok(ValidateCallbackResult::Invalid(
                            "The entry hash is not valid for the given entry".into(),
                        ));
                    }

                    if private_publication_membrane_proof.dna_hash != dna_info()?.hash {
                        return Ok(ValidateCallbackResult::Invalid(
                            "The membrane proof is not for this dna".into(),
                        ));
                    }

                    if !private_publication_membrane_proof.recipient.eq(&for_agent) {
                        return Ok(ValidateCallbackResult::Invalid(
                            "The membrane proof is not for this agent".into(),
                        ));
                    }
                }
                None => {
                    return Ok(ValidateCallbackResult::Invalid(
                        "Malformed membrane proof".into(),
                    ));
                }
            }

            Ok(ValidateCallbackResult::Valid)
        }
    }
}
