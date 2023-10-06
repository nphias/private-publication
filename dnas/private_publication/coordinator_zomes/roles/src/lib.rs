use hdk::prelude::*;

// Uncomment this line
use private_publication_integrity::{EntryTypes, PublicationRole};

/**
 * Add your edits to the bottom of this file
 */

/** Don't change */
#[cfg(not(feature = "exercise2"))]
extern crate roles;

#[hdk_extern]
pub fn assign_editor_role(agent:AgentPubKey)-> ExternResult<PublicationRole>{
  let pub_role = PublicationRole { role: String::from("editor"), assignee: agent };
  let _ = create_entry(EntryTypes::PublicationRole(pub_role.clone()));
  Ok(pub_role)
}
