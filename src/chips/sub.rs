use std::marker::PhantomData;

use halo2_proofs::{
    arithmetic::FieldExt,
    circuit::{Chip, Layouter, Region},
    plonk::{Advice, Column, ConstraintSystem, Error, Selector},
    poly::Rotation,
};

// import Number from top level chip to avoid redeclaring the same Number type for each operator chip
use crate::chips::arithmetic::Number;

/// Subtraction intruction set
pub trait SubInstructions<F: FieldExt>: Chip<F> {
    /// Numeric variable
    type Num;

    /// Subtraction instruction
    /// Takes two inputs and return the sum
    fn sub(
        &self,
        layouter: &mut impl Layouter<F>,
        a: Self::Num,
        b: Self::Num,
    ) -> Result<Self::Num, Error>;
}

/// Subtraction chip configuration
/// Derived during `Chip::configure`
#[derive(Clone, Debug)]
pub struct SubConfig {
    /// Advice column for `input_a` and `output`
    a: Column<Advice>,
    /// Advice column for `input_b`
    b: Column<Advice>,
    /// Subtraction Selector
    sel_sub: Selector,
}

/// Subtraction chip definition
pub struct SubChip<F: FieldExt> {
    /// Subtraction configuration
    config: SubConfig,
    /// Placeholder data
    _marker: PhantomData<F>,
}

/// Subtraction chip implementations
impl<F: FieldExt> SubChip<F> {
    /// Construct SubChip and return
    pub fn construct(
        config: <Self as Chip<F>>::Config,
        _loaded: <Self as Chip<F>>::Loaded,
    ) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }

    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        a: Column<Advice>,
        b: Column<Advice>,
    ) -> <Self as Chip<F>>::Config {
        // enable equality on columns
        meta.enable_equality(a);
        meta.enable_equality(b);

        // get selector
        let sel_sub = meta.selector();

        // define the subtraction gate
        meta.create_gate(
            "sub", // gate name
            |meta| {
                // gate logic

                // query advice value from a on the current rotation
                let lhs = meta.query_advice(a, Rotation::cur());
                // query advice value from b on the current rotation
                let rhs = meta.query_advice(b, Rotation::cur());
                // query advice value from c on the current rotation
                let out = meta.query_advice(a, Rotation::next());

                // query selector
                let sel_sub = meta.query_selector(sel_sub);

                // return an iterable of `selector * (a - b - c)`
                // if `sel_sub == 0`, then lhs, rhs and out are not constrained
                // if `sel_sub != 0`, then lhs, rhs and out are constrained
                vec![sel_sub * (lhs - rhs - out)]
            },
        );

        // return config
        SubConfig { a, b, sel_sub }
    }
}

/// Halo2 Chip implementation for SubChip
impl<F: FieldExt> Chip<F> for SubChip<F> {
    /// Subtraction configuration
    type Config = SubConfig;
    /// Loaded data
    type Loaded = ();

    /// Returns a configuration reference
    fn config(&self) -> &Self::Config {
        &self.config
    }

    /// Returns the loaded data reference
    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

/// Subtraction instruction set implementation for SubChip
impl<F: FieldExt> SubInstructions<F> for SubChip<F> {
    /// Num type definition
    type Num = Number<F>;

    /// Subtraction instruction implementation
    fn sub(
        &self,
        layouter: &mut impl Layouter<F>,
        a: Self::Num,
        b: Self::Num,
    ) -> Result<Self::Num, Error> {
        // get config
        let config = self.config();

        // assign a region of gates and return
        layouter.assign_region(
            // region name
            || "sub",
            // assignment
            |mut region: Region<'_, F>| {
                // enable subtraction gate, set at region offset zero,
                // it will constrain cells zero and one
                config.sel_sub.enable(&mut region, 0)?;

                // copy advice value a to offset zero,
                // column a of the region
                a.0.copy_advice(|| "lhs", &mut region, config.a, 0)?;

                // copy advice value b to offset zero,
                // column b of the region
                b.0.copy_advice(|| "rhs", &mut region, config.b, 0)?;

                // subtract the values in columns a and b at offset zero
                let c = a.0.value().copied() - b.0.value();

                // mutate the region and return
                region
                    // assign the sum c as an advice into column a, offset one
                    .assign_advice(|| "lhs - rhs", config.a, 1, || c)
                    // map result to Number
                    .map(Number)
            },
        )
    }
}
