use bson::oid::ObjectId;
use chrono::Local;
use thiserror::Error;

use crate::{
    training::{Training, TrainingId, TrainingStatus},
    user::rate::Rate,
};

#[derive(Error, Debug)]
pub enum LedgerError {
    // common
    #[error("Common error: {0}")]
    Eyre(#[from] eyre::Error),
    #[error("Mongo error: {0}")]
    MongoError(#[from] mongodb::error::Error),
    // users
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
    #[error("User already in family")]
    UserAlreadyInFamily {
        user_id: ObjectId,
        member_id: ObjectId,
    },
    #[error("User already employee")]
    UserAlreadyEmployee { user_id: ObjectId },
    #[error("User not employee")]
    UserNotEmployee { user_id: ObjectId },
    #[error("Employee has reward")]
    EmployeeHasReward { user_id: ObjectId },
    #[error("Employee has trainings")]
    CouchHasTrainings(ObjectId),
    #[error("Employee has trainings")]
    NoRatesFound { user_id: ObjectId },
    #[error("Rate not found")]
    RateNotFound { user_id: ObjectId, rate: Rate },
    #[error("Rate already exists")]
    RateTypeAlreadyExists { user_id: ObjectId, rate: Rate },
    #[error("Wrong numbers of users")]
    WrongTrainingClients { training_id: TrainingId },
    #[error("Request not found")]
    RequestNotFound { id: ObjectId },

    //new training
    #[error("Program not found:{0}")]
    ProgramNotFound(ObjectId),
    #[error("Instructor not found:{0}")]
    InstructorNotFound(ObjectId),
    #[error("Client not found:{0}")]
    ClientNotFound(ObjectId),
    #[error("Instructor has no rights:{0}")]
    InstructorHasNoRights(ObjectId),
    #[error("Too close to start")]
    TooCloseToStart { start_at: chrono::DateTime<Local> },
    #[error("Time slot collision:{0:?}")]
    TimeSlotCollision(Training),

    //signin
    #[error("Training not open to sign up")]
    TrainingNotOpenToSignUp(TrainingId, TrainingStatus),
    #[error("Client already signed up:{0:?} {1:?}")]
    ClientAlreadySignedUp(ObjectId, TrainingId),
    #[error("Training is full:{0:?}")]
    TrainingIsFull(TrainingId),
    #[error("Not enough balance:{0:?}")]
    NotEnoughBalance(ObjectId),


    //signout
    #[error("Training not found:{0:?}")]
    TrainingNotFound(TrainingId),
    #[error("Training is not open to sign out:{0:?}")]
    TrainingNotOpenToSignOut(TrainingId),
    #[error("Client not signed up:{0:?} {1:?}")]
    ClientNotSignedUp(ObjectId, TrainingId),
    #[error("Not enough reserved balance:{0:?}")]
    NotEnoughReservedBalance(ObjectId),

    // delete training
    #[error("Training has clients")]
    TrainingHasClients(TrainingId),
}
