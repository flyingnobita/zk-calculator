use std::marker::PhantomData;

use halo2_proofs::{
    arithmetic::FieldExt,
    circuit::{AssignedCell, Chip, Layouter, Value},
    plonk::{Advice, Column, ConstraintSystem, Error, Instance},
};

use crate::chips::{
    add::{AddChip, AddConfig, AddInstructions},
    mul::{MulChip, MulConfig, MulInstructions},
    sub::{SubChip, SubConfig, SubInstructions},
};

/// Top-level arithmetic instruction set
pub trait ArithmeticInstructions<F: FieldExt>:
    AddInstructions<F> + MulInstructions<F> + SubInstructions<F>
{
    /// Numeric variable
    type Num;

    /// Loads a private number into the circuit
    fn load_private(
        &self,
        layouter: impl Layouter<F>,
        value: Value<F>,
    ) -> Result<<Self as ArithmeticInstructions<F>>::Num, Error>;

    /// Expose a number as a public input to the circuit
    fn expose_public(
        &self,
        layouter: impl Layouter<F>,
        num: <Self as ArithmeticInstructions<F>>::Num,
        row: usize,
    ) -> Result<(), Error>;
}

/// Numeric variable type
/// Imported into each chip's implementation
#[derive(Clone)]
pub struct Number<F: FieldExt>(pub AssignedCell<F, F>);

// note: dead code is only allowed on this config because of a warning regarding reading 'b'

/// Top-level arithmetic chip configuration
/// Derived in `Chip::configure`
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ArithmeticConfig {
    /// Advice column for `input_a` and `output`
    a: Column<Advice>,
    /// Advice column for `input_a` and `output`
    b: Column<Advice>,
    /// Instance column for public inputs
    instance: Column<Instance>,
    /// Addition chip configuration
    add_config: AddConfig,
    /// Subtraction chip configuration
    sub_config: SubConfig,
    /// Multiplication chip configuration
    mul_config: MulConfig,
}

/// Arithmetic chip definition
pub struct ArithmeticChip<F: FieldExt> {
    /// ArithmeticChip<F: FieldExt> {
    config: ArithmeticConfig,
    /// Placeholder data
    _marker: PhantomData<F>,
}

/// Arithmetic chip implementation
impl<F: FieldExt> ArithmeticChip<F> {
    /// Construct ArithmeticChip and return
    pub fn construct(
        config: <Self as Chip<F>>::Config,
        _loaded: <Self as Chip<F>>::Loaded,
    ) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }

    /// Configure ArithmeticChip and return the Config
    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        a: Column<Advice>,
        b: Column<Advice>,
        instance: Column<Instance>,
    ) -> <Self as Chip<F>>::Config {
        // configure addition chip
        let add_config = AddChip::configure(meta, a, b);
        // configure substraction chip
        let sub_config = SubChip::configure(meta, a, b);
        // configure multiplication chip
        let mul_config = MulChip::configure(meta, a, b);

        // enable instance equality checks
        meta.enable_equality(instance);

        // return the top-level config
        ArithmeticConfig {
            a,
            b,
            instance,
            add_config,
            sub_config,
            mul_config,
        }
    }
}

/// Halo2 Chip implementation for ArithmeticChip
impl<F: FieldExt> Chip<F> for ArithmeticChip<F> {
    /// Arithmetic configuration
    type Config = ArithmeticConfig;
    /// Loaded data
    type Loaded = ();

    /// return configuration reference
    fn config(&self) -> &Self::Config {
        &self.config
    }

    /// return loaded data reference
    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

/// Arithmetic instruction set implementation for ArithmeticChip
impl<F: FieldExt> ArithmeticInstructions<F> for ArithmeticChip<F> {
    /// Numeric type definition
    type Num = Number<F>;

    /// Loadds a private number into the circuit
    fn load_private(
        &self,
        mut layouter: impl Layouter<F>,
        value: Value<F>,
    ) -> Result<<Self as ArithmeticInstructions<F>>::Num, Error> {
        // get config
        let config = self.config();

        // assign region of gates and return
        layouter.assign_region(
            // region name
            || "load private",
            // assignment
            |mut region| {
                region
                    .assign_advice(|| "privatre input", config.a, 0, || value)
                    .map(Number)
            },
        )
    }

    /// Expose a number as a public input to the circuit
    fn expose_public(
        &self,
        mut layouter: impl Layouter<F>,
        num: <Self as ArithmeticInstructions<F>>::Num,
        row: usize,
    ) -> Result<(), Error> {
        // get config
        let config = self.config();

        // constrain the `num` to equal instance column at a given row,
        // publicly exposing the number
        layouter.constrain_instance(num.0.cell(), config.instance, row)
    }
}

/// Addition instruction set implementation for ArithmeticChip.
impl<F: FieldExt> AddInstructions<F> for ArithmeticChip<F> {
    /// Numeric type definition.
    type Num = Number<F>;

    /// Addition instruction definition.
    fn add(
        &self,
        layouter: &mut impl Layouter<F>,
        a: Self::Num,
        b: Self::Num,
    ) -> Result<Self::Num, Error> {
        // configure the add chip
        let config = self.config().add_config.clone();

        // construct the add chip
        let add_chip = AddChip::<F>::construct(config, ());

        // return the result of add_chip's addition gate
        add_chip.add(layouter, a, b)
    }
}

/// Multiplication instruction set implementation for ArithmeticChip.
impl<F: FieldExt> MulInstructions<F> for ArithmeticChip<F> {
    /// Numeric type definition.
    type Num = Number<F>;

    /// Addition instruction definition.
    fn mul(
        &self,
        layouter: &mut impl Layouter<F>,
        a: Self::Num,
        b: Self::Num,
    ) -> Result<Self::Num, Error> {
        // configure the mul chip
        let config = self.config().mul_config.clone();

        // construct the add chip
        let mul_chip = MulChip::<F>::construct(config, ());

        // return the result of add_chip's multiplication gate
        mul_chip.mul(layouter, a, b)
    }
}

/// Subtraction instruction set implementation for ArithmeticChip.
impl<F: FieldExt> SubInstructions<F> for ArithmeticChip<F> {
    /// Numeric type definition.
    type Num = Number<F>;

    /// Addition instruction definition.
    fn sub(
        &self,
        layouter: &mut impl Layouter<F>,
        a: Self::Num,
        b: Self::Num,
    ) -> Result<Self::Num, Error> {
        // configure the sub chip
        let config = self.config().sub_config.clone();

        // construct the add chip
        let sub_chip = SubChip::<F>::construct(config, ());

        // return the result of sub_chip's subtraction gate
        sub_chip.sub(layouter, a, b)
    }
}
