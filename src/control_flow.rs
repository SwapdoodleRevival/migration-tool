use std::{
    ops::ControlFlow,
    process::{ExitCode, Termination},
};

pub type MigrationFlow<T = ()> = ControlFlow<AbortMigration, T>;

pub struct AbortMigration;

impl Termination for AbortMigration {
    fn report(self) -> ExitCode {
        ExitCode::SUCCESS
    }
}
