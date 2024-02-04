use halo2_proofs::arithmetic::Field;
use halo2_proofs::circuit::{AssignedCell, Layouter, SimpleFloorPlanner, Value};
use halo2_proofs::dev::MockProver;
use halo2_proofs::pasta::Fp;
use halo2_proofs::plonk::*;
use halo2_proofs::poly::Rotation;
use plotters::prelude::WHITE;

#[derive(Clone, Debug, Copy)]
pub struct FibConfig {
    pub selector: Selector,
    pub a: Column<Advice>,
    pub b: Column<Advice>,
    pub c: Column<Advice>,
    pub target: Column<Instance>,
}

pub struct FibChip {
    pub config: FibConfig,
}

impl FibChip {
    fn configure<F: Field>(meta: &mut ConstraintSystem<F>) -> FibConfig {
        let selector = meta.selector();
        let a = meta.advice_column();
        let b = meta.advice_column();
        let c = meta.advice_column();
        let target = meta.instance_column();

        meta.enable_equality(a);
        meta.enable_equality(b);
        meta.enable_equality(c);
        meta.enable_equality(target);

        meta.create_gate("fib(plus)", |meta| {
            let selector = meta.query_selector(selector);
            let num_a = meta.query_advice(a, Rotation::cur());
            let num_b = meta.query_advice(b, Rotation::cur());
            let num_c = meta.query_advice(c, Rotation::cur());
            vec![("a + b = c", selector * (num_a + num_b - num_c))]
        });
        FibConfig {
            selector,
            a,
            b,
            c,
            target,
        }
    }

    fn assign_first_row<F: Field>(
        &self,
        mut layouter: impl Layouter<F>,
        a: Value<F>,
        b: Value<F>,
    ) -> Result<(AssignedCell<F, F>, AssignedCell<F, F>), Error> {
        layouter.assign_region(
            || "first row",
            |mut region| {
                self.config.selector.enable(&mut region, 0)?;
                region
                    .assign_advice(|| "load a", self.config.a, 0, || a)
                    .expect("加载a失败");
                let cur_b = region
                    .assign_advice(|| "load b", self.config.b, 0, || b)
                    .expect("加载b失败");
                let cur_c = region
                    .assign_advice(|| "calc c", self.config.c, 0, || a + b)
                    .expect("填写c失败");
                Ok((cur_b, cur_c))
            },
        )
    }

    fn assign_next_row<F: Field>(
        &self,
        mut layouter: impl Layouter<F>,
        pre_b: &AssignedCell<F, F>,
        pre_c: &AssignedCell<F, F>,
    ) -> Result<(AssignedCell<F, F>, AssignedCell<F, F>), Error> {
        layouter.assign_region(
            || "next row",
            |mut region| {
                self.config.selector.enable(&mut region, 0)?;
                let cur_a = pre_b
                    .copy_advice(|| "copy last b to a", &mut region, self.config.a, 0)
                    .expect("拷贝到a失败");
                let cur_b = pre_c
                    .copy_advice(|| "copy last c to b", &mut region, self.config.b, 0)
                    .expect("拷贝到b失败");
                let value_c = cur_a.value_field().evaluate() + cur_b.value_field().evaluate();
                let cur_c = region
                    .assign_advice(|| "calc cur c", self.config.c, 0, || value_c)
                    .expect("填写c失败");
                Ok((cur_b, cur_c))
            },
        )
    }

    fn expose_public<F: Field>(
        &self,
        mut layouter: impl Layouter<F>,
        cell: &AssignedCell<F, F>,
        row: usize,
    ) -> Result<(), Error> {
        layouter.constrain_instance(cell.cell(), self.config.target, row)
    }
}

#[derive(Default)]
pub struct FibCircuit<F: Field> {
    pub a: Value<F>, // 初始a=1
    pub b: Value<F>, // 初始b=1
}

impl<F: Field> Circuit<F> for FibCircuit<F> {
    type Config = FibConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        FibChip::configure(meta)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let fib = FibChip { config };
        // 初始化第一行
        let (mut b, mut c) = fib
            .assign_first_row(layouter.namespace(|| "first row"), self.a, self.b)
            .expect("first row fail");
        // 循环填写下一行
        for _i in 3..4 {
            let (next_b, next_c) = fib
                .assign_next_row(layouter.namespace(|| "next row"), &b, &c)
                .expect("next row fail");
            b = next_b;
            c = next_c;
        }
        // 暴露结果
        fib.expose_public(layouter, &c, 0)?;
        Ok(())
    }
}
