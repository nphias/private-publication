use hdi::prelude::*;

use crate::properties::progenitor;

// Uncomment this line
// use crate::properties::progenitor;

#[derive(Clone)]
#[hdk_entry_helper]
pub struct PublicationRole {
    pub role: String,
    pub assignee: AgentPubKey,
}

/**
* Implement this function
 */
pub fn validate_create_publication_role(
    action: EntryCreationAction,
    publication_role: PublicationRole,
) -> ExternResult<ValidateCallbackResult> {
        if action.author().eq(&progenitor(())?){ 
            return Ok(ValidateCallbackResult::Valid);
        }
    Ok(ValidateCallbackResult::Invalid(String::from("Only the progenitor can create roles")))

}

/** Validation that is already implemented, don't touch **/

pub fn validate_update_publication_role(
    _action: Update,
    _publication_role: PublicationRole,
    _original_action: EntryCreationAction,
    _original_publication_role: PublicationRole,
) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Invalid(String::from(
        "Publication Roles cannot be updated",
    )))
}

pub fn validate_delete_publication_role(
    _action: Delete,
    _original_action: EntryCreationAction,
    _original_publication_role: PublicationRole,
) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Invalid(String::from(
        "Publication Roles cannot be deleted",
    )))
}
