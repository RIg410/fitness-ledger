use bson::oid::ObjectId;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LedgerError {
    #[error("Common error: {0}")]
    Eyre(#[from] eyre::Error),
    #[error("User not found: {0}")]
    UserNotFound(ObjectId),
    #[error("Member not found")]
    MemberNotFound {
        user_id: ObjectId,
        member_id: ObjectId,
    },
    #[error("Wrong family member")]
    WrongFamilyMember {
        user_id: ObjectId,
        member_id: ObjectId,
    },
    #[error("Mongo error: {0}")]
    MongoError(#[from] mongodb::error::Error),
    #[error("User already in family")]
    UserAlreadyInFamily {
        user_id: ObjectId,
        member_id: ObjectId,
    },
}
