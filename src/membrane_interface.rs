use alloc::string::String;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MembraneProgress {
    Open,
    ReadyToClose,
}

pub enum MembraneClosure<M, C, R> {
    Open(C),
    Closed { membrane: Option<M>, readout: R },
}

pub trait Membrane: Sized {
    type Input;
    type Readout;
    type Continuation;
    type Error;

    fn prepare(input: Self::Input) -> Result<Self, Self::Error>;
    fn advance(&mut self) -> Result<MembraneProgress, Self::Error>;
    fn close(
        self,
    ) -> Result<MembraneClosure<Self, Self::Continuation, Self::Readout>, Self::Error>;
}

pub type MembraneError = String;
