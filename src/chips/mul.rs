use std::marker::PhantomData;

use halo2_proofs::{
    arithmetic::FieldExt,
    circuit::{Chip, Layouter, Region},
    plonk::{Advice, Column, ConstraintSystem, Error, Selector},
    poly::Rotation,
};

// import Number from top level chip to avoid redeclaring the same Number type for each operator chip
use crate::chips::arithmetic::Number;

/// Multiplication intruction set
pub trait MulInstructions<F: FieldExt>: Chip<F> {
    /// Numeric variable
    type Num;

    /// Multiplication instruction
    /// Takes two inputs and return the sum
    fn mul(
        &self,
        layouter: &mut impl Layouter<F>,
        a: Self::Num,
        b: Self::Num,
    ) -> Result<Self::Num, Error>;
}

/// Multiplication chip configuration
/// Derived during `Chip::configure`
#[derive(Clone, Debug)]
pub struct MulConfig {
    /// Advice column for `input_a` and `output`
    a: Column<Advice>,
    /// Advice column for `input_b`
    b: Column<Advice>,
    /// Multiplication Selector
    sel_mul: Selector,
}

/// Multiplication chip definition
pub struct MulChip<F: FieldExt> {
    /// Multiplication configuration
    config: MulConfig,
    /// Placeholder data
    _marker: PhantomData<F>,
}

/// Multiplication chip implementations
impl<F: FieldExt> MulChip<F> {
    /// Construct MulChip and return
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
        let sel_mul = meta.selector();

        // define the multiplication gate
        meta.create_gate(
            "mul", // gate name
            |meta| {
                // gate logic

                // query advice value from a on the current rotation
                let lhs = meta.query_advice(a, Rotation::cur());
                // query advice value from b on the current rotation
                let rhs = meta.query_advice(b, Rotation::cur());
                // query advice value from c on the current rotation
                let out = meta.query_advice(a, Rotation::next());

                // query selector
                let sel_mul = meta.query_selector(sel_mul);

                // return an iterable of `selector * (a * b - c)`
                // if `sel_mul == 0`, then lhs, rhs and out are not constrained
                // if `sel_mul != 0`, then lhs, rhs and out are constrained
                vec![sel_mul * (lhs * rhs - out)]
            },
        );

        // return config
        MulConfig { a, b, sel_mul }
    }
}

/// Halo2 Chip implementation for MulChip
impl<F: FieldExt> Chip<F> for MulChip<F> {
    /// Multiplication configuration
    type Config = MulConfig;
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

/// Multiplication instruction set implementation for MulChip
impl<F: FieldExt> MulInstructions<F> for MulChip<F> {
    /// Num type definition
    type Num = Number<F>;

    /// Multiplication instruction implementation
    fn mul(
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
            || "mul",
            // assignment
            |mut region: Region<'_, F>| {
                // enable multiplication gate, set at region offset zero,
                // it will constrain cells zero and one
                config.sel_mul.enable(&mut region, 0)?;

                // copy advice value a to offset zero,
                // column a of the region
                a.0.copy_advice(|| "lhs", &mut region, config.a, 0)?;

                // copy advice value b to offset zero,
                // column b of the region
                b.0.copy_advice(|| "rhs", &mut region, config.b, 0)?;

                // multiply the values in columns a and b at offset zero
                let c = a.0.value().copied() * b.0.value();

                // mutate the region and return
                region
                    // assign the sum c as an advice into column a, offset one
                    .assign_advice(|| "lhs * rhs", config.a, 1, || c)
                    // map result to Number
                    .map(Number)
            },
        )
    }
}
