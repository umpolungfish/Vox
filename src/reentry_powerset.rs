//! Stage 43/44 powerset unit, multiplication and bind over retained arena IDs.
//! IDs name complete correlated states held in one shared execution arena.
use alloc::collections::BTreeSet;

pub type Held<X> = BTreeSet<X>;

pub fn unit<X: Ord>(x: X) -> Held<X> {
    core::iter::once(x).collect()
}

pub fn multiply<X: Ord>(families: Held<Held<X>>) -> Held<X> {
    families.into_iter().flatten().collect()
}

pub fn bind<X: Ord, Y: Ord, E>(
    held: Held<X>,
    mut arrow: impl FnMut(X) -> Result<Held<Y>, E>,
) -> Result<Held<Y>, E> {
    let mut result = Held::new();
    for member in held {
        result.extend(arrow(member)?);
    }
    Ok(result)
}

pub fn bind_one<X: Ord, E>(member: X, arrow: impl FnOnce(X) -> Result<X, E>) -> Result<X, E> {
    let mut arrow = Some(arrow);
    let result = bind(unit(member), |member| {
        arrow.take().expect("unit has exactly one member")(member).map(unit)
    })?;
    Ok(result.into_iter().next().expect("deterministic arrow returns one successor"))
}
