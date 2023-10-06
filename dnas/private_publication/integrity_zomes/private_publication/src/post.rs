use hdi::prelude::*;

// Uncomment this line
 use crate::{properties::progenitor, *};

#[derive(Clone)]
#[hdk_entry_helper]
pub struct Post {
    pub title: String,
    pub content: String,
}

/**
* Implement these functions
 */

pub fn validate_create_post(
    action: EntryCreationAction,
    post: Post,
) -> ExternResult<ValidateCallbackResult> {
        let pub_role = PublicationRole { role: String::from("editor"), assignee: action.author().clone()};
        let role_hash = hash_entry(pub_role)?;
        let entry = must_get_entry(role_hash)?;
    
        //solution above might not be secure.. a bad actor might re-create the entry by another means
        // alternative is to use a claim and cache entry's of the role type and then check by the agent's activity
        // must_get_agent_activity(author, filter)
        
        return Ok(ValidateCallbackResult::Valid);
}


pub fn validate_update_post(
    action: Update,
    post: Post,
    original_action: EntryCreationAction,
    original_post: Post,
) -> ExternResult<ValidateCallbackResult> {
    if &action.author == original_action.author() {
        return Ok(ValidateCallbackResult::Valid)
    }
    Ok(ValidateCallbackResult::Invalid("Only the creator can make updates".into()))
}

/** These validations are already implemeneted, don't touch **/

pub fn validate_delete_post(
    _action: Delete,
    _original_action: EntryCreationAction,
    _original_post: Post,
) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Invalid(String::from(
        "Posts cannot be deleted",
    )))
}
pub fn validate_create_link_all_posts(
    _action: CreateLink,
    _base_address: AnyLinkableHash,
    target_address: AnyLinkableHash,
    _tag: LinkTag,
) -> ExternResult<ValidateCallbackResult> {
    let action_hash = ActionHash::from(target_address);
    let record = must_get_valid_record(action_hash)?;
    let _post: Post = record
        .entry()
        .to_app_option()
        .map_err(|e| wasm_error!(e))?
        .ok_or(wasm_error!(WasmErrorInner::Guest(String::from(
            "Linked action must reference an entry"
        ))))?;
    Ok(ValidateCallbackResult::Valid)
}
pub fn validate_delete_link_all_posts(
    _action: DeleteLink,
    _original_action: CreateLink,
    _base: AnyLinkableHash,
    _target: AnyLinkableHash,
    _tag: LinkTag,
) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Invalid(String::from(
        "AllPosts links cannot be deleted",
    )))
}
