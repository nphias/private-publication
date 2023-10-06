use hdk::prelude::{
    holo_hash::{DnaHash, DnaHashB64, AgentPubKeyB64},
    *, tracing::field::debug,
};
use membrane_proof::*;
use private_publication_lobby_integrity::*;

/**
 * Add your edits to the bottom of this file
 */

// Don't change
#[cfg(feature = "exercise")]
extern crate private_publication_lobby;

#[derive(Serialize, Deserialize, Debug)]
pub struct GrantCapabilityToReadInput {
    reader: AgentPubKey,
    private_publication_dna_hash: DnaHash
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StoreCapClaimInput {
    author: AgentPubKey,
    cap_secret: CapSecret
}



//private_publication_lobby

#[hdk_extern]
pub fn grant_capability_to_read(grant_input:GrantCapabilityToReadInput) -> ExternResult<CapSecret>{
    let cap_tag = DnaHashB64::from(grant_input.private_publication_dna_hash).to_string();
    let functions = functions_to_grant_capability_for()?;
    let cap_secret = cap_secret()?;
    let mut assignees: BTreeSet<AgentPubKey> = BTreeSet::new();
    assignees.insert(grant_input.reader);

    let access = CapAccess::Assigned { secret: cap_secret, assignees: assignees }; 
    let capability_grant = CapGrantEntry { 
        functions,
        access,
        tag: String::from(cap_tag),    // Arbitrary humand readable tag, just for convenience
      };
    create_cap_grant(capability_grant)?;
    Ok(cap_secret)
}

#[hdk_extern]
// The "CapSecret" would be provided by another process, // maybe a bluetooth handshake
pub fn store_capability_claim(claim_input: StoreCapClaimInput) -> ExternResult<()>{
    let cap_claim = CapClaimEntry {   // Private entry built-in to Holochain and the HDK
    grantor: claim_input.author, // Just to remember which agent to call
    secret: claim_input.cap_secret, // Store the secret to be able to add it to the request
    tag: String::from("claim to read Bobs posts"), // Can be totally different from the tag in the capability grant                                       
  };
  create_cap_claim(cap_claim)?; // Create the claim privately, nothing else happens
  Ok(())
}

#[hdk_extern]
pub fn read_posts_for_author(poster:AgentPubKey) -> ExternResult<Vec<Record>>{
    let cap_claims = query_cap_claims_for(&poster)?;
    let wanted_claim = &cap_claims
        .into_iter()
        .filter(|claim|claim.tag()
        .eq_ignore_ascii_case("claim to read Bobs posts"))
        .collect::<Vec<CapClaim>>()[0];
    //debug!()
    let zome_response = call_remote(wanted_claim.grantor.clone(), zome_info()?.name, "request_read_private_publication_posts".into(), Some(wanted_claim.secret), ())?;
    match zome_response {
        ZomeCallResponse::Ok(result) => {     // Of type ExternIO, wrapper around byte array
            let posts: Vec<Record> = result.decode().map_err(|err| wasm_error!(err))?;
            Ok(posts)
        },
        ZomeCallResponse::NetworkError(err) => {
            Err(wasm_error!(WasmErrorInner::Guest(format!("There was a network error: {:?}", err))))?
          },
        _ => Err(wasm_error!(WasmErrorInner::Guest(
              "Failed to handle remote call".into()
        )))?
    }
}


// capability.. 

#[hdk_extern]
pub fn request_read_private_publication_posts(_:()) -> ExternResult<Vec<Record>> {
    let info = call_info()?;
    let grant = info.cap_grant;
    let grant_info = match grant {
        // The cell's agent called this function
        CapGrant::ChainAuthor(_my_pub_key) => {None}, 
        // An external agent called this function via call or call_remote, and it was 
        // authorized because this "zome_call_cap_grant" exists in the source chain for this cell
        CapGrant::RemoteAgent(zome_call_cap_grant) => {Some(zome_call_cap_grant)} 
    };
    if let Some(zome_call_cap_grant) = grant_info {
        let private_publication_dna_hash = DnaHash::from(
            DnaHashB64::from_b64_str(zome_call_cap_grant.tag.as_str()).or(Err(wasm_error!(
                WasmErrorInner::Guest(String::from("Bad cap_grant tag"))
            )))?,
        );
        let cell_id = CellId::new(private_publication_dna_hash, agent_info()?.agent_latest_pubkey);
        let call_response = call(CallTargetCell::OtherCell(cell_id), ZomeName::from(String::from("posts")), FunctionName(String::from("get_all_posts")), None, ())?;
        match call_response {
            ZomeCallResponse::Ok(result) => {     // Of type ExternIO, wrapper around byte array
                let posts: Vec<Record> = result.decode().map_err(|err| wasm_error!(err))?;
                return Ok(posts)
            },
            ZomeCallResponse::NetworkError(err) => {
                Err(wasm_error!(WasmErrorInner::Guest(format!("There was a network error: {:?}", err))))?
              },
            _ => Err(wasm_error!(WasmErrorInner::Guest(
                  "Failed to handle remote call".into()
            )))?
        }
    }
    Err(wasm_error!(WasmErrorInner::Guest("Failed to handle remote call".into())))
}

#[hdk_extern]
pub fn create_membrane_proof_for(proof:PrivatePublicationMembraneProof)-> ExternResult<()>{
    let hash = create_entry(EntryTypes::PrivatePublicationMembraneProof(proof.clone()))?;
    create_link(proof.recipient, hash, LinkTypes::AgentToMembraneProof, ()).ok();
    Ok(())
}

#[hdk_extern]
pub fn get_my_membrane_proof(_:())-> ExternResult<Option<Record>>{
    let my_pub_key = agent_info()?.agent_latest_pubkey;
    let links = get_links(my_pub_key, LinkTypes::AgentToMembraneProof, None)?;    
    if !links.is_empty() {
        if let Some(target_hash) = links[0].target.clone().into_action_hash(){
            return get(target_hash,GetOptions::default())
        }
        return Ok(None)
    }
    Ok(None)
}

//helpers

// Declares which functions we want to grant access to
fn functions_to_grant_capability_for() -> ExternResult<GrantedFunctions> { 
      let zome_name = zome_info()?.name; 
    // Getting the zome name
      let fn_name = FunctionName::from("request_read_private_publication_posts");
    
      let mut functions: BTreeSet<(ZomeName, FunctionName)> = BTreeSet::new();
      functions.insert((zome_name, fn_name)); 
      Ok(GrantedFunctions::Listed(functions))     
}

// Generate a random capability secret to give to a the granted agent
fn cap_secret() -> ExternResult<CapSecret> { 
    let bytes = random_bytes(64)?; // "CapSecret" is a wrapper around a byte array

    // "random_bytes" is a utility function from the HDK
      let secret = CapSecret::try_from(bytes.into_vec())
          .map_err(|_| wasm_error!(WasmErrorInner::Guest("Could not build secret".into())))?;
    
    Ok(secret)
}

// Imagine we have already stored some CapClaims
// and we want to retrieve all stored CapClaims from a certain grantor
fn query_cap_claims_for(grantor: &AgentPubKey) -> ExternResult<Vec<CapClaim>> {
    let claims_records = query(
      ChainQueryFilter::new()
        .entry_type(EntryType::CapClaim)  // Only query Capability Claim related records
        .include_entries(true),// Include the Capability Claim entries in the records          
    )?;
  
    let claims_from_grantor: Vec<CapClaim> = claims_records
      .into_iter()
      .filter_map(|record| record.entry().as_option().cloned()) // Extract the entry from the record                                         
      .filter_map(|entry| match entry { Entry::CapClaim(claim) => Some(claim.clone()),   // Deserialize the entry to a "CapClaim"
        _ => None,
      })
      .filter(|claim| claim.grantor.eq(&grantor)) // Only select the claims with the given grantor
      .collect();
  
    Ok(claims_from_grantor)
  }  

  
  
  
  

